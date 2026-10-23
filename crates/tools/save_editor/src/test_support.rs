//! The unit tests' scratch folders, under the test binary's own build folder
//! rather than the system Temp.

use std::fs;
use std::path::{Path, PathBuf};

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

/// A `DpFileList.bin` listing `names` in load order: a 16-byte header whose second word is the
/// count, then one 48-byte NUL-padded record per CPK.
pub(crate) fn dp_file_list(names: &[&str]) -> Vec<u8> {
    let mut bytes = vec![0; 4];
    bytes.extend(u32::try_from(names.len()).unwrap().to_le_bytes());
    bytes.extend([0; 8]);
    for name in names {
        let mut record = [0; 48];
        record[..name.len()].copy_from_slice(name.as_bytes());
        bytes.extend(record);
    }
    bytes
}

/// A CPK's entries as the tests give them (`(entry path, bytes)`).
pub(crate) type CpkEntries<'a> = &'a [(&'a str, &'a [u8])];

/// `path` written as a CPK holding `entries` (`(entry path, bytes)`).
pub(crate) fn write_cpk(path: &Path, entries: CpkEntries<'_>) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut cpk = cpk::CpkWriter::new(fs::File::create(path).unwrap(), "test").unwrap();
    for (entry, bytes) in entries {
        cpk.add(entry, bytes, None).unwrap();
    }
    cpk.finish().unwrap();
}

/// A PES folder under `test`'s scratch holding `download/DpFileList.bin`, listing `cpks`' names
/// in load order, each written with its `entries` (`(entry path, bytes)`).
pub(crate) fn pes_folder(test: &str, cpks: &[(&str, CpkEntries<'_>)]) -> PathBuf {
    let download = scratch(test).join("PES/download");
    fs::create_dir_all(&download).unwrap();
    let names: Vec<&str> = cpks.iter().map(|(name, _)| *name).collect();
    fs::write(download.join("DpFileList.bin"), dp_file_list(&names)).unwrap();
    for (name, entries) in cpks {
        write_cpk(&download.join(name), entries);
    }
    download.parent().unwrap().to_path_buf()
}
