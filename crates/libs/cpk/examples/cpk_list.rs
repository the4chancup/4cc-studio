//! Lists a CPK archive's entries, one line each: uncompressed size, stored size and path, in
//! `TOC ` order. A nested `.cpk` entry (a pre-Fox face package) is opened and its own entries
//! listed under it, as `<entry path>/<member path>`.
//!
//! A debugging aid until the suite has a CPK tool of its own.
//!
//! Usage: `cpk_list <archive.cpk> [<substring> [<folder>]]`: with a substring, only the lines
//! whose path contains it (letter case ignored, `""` for all); with a folder, each listed
//! entry is also written there under its path, a nested archive's members under
//! `<entry path>.d/` (the archive itself is a file at `<entry path>`).

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, Write};
use std::path::Path;

use cpk::CpkArchive;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let (path, filter, folder) = match args.as_slice() {
        [_, path] => (path, None, None),
        [_, path, filter] => (path, Some(filter.to_lowercase()), None),
        [_, path, filter, folder] => (path, Some(filter.to_lowercase()), Some(Path::new(folder))),
        _ => return Err("usage: cpk_list <archive.cpk> [<substring> [<folder>]]".into()),
    };
    let archive = CpkArchive::open(BufReader::new(File::open(path)?))?;
    let mut out = std::io::stdout().lock();
    list(archive, "", "", filter.as_deref(), folder, &mut out)
}

/// Lists `archive`'s entries as `prefix` + path and writes each listed one to `folder` +
/// `extract_prefix` + path.
fn list<R: Read + Seek>(
    mut archive: CpkArchive<R>,
    prefix: &str,
    extract_prefix: &str,
    filter: Option<&str>,
    folder: Option<&Path>,
    out: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    // Copied out: reading a nested archive's bytes borrows the archive mutably.
    for entry in archive.entries().to_vec() {
        let path = format!("{prefix}{}", entry.path);
        if filter.is_none_or(|filter| path.to_lowercase().contains(filter)) {
            writeln!(out, "{:>10} {:>10} {path}", entry.size, entry.packed_size)?;
            if let Some(folder) = folder {
                let target = folder.join(format!("{extract_prefix}{}", entry.path));
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(target, archive.read(&entry)?)?;
            }
        }
        if entry.path.to_lowercase().ends_with(".cpk") {
            let nested = CpkArchive::open(Cursor::new(archive.read(&entry)?))?;
            let nested_extract = format!("{extract_prefix}{}.d/", entry.path);
            list(
                nested,
                &format!("{path}/"),
                &nested_extract,
                filter,
                folder,
                out,
            )?;
        }
    }
    Ok(())
}
