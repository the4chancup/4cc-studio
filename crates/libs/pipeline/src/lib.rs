//! The shared scaffolding every compiling tool (Team compiler, Balls
//! compiler, Stadium compiler) runs on: the run's memory budget and its cap
//! and cancellation, the rayon worker count, and the `CpkStem` every emitted
//! CPK name is validated against.

mod budget;
mod cpk_stem;
mod memory;
mod threads;

pub use budget::{Cancelled, MemoryBudget, Permit};
pub use cpk_stem::{CpkStem, CpkStemError};
pub use memory::{FALLBACK_AVAILABLE, memory_cap};
pub use threads::thread_count_detect;
