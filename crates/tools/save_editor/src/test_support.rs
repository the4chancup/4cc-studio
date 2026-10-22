//! The unit tests' scratch folders, under the test binary's own build folder
//! rather than the system Temp.

use std::path::PathBuf;

/// A fresh, empty scratch folder for `test` under the test binary's build folder
/// (`target/<profile>/save_editor_tests/<test>`): per checkout, so a mutation run's copy
/// has its own, and emptied at the start of each run, so no process id is needed.
pub(crate) fn scratch(test: &str) -> PathBuf {
    // `CARGO_TARGET_TMPDIR` is set for integration tests only, so the build folder
    // comes from the test binary's path (`target/<profile>/deps/<exe>`).
    let deps = std::env::current_exe()
        .expect("current_exe is readable")
        .parent()
        .expect("the test binary sits under target/<profile>/deps")
        .to_path_buf();
    let dir = deps
        .parent()
        .expect("deps sits under target/<profile>")
        .join("save_editor_tests")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the scratch folder is removable");
    }
    std::fs::create_dir_all(&dir).expect("the scratch folder is creatable");
    dir
}
