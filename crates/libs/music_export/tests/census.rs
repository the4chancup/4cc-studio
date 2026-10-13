//! The census over every `.4ccm` on the writing machine, listed by
//! `scripts/provenance/music_export/list_4ccm.py` into
//! `.tmp/music_export/all_4ccm.txt` (one absolute path per line). Skipped when
//! the list is absent. Each file is parsed, written and parsed again; the tally
//! lands in `.tmp/music_export/census.txt`.

use std::collections::BTreeMap;
use std::path::Path;

use music_export::{ParseErrorKind, parse, write};

const LIST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../.tmp/music_export/all_4ccm.txt"
);
const REPORT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../.tmp/music_export/census.txt"
);

#[test]
fn every_export_on_this_machine_parses_and_reads_back_equal() {
    let Ok(listing) = std::fs::read_to_string(LIST) else {
        return;
    };
    let mut parsed_count = 0;
    let mut identical = 0;
    let mut semantic = 0;
    let mut warnings: BTreeMap<String, usize> = BTreeMap::new();
    let mut refusals: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut mismatches = Vec::new();
    for path in listing
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let bytes = std::fs::read(path).expect("a listed file reads");
        let stem = Path::new(path)
            .file_stem()
            .expect("a file name")
            .to_str()
            .expect("a UTF-8 file name");
        let parsed = match parse(&bytes, stem) {
            Ok(parsed) => parsed,
            Err(error) => {
                refusals
                    .entry(error.kind.to_string())
                    .or_default()
                    .push(format!("{path} (line {})", error.line));
                // The only refusals the census recorded: keywords no reader
                // accepts (`random`, `randomize`, `goals>=3` without spaces).
                assert!(
                    matches!(error.kind, ParseErrorKind::UnknownKeyword(_)),
                    "{path}: {error}"
                );
                continue;
            }
        };
        parsed_count += 1;
        for warning in &parsed.warnings {
            *warnings.entry(format!("{warning:?}")).or_default() += 1;
        }
        let written = write(&parsed.export);
        let again = parse(written.as_bytes(), stem).expect("the written form parses");
        if again.export == parsed.export {
            // The writer emits LF; the files are CRLF, so compare line by line.
            let original = String::from_utf8_lossy(&bytes);
            if original.lines().eq(written.lines()) {
                identical += 1;
            } else {
                semantic += 1;
            }
        } else {
            mismatches.push(path.to_string());
        }
    }
    let refused: usize = refusals.values().map(Vec::len).sum();
    let mut report = format!(
        "{} files: {parsed_count} parsed ({identical} identical line for line, {semantic} \
         semantically equal, {} model mismatches), {refused} refused\n",
        parsed_count + refused,
        mismatches.len()
    );
    for (warning, count) in &warnings {
        report.push_str(&format!("warning {warning}: {count}\n"));
    }
    for (kind, paths) in &refusals {
        report.push_str(&format!("refused ({kind}): {}\n", paths.len()));
        for path in paths {
            report.push_str(&format!("    {path}\n"));
        }
    }
    for path in &mismatches {
        report.push_str(&format!("mismatch {path}\n"));
    }
    std::fs::write(REPORT, &report).expect("the report writes");
    assert!(mismatches.is_empty(), "{report}");
}
