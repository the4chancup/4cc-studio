//! The Save editor's command line run in-process (`plans/save_editor.md` "CLI"): the refusals,
//! the findings and the files written, observed as the exit code, the `Message` events the tool
//! emits (which the binary prints one line each) and the saves and documents on disk.

#[path = "../common/mod.rs"]
mod common;

mod export_toml;

use common::Sandbox;
