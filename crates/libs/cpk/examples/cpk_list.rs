//! Lists a CPK archive's entries, one line each: uncompressed size, stored size and path, in
//! `TOC ` order. A nested `.cpk` entry (a pre-Fox face package) is opened and its own entries
//! listed under it, as `<entry path>/<member path>`.
//!
//! A debugging aid until the suite has a CPK tool of its own.
//!
//! Usage: `cpk_list <archive.cpk> [<substring>]`: with a substring, only the lines whose path
//! contains it (letter case ignored).

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, Write};

use cpk::CpkArchive;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let (path, filter) = match args.as_slice() {
        [_, path] => (path, None),
        [_, path, filter] => (path, Some(filter.to_lowercase())),
        _ => return Err("usage: cpk_list <archive.cpk> [<substring>]".into()),
    };
    let archive = CpkArchive::open(BufReader::new(File::open(path)?))?;
    list(
        archive,
        "",
        filter.as_deref(),
        &mut std::io::stdout().lock(),
    )
}

fn list<R: Read + Seek>(
    mut archive: CpkArchive<R>,
    prefix: &str,
    filter: Option<&str>,
    out: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    // Copied out: reading a nested archive's bytes borrows the archive mutably.
    for entry in archive.entries().to_vec() {
        let path = format!("{prefix}{}", entry.path);
        if filter.is_none_or(|filter| path.to_lowercase().contains(filter)) {
            writeln!(out, "{:>10} {:>10} {path}", entry.size, entry.packed_size)?;
        }
        if entry.path.to_lowercase().ends_with(".cpk") {
            let nested = CpkArchive::open(Cursor::new(archive.read(&entry)?))?;
            list(nested, &format!("{path}/"), filter, out)?;
        }
    }
    Ok(())
}
