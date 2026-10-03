# 4cc Studio — Library crates plan: `libs/pipeline`

Part of the [Library crates plan](README.md). The shared scaffolding every compiling tool (Team
compiler, Balls compiler, Stadium compiler) runs on. The reasons behind each piece are in the core
plan's [Parallelism](../core/parallelism.md) section and the Team compiler's
[pipeline walkthrough](../team_compiler/pipeline.md); this part is the crate's contract.

The crate holds four pieces, built in Phase 3: the memory budget and its cap, the thread count
and `CpkStem`. The folder watcher and the check cache join with live validation (Phase 8); the
browser variants (`core/gui.md` "Browser deployment: Studio Web") with the web build's pipeline
tier.

## Crate layout

```
crates/libs/pipeline/
├── Cargo.toml          # thiserror; windows (cfg(windows), Win32_System_SystemInformation)
└── src/
    ├── lib.rs          # re-exports
    ├── budget.rs       # MemoryBudget, Permit, Cancelled
    ├── memory.rs       # memory_cap: the available physical memory, per OS
    ├── threads.rs      # thread_count_detect
    └── cpk_stem.rs     # CpkStem, CpkStemError
```

## Memory budget

```rust
/// A byte budget shared by a run's tasks: `acquire` blocks until the bytes fit.
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
    pub fn new(cap: usize) -> Arc<MemoryBudget>;
    pub fn acquire(self: &Arc<Self>, size: usize) -> Result<Permit, Cancelled>;
    /// Wakes every waiter; every later `acquire` returns `Cancelled`.
    pub fn cancel(&self);
}

impl Permit {
    pub fn size(&self) -> usize;
}
```

Admission rules:

- **Ordinary request** (`size <= cap`): waits until `in_flight + size <= cap` and no oversized
  request is waiting, then adds `size`. The second condition is what lets an oversized request
  ever run: without it, ordinary tasks arriving faster than others finish keep `in_flight` above
  zero for the whole run.
- **Oversized request** (`size > cap`): counts itself in `oversized_waiting`, waits until
  `in_flight == 0`, then takes the budget alone (`in_flight = size`) and leaves the waiting count.
  It never waits for ordinary capacity, which can never satisfy it. Two oversized requests run one
  after the other, each once the pipeline is empty again.
- **Release** is the `Permit`'s `Drop`: it subtracts its size and wakes every waiter
  (`notify_all`: an ordinary waiter and an oversized waiter wait for different conditions on one
  `Condvar`). A permit travels with its task's `TaskBatch` to the writer, which drops it once the
  batch's entries are written; that is why it holds an `Arc`, not a borrow.
- **Cancellation** sets `cancelled` under the same mutex and wakes every waiter; a waiter that
  wakes into a cancelled budget returns `Cancelled` without taking bytes. Permits already held
  still release normally.
- The condition and the values it guards share one `Mutex`: updating a counter outside it could
  notify between a waiter's check and its wait and lose the wake-up. A poisoned mutex is a bug in
  this module (nothing panics while holding it), so `lock()` failures `expect`.
- A task acquires once, for its whole charge, before loading. Phase 3's tasks (a player's face
  content, a kit) know their source sizes up front; the growing permit that charges decoded
  textures and converted models as they are allocated is the open "Complete memory accounting"
  item in `team_compiler/pipeline.md`, decided with Phase 4's processing.

**What a solid `.7z` is charged.** The structure pass needs a solid `.7z` export's small metadata
(`players.txt`, `refs.txt`, `notes.txt`), and `archives` decompresses the whole archive on its
first read. The structure pass (`check`, and the reading phase of `compile`) acquires the sum of
the archive's `entries()` sizes before that read and drops the permit with the `Archive` once the
metadata bytes are copied out; `compile` opens the archive again for its tasks and charges it
again until they drain (`team_compiler/pipeline.md` step 3). Keeping every 7z export decompressed
from the structure pass to its tasks would hold all of them at once while the run is planned,
which is the residency the budget exists to prevent; the second decompression costs time only
for 7z exports, and the main workflow uses folders.

## Memory cap

```rust
/// The budget cap for a run: `percent` of the physical memory available now, or of
/// `FALLBACK_AVAILABLE` where the OS cannot say. `percent` is validated by the caller's
/// settings (`0 < percent <= 100`).
pub fn memory_cap(percent: f64) -> usize;

/// What `memory_cap` assumes is available where the OS cannot say: 4 GiB.
pub const FALLBACK_AVAILABLE: u64 = 4 << 30;
```

The base is the memory **available** when the run starts, not the machine's total: the game,
a browser or Blender may hold much of it, and a cap on the total would push the run into swap.
It is read once, at run start; the budget does not follow later changes. On Windows it is
`GlobalMemoryStatusEx`'s `ullAvailPhys` (the `windows` crate, one `unsafe` call), which counts the
standby cache as available; on Linux the `MemAvailable` line of `/proc/meminfo` (no `unsafe`),
the kernel's own estimate including reclaimable cache, where `sysinfo`'s free memory leaves
the cache out and would undercount. Any other platform, and either read failing, uses
`FALLBACK_AVAILABLE`; none of them is a target. There is no minimum cap: the oversized branch
keeps the budget making progress at any cap, and a machine with little memory free should run
tasks one at a time. The product is computed in `f64` and saturates at `usize::MAX`. The OS
read is a private `fn available_memory() -> Option<u64>`; `memory_cap` takes it apart from a
private `fn cap_of(available: u64, percent: f64) -> usize`, which the tests cover.

## Thread count

```rust
/// The rayon worker count: `requested` when non-zero (the `thread_count` setting), else the
/// logical cores minus one for the reader and writer, at least one.
pub fn thread_count_detect(requested: usize) -> usize;
```

`available_parallelism()` failing counts as one logical core. The detection is a private
`fn thread_count_for(requested: usize, logical: usize) -> usize`, so the one- and two-core cases
are tested without such a host.

## `CpkStem`

```rust
/// A CPK file name without its `.cpk`: valid on every platform the game and DLC run on.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CpkStem(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CpkStemError {
    #[error("empty")]
    Empty,
    #[error("{len} characters, more than 28")]
    TooLong { len: usize },
    #[error("'{character}' is not a letter, digit, '_', '-' or '.'")]
    InvalidCharacter { character: char },
    #[error("ends with '.'")]
    TrailingDot,
    #[error("ends with '.cpk'; the extension is added")]
    CpkSuffix,
    #[error("'{name}' is a reserved Windows device name")]
    ReservedName { name: String },
}

impl CpkStem {
    pub fn new(text: &str) -> Result<CpkStem, CpkStemError>;
    pub fn as_str(&self) -> &str;
}

impl fmt::Display for CpkStem { /* the stem */ }
```

The contract is `team_compiler/pipeline.md` "Writer" item 5: 1–28 characters, each ASCII
alphanumeric, `_`, `-` or `.`; no trailing `.`; no `.cpk` suffix in any case; and not a Windows
reserved device name (`CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9`, any case), judged
on the part before the first `.`, since Windows reserves `con.x` as it does `con`. Checks run in
the order of the enum, so the first failing rule is the one reported. The length is counted in
characters, which the character rule makes equal to bytes. Uniqueness among a run's stems is the
consumer's check and is case-insensitive; on an ASCII-only stem that is
`eq_ignore_ascii_case`, so the type adds no key method for it.

## Tests

The `team_compiler/testing.md` "Infrastructure" cases for these three:

- **Budget:** an ordinary request blocks while the cap is full and wakes on a release; an oversized
  request waits for `in_flight == 0`, then runs alone, while a later ordinary request waits for it;
  two oversized waiters both complete, one at a time (stress: several threads, many rounds, a
  timeout guard so a lost wake-up fails the test rather than hanging it); `cancel` wakes a blocked
  ordinary and a blocked oversized waiter with `Cancelled`, and a held permit still releases.
- **Memory cap:** `cap_of` gives 80% of 10 GiB as 8 GiB, 100% as the whole, a tiny percent as
  its share (no floor), and saturates rather than overflowing on a huge `available`;
  `available_memory()` returns `Some` on Windows and Linux (the CI and maintainer platforms), so
  a silent fallback there fails a test.
- **Thread count:** a non-zero request is returned as is; on 1 logical core the result is 1, on 2
  it is 1, on 8 it is 7.
- **`CpkStem`:** accepted: `4cc_99_test`, a 28-character stem, `a.b`; refused with its variant:
  empty, 29 characters, a space, `/` and `\`, `con`, `CON.x`, `com1`, `lpt9`, `x.`, `x.CPK`.
  Case collisions are the consumer's (TC-CLI-05 is the Team compiler's, cited in 3.8).
