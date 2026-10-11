//! `extra_mutants list <crate dir>`: one mutant per line, tab-separated
//! (`<path>\t<start>\t<end>\t<kind>\t<line>\t<replacement>\t<description>`), for
//! `scripts/extra_mutants.py` to apply and test (worklog 5.0b).

mod scan;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[expect(clippy::print_stderr, reason = "a dev tool's errors go to stderr")]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 || args[0] != "list" {
        eprintln!("usage: extra_mutants list <crate dir>");
        return ExitCode::from(2);
    }
    match list(Path::new(&args[1])) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[expect(
    clippy::print_stdout,
    reason = "the lister's only output is the mutant list"
)]
fn list(crate_dir: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    walk(&crate_dir.join("src"), &mut files)
        .map_err(|error| format!("{}: {error}", crate_dir.join("src").display()))?;
    files.sort();
    for path in files {
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let mutants =
            scan::scan_source(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        let rel = relative(&path, crate_dir);
        for mutant in &mutants {
            println!(
                "{rel}\t{}\t{}\t{}\t{}\t{}\t{}",
                mutant.start,
                mutant.end,
                mutant.kind,
                mutant.line,
                mutant.replacement,
                mutant.description
            );
        }
    }
    Ok(())
}

/// Every `.rs` file under `dir`, recursively.
fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            walk(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

/// `path` relative to `crate_dir`, forward slashes for the tab-separated output.
fn relative(path: &Path, crate_dir: &Path) -> String {
    path.strip_prefix(crate_dir)
        .unwrap_or(path)
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
