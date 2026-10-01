//! The run's memory budget: tasks acquire their whole charge before loading,
//! the writer releases it by dropping the batch's `Permit`. An oversized
//! request waits for an empty budget rather than for capacity it can never
//! get, and while one waits, ordinary requests wait behind it — otherwise
//! `in_flight` never reaches zero.

use std::sync::{Arc, Condvar, Mutex, MutexGuard};

/// A byte budget shared by a run's tasks: `acquire` blocks until the bytes
/// fit.
pub struct MemoryBudget {
    cap: usize,
    state: Mutex<BudgetState>,
    notify: Condvar,
}

struct BudgetState {
    in_flight: usize,
    /// Oversized requests waiting for the pipeline to drain.
    oversized_waiting: usize,
    cancelled: bool,
}

/// Bytes held from a `MemoryBudget`, released when dropped.
pub struct Permit {
    budget: Arc<MemoryBudget>,
    size: usize,
}

/// The run was cancelled while (or before) waiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the run was cancelled")]
pub struct Cancelled;

impl MemoryBudget {
    /// A budget of `cap` bytes; shared via `Arc` because a permit outlives its
    /// acquiring thread (it travels with the batch to the writer).
    pub fn new(cap: usize) -> Arc<MemoryBudget> {
        Arc::new(MemoryBudget {
            cap,
            state: Mutex::new(BudgetState {
                in_flight: 0,
                oversized_waiting: 0,
                cancelled: false,
            }),
            notify: Condvar::new(),
        })
    }

    /// Blocks until `size` bytes fit (or, for a request over the cap, until
    /// the pipeline is empty), then charges them. `Err(Cancelled)` when the
    /// budget is cancelled before or during the wait; no bytes are taken.
    pub fn acquire(self: &Arc<Self>, size: usize) -> Result<Permit, Cancelled> {
        let mut state = self.lock();
        if size > self.cap {
            // An oversized request waits for the pipeline to drain, then
            // proceeds alone — waiting for ordinary capacity would deadlock.
            state.oversized_waiting += 1;
            while !state.cancelled && state.in_flight > 0 {
                state = self
                    .notify
                    .wait(state)
                    .expect("nothing panics while holding the budget lock");
            }
            state.oversized_waiting -= 1;
        } else {
            // `oversized_waiting` is what lets an oversized request ever run:
            // without it, ordinary arrivals keep `in_flight` above zero for
            // the whole run.
            while !state.cancelled
                && (state.oversized_waiting > 0
                    || state
                        .in_flight
                        .checked_add(size)
                        .is_none_or(|total| total > self.cap))
            {
                state = self
                    .notify
                    .wait(state)
                    .expect("nothing panics while holding the budget lock");
            }
        }
        if state.cancelled {
            return Err(Cancelled);
        }
        state.in_flight += size;
        Ok(Permit {
            budget: Arc::clone(self),
            size,
        })
    }

    /// Wakes every waiter; every later `acquire` returns `Cancelled`.
    pub fn cancel(&self) {
        self.lock().cancelled = true;
        self.notify.notify_all();
    }

    /// Nothing panics while holding the lock, so a poisoned mutex is a bug in
    /// this module.
    fn lock(&self) -> MutexGuard<'_, BudgetState> {
        self.state
            .lock()
            .expect("nothing panics while holding the budget lock")
    }

    #[cfg(test)]
    fn in_flight(&self) -> usize {
        self.lock().in_flight
    }

    #[cfg(test)]
    fn oversized_waiting(&self) -> usize {
        self.lock().oversized_waiting
    }
}

impl Permit {
    /// The bytes this permit holds.
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        let mut state = self.budget.lock();
        assert!(
            state.in_flight >= self.size,
            "memory-budget permit released twice"
        );
        state.in_flight -= self.size;
        // An ordinary waiter and an oversized waiter wait for different
        // conditions on the one Condvar, so every release wakes them all.
        self.budget.notify.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
    use std::thread;
    use std::time::Duration;

    /// Long enough that a scheduling hiccup never outlives it; short enough
    /// that a lost wake-up fails the test rather than hanging it.
    const GUARD: Duration = Duration::from_secs(5);
    /// How long "still waiting" takes to observe.
    const BLOCKED: Duration = Duration::from_millis(100);

    /// A thread that acquires `size`, reports, holds the permit until
    /// `release` is dropped, then reports the release.
    struct Waiter {
        acquired: Receiver<Result<(), Cancelled>>,
        released: Receiver<()>,
        release: Sender<()>,
    }

    fn waiter(budget: &Arc<MemoryBudget>, size: usize) -> Waiter {
        let (acquired_tx, acquired) = channel();
        let (release, release_rx) = channel::<()>();
        let (released_tx, released) = channel();
        let budget = Arc::clone(budget);
        thread::spawn(move || {
            let permit = budget.acquire(size);
            acquired_tx
                .send(permit.as_ref().map(|_| ()).map_err(|e| *e))
                .unwrap();
            if let Ok(permit) = permit {
                // The sender going away releases the hold.
                while release_rx.recv().is_ok() {}
                drop(permit);
                released_tx.send(()).unwrap();
            }
        });
        Waiter {
            acquired,
            released,
            release,
        }
    }

    impl Waiter {
        /// The waiter acquired (or was cancelled) within `GUARD`.
        fn acquired(&self) -> Result<(), Cancelled> {
            self.acquired
                .recv_timeout(GUARD)
                .expect("a waiter must report within GUARD")
        }

        /// The waiter is still waiting after `BLOCKED`.
        fn blocked(&self) {
            assert!(
                matches!(
                    self.acquired.recv_timeout(BLOCKED),
                    Err(RecvTimeoutError::Timeout)
                ),
                "the waiter was not blocked"
            );
        }

        /// Lets the held permit go and confirms it released.
        fn release(self) {
            drop(self.release);
            self.released
                .recv_timeout(GUARD)
                .expect("a released waiter must report within GUARD");
        }
    }

    #[test]
    fn a_request_that_fits_acquires_immediately() {
        let budget = MemoryBudget::new(10);
        let permit = budget.acquire(5).unwrap();
        assert_eq!(permit.size(), 5);
        assert_eq!(budget.in_flight(), 5);
        drop(permit);
        assert_eq!(budget.in_flight(), 0);
    }

    #[test]
    fn an_ordinary_request_waits_for_room_and_wakes_on_release() {
        let budget = MemoryBudget::new(100);
        let held = budget.acquire(80).unwrap();
        let waiter = waiter(&budget, 50);
        waiter.blocked();
        drop(held);
        assert_eq!(waiter.acquired(), Ok(()));
        waiter.release();
        assert_eq!(budget.in_flight(), 0);
    }

    #[test]
    fn an_oversized_request_waits_for_empty_and_runs_alone() {
        let budget = MemoryBudget::new(100);
        let held = budget.acquire(80).unwrap();
        let oversized = waiter(&budget, 200);
        // The ordinary request must observe the oversized one registered as
        // waiting — a sleep here would only hope the spawn scheduled it.
        let deadline = std::time::Instant::now() + GUARD;
        while budget.oversized_waiting() == 0 {
            assert!(
                std::time::Instant::now() < deadline,
                "the oversized waiter never registered"
            );
            thread::yield_now();
        }
        // 80 + 20 fits the cap, but an oversized waiter goes first: the
        // ordinary request waits behind it or `in_flight` never drains.
        let ordinary = waiter(&budget, 20);
        oversized.blocked();
        ordinary.blocked();
        drop(held);
        assert_eq!(oversized.acquired(), Ok(()));
        assert_eq!(budget.in_flight(), 200);
        ordinary.blocked();
        oversized.release();
        assert_eq!(ordinary.acquired(), Ok(()));
        ordinary.release();
        assert_eq!(budget.in_flight(), 0);
    }

    #[test]
    fn two_oversized_waiters_complete_one_at_a_time() {
        let budget = MemoryBudget::new(100);
        let held = budget.acquire(10).unwrap();
        let (tx, rx) = channel();
        let mut releases = Vec::new();
        let mut joins = Vec::new();
        for size in [150usize, 160] {
            let budget = Arc::clone(&budget);
            let tx = tx.clone();
            let (release_tx, release_rx) = channel::<()>();
            releases.push(release_tx);
            joins.push(thread::spawn(move || {
                let permit = budget.acquire(size).unwrap();
                // Still holding: the runner sees the whole budget as its own.
                tx.send((size, budget.in_flight())).unwrap();
                while release_rx.recv().is_ok() {}
                drop(permit);
            }));
        }
        // Both wait for the pipeline to empty; an empty drain report before
        // then means one ran early.
        assert!(matches!(
            rx.recv_timeout(BLOCKED),
            Err(RecvTimeoutError::Timeout)
        ));
        drop(held);
        let (size, in_flight) = rx.recv_timeout(GUARD).unwrap();
        assert_eq!(in_flight, size, "the oversized request did not run alone");
        assert!(matches!(
            rx.recv_timeout(BLOCKED),
            Err(RecvTimeoutError::Timeout)
        ));
        drop(releases);
        let (size, in_flight) = rx.recv_timeout(GUARD).unwrap();
        assert_eq!(in_flight, size);
        for handle in joins {
            handle.join().unwrap();
        }
        assert_eq!(budget.in_flight(), 0);
    }

    #[test]
    fn many_waiters_never_lose_a_wakeup() {
        let budget = MemoryBudget::new(64);
        let (done_tx, done_rx) = channel();
        let handles: Vec<_> = (0..8)
            .map(|thread_index| {
                let budget = Arc::clone(&budget);
                let done_tx = done_tx.clone();
                thread::spawn(move || {
                    for round in 0..200 {
                        // A mix of ordinary and oversized charges.
                        let size = if (thread_index + round) % 7 == 0 {
                            100
                        } else {
                            1 + (round % 30)
                        };
                        drop(budget.acquire(size).unwrap());
                    }
                    done_tx.send(()).unwrap();
                })
            })
            .collect();
        for _ in &handles {
            done_rx
                .recv_timeout(GUARD)
                .expect("every worker finishes within GUARD");
        }
        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(budget.in_flight(), 0);
    }

    #[test]
    fn cancel_wakes_every_waiter_and_a_held_permit_still_releases() {
        let budget = MemoryBudget::new(100);
        let held = budget.acquire(100).unwrap();
        let ordinary = waiter(&budget, 50);
        let oversized = waiter(&budget, 200);
        ordinary.blocked();
        oversized.blocked();
        budget.cancel();
        assert_eq!(ordinary.acquired(), Err(Cancelled));
        assert_eq!(oversized.acquired(), Err(Cancelled));
        drop(held);
        assert_eq!(budget.in_flight(), 0);
        assert_eq!(budget.acquire(10).map(|_| ()), Err(Cancelled));
    }
}
