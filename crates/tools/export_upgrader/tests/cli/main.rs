//! The Export upgrader's command line run in-process
//! (`plans/export_upgrader.md` "CLI"): the refusals, the findings and the files
//! written, observed as the exit code, the `Message` events the tool emits
//! (which the binary prints one line each) and the folders on disk.

#[path = "../common/mod.rs"]
mod common;

mod cli;
mod input;
mod roots;
