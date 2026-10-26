//! A temporary folder a unit test owns, shared by the binary's test modules.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// `<temp>/studio_<name>_<pid>_<n>` (`n` from a process counter, so no two calls share a
/// folder), created in `new` and removed in `Drop`, so a test that panics still leaves the
/// temp folder clean.
pub(crate) struct TestDir(PathBuf);

impl TestDir {
    pub(crate) fn new(name: &str) -> TestDir {
        static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "studio_{name}_{}_{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        TestDir(dir)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        // A drop cannot fail the test, and a leftover folder in the temp folder harms
        // nothing: the failure is logged, not raised.
        if let Err(error) = fs::remove_dir_all(&self.0) {
            log::debug!("test folder {} not removed: {error}", self.0.display());
        }
    }
}
