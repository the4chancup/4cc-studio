//! The Team compiler: Studio-format aesthetics exports compiled to CPK archives.
//!
//! Only the tracer bullet exists so far: `compile_tracer` compiles the Phase 3
//! subset (a Fox face plus kits) for the parity test; the real pipeline replaces
//! it in later steps.

mod tracer;

pub use tracer::compile_tracer;
