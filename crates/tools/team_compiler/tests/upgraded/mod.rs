//! The parity tests' upgraded case, its own module so that only the two parity targets declare
//! it (the cli suite shares `common`, and compiles no upgraded export).

use std::fs;
use std::path::{Path, PathBuf};

use crate::common::Sandbox;

/// The exports root for the parity tests' upgraded case: the one export under
/// `tests/fixtures/<tracer>/upgraded/` copied to `<sandbox.root>/upgraded/`,
/// under its name with ` Full ` replaced by ` Midcup `, and that root returned
/// for `compile` to read.
///
/// The upgrader names the export `Full`, the old name carrying no additions
/// word, and a `Full` export missing player kit `p1` gets the compiler's
/// placeholder kit (`Kits/p1`, `kit_colors_missing`, and entries the reference
/// tree has no counterpart for), the tracers' cut keeping kit `g1` alone. The
/// cup tag is not what the parity is about, so the case compiles the golden's
/// content under the twin's tag. The golden's own name is pinned by
/// `export_upgrader`'s `upgraded_golden` test.
pub fn upgraded_as_midcup(sandbox: &Sandbox, tracer: &str) -> PathBuf {
    let upgraded = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(tracer)
        .join("upgraded");
    let exports: Vec<PathBuf> = fs::read_dir(&upgraded)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(exports.len(), 1, "upgraded/ holds the one golden export");
    let name = exports[0].file_name().unwrap().to_str().unwrap();
    assert!(
        name.contains(" Full "),
        "{name}: the golden is a Full export"
    );
    let root = sandbox.root.join("upgraded");
    copy_dir(&exports[0], &root.join(name.replace(" Full ", " Midcup ")));
    root
}

/// Copies the folder `from` to `to`, recursively.
fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}
