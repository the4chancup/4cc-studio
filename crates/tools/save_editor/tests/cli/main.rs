//! The Save editor's command line run in-process (`plans/save_editor.md` "CLI"): the refusals,
//! the findings and the files written, observed as the exit code, the `Message` events the tool
//! emits (which the binary prints one line each) and the saves and documents on disk.

#[path = "../common/mod.rs"]
mod common;

mod apply_patch;
mod diff;
mod export_toml;
mod import_toml;
mod transplant;

use std::fs;
use std::path::{Path, PathBuf};

use common::Sandbox;
use pes_savefile::EditFile;
use pes_savefile::container::SaveContainer;
use pes_savefile::model::player::STRIPPED_APPEARANCE_ID;
use pes_version::PesVersion;

/// The salt the tests' own writes use; a save's bytes depend on it, its payload does not.
const TEST_SALT: [u8; 320] = [7; 320];

/// The decrypted payload of the save at `path`: what a byte-level comparison of two saves
/// looks at, since every write re-encrypts under a fresh salt.
fn payload(path: &Path) -> Vec<u8> {
    SaveContainer::decrypt(&fs::read(path).unwrap())
        .unwrap()
        .payload
}

/// The payload the save at `path` has after `edit` is applied to its model through the lib:
/// what a command that makes the same edit must write.
fn payload_after(path: &Path, edit: impl FnOnce(&mut EditFile)) -> Vec<u8> {
    let mut file = EditFile::load(path).unwrap();
    edit(&mut file);
    SaveContainer::decrypt(&file.to_bytes(&TEST_SALT).unwrap())
        .unwrap()
        .payload
}

impl Sandbox {
    /// Writes the PES 21 fixture save with every player's appearance id at -1 (what
    /// `strip-and-seed` leaves) at `relative`.
    fn stripped_fixture(&self, relative: &str) -> PathBuf {
        let mut file =
            EditFile::from_bytes(&pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
        for player in file.players_mut() {
            player.appearance_id = Some(STRIPPED_APPEARANCE_ID);
        }
        let path = self.path(relative);
        fs::write(&path, file.to_bytes(&TEST_SALT).unwrap()).unwrap();
        path
    }
}
