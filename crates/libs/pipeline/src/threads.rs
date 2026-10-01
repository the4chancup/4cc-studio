//! The rayon worker count: the requested setting, else the logical cores
//! minus one for the reader and writer.

/// The rayon worker count: `requested` when non-zero (the `thread_count`
/// setting), else the logical cores minus one for the reader and writer, at
/// least one.
pub fn thread_count_detect(requested: usize) -> usize {
    // `available_parallelism` failing counts as one logical core.
    thread_count_for(
        requested,
        std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1),
    )
}

/// `requested` wins when non-zero; else `logical` minus one, at least one.
fn thread_count_for(requested: usize, logical: usize) -> usize {
    if requested > 0 {
        return requested;
    }
    logical.saturating_sub(1).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_requested_count_is_used_verbatim() {
        assert_eq!(thread_count_for(4, 8), 4);
        assert_eq!(thread_count_for(32, 8), 32);
    }

    #[test]
    fn zero_reserves_a_core_for_reader_and_writer() {
        assert_eq!(thread_count_for(0, 1), 1);
        assert_eq!(thread_count_for(0, 2), 1);
        assert_eq!(thread_count_for(0, 8), 7);
    }

    #[test]
    fn detect_never_returns_zero() {
        assert!(thread_count_detect(0) >= 1);
        assert_eq!(thread_count_detect(3), 3);
    }
}
