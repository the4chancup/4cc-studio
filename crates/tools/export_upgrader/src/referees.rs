//! Step 9's referee export (`plans/export_upgrader.md` step 9, `DECISIONS.md`
//! 2026-10-18 "steps 9 and 12's gaps…"): the roster, `refs.txt` rewritten as
//! the root `players.txt` in the canonical grammar, and the root files; and
//! the referee folders under `Players/`, copied as they are (the flatten of
//! their `face/`, `boots/`, `gloves/` and `common/` subfolders is step 12's).

use crate::carried::under;
use crate::players::PlayerFolders;

/// The root `refs.txt` of `files` (directly at the root, any letter case), if any.
pub(crate) fn find_refs_txt(files: &[String]) -> Option<&str> {
    files
        .iter()
        .find(|file| file.eq_ignore_ascii_case("refs.txt"))
        .map(String::as_str)
}

/// The root `players.txt` text step 9 writes for `refs_txt`'s bytes: a UTF-8
/// BOM dropped, LF line ends, blank lines left out, every other line
/// trimmed; a line whose head (up to the first whitespace) is ASCII digits
/// making a `u16` with something after it is written as the slot in two
/// digits, one space and the trimmed rest (`1 germanbro` → `01 germanbro`,
/// `2\tdonte` → `02 donte`); any other line as trimmed, for the shared
/// validation to report on the output. `None` when the bytes are not UTF-8.
pub(crate) fn players_txt(refs_txt: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(refs_txt).ok()?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut out = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // The head and rest as the roster reader splits them, so a line
        // written canonical here reads back as the same entry.
        let slot = line
            .split_once(char::is_whitespace)
            .filter(|(head, _)| head.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|(head, rest)| Some((head.parse::<u16>().ok()?, rest.trim())));
        match slot {
            Some((slot, rest)) => out.push_str(&format!("{slot:02} {rest}")),
            None => out.push_str(line),
        }
        out.push('\n');
    }
    Some(out)
}

/// Step 9's root files and referee folders: the root `refs.txt` consumed
/// (not copied: `run` writes `players.txt` from it), a root `ref_marker.dds`
/// (any letter case) copied under its own name, and every file below a
/// direct subfolder of `Players/` (any letter case) copied to
/// `Players/<folder>/<rest>`, as it is. A file directly in `Players/` is
/// nobody's: the fallback reports it. `portraits` and `folders` stay empty:
/// a referee has no save record.
pub(crate) fn referee_folders(files: &[String]) -> PlayerFolders {
    let mut plan = PlayerFolders::default();
    for file in files {
        if file.eq_ignore_ascii_case("refs.txt") {
            plan.consumed.insert(file.clone());
        } else if file.eq_ignore_ascii_case("ref_marker.dds") {
            plan.copies.push((file.clone(), file.clone()));
            plan.consumed.insert(file.clone());
        } else if let Some(rest) = under(file, "Players")
            && rest.contains('/')
        {
            plan.copies.push((file.clone(), format!("Players/{rest}")));
            plan.consumed.insert(file.clone());
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn players_txt_writes_each_parsed_line_canonical() {
        assert_eq!(
            players_txt(b"\xEF\xBB\xBF1 germanbro\r\n2\tdonte\r\n\r\n 07 x \n").as_deref(),
            Some("01 germanbro\n02 donte\n07 x\n")
        );
    }

    #[test]
    fn players_txt_keeps_a_line_that_does_not_parse_as_written() {
        assert_eq!(
            players_txt(b"foo bar\n03\n135 x\n").as_deref(),
            Some("foo bar\n03\n135 x\n")
        );
    }

    #[test]
    fn players_txt_of_bytes_that_are_not_utf8_is_none() {
        assert_eq!(players_txt(b"\xFF\xFE"), None);
    }

    #[test]
    fn referee_folders_copies_the_marker_and_the_folders_and_consumes_the_roster() {
        let files: Vec<String> = [
            "players/arbiter/boots.fmdl",
            "Players/readme.txt",
            "Refs.txt",
            "REF_MARKER.DDS",
        ]
        .map(str::to_owned)
        .to_vec();
        assert_eq!(find_refs_txt(&files), Some("Refs.txt"));
        let plan = referee_folders(&files);
        assert_eq!(
            plan.copies,
            vec![
                (
                    "players/arbiter/boots.fmdl".to_owned(),
                    "Players/arbiter/boots.fmdl".to_owned()
                ),
                ("REF_MARKER.DDS".to_owned(), "REF_MARKER.DDS".to_owned()),
            ]
        );
        assert_eq!(
            plan.consumed.iter().map(String::as_str).collect::<Vec<_>>(),
            ["REF_MARKER.DDS", "Refs.txt", "players/arbiter/boots.fmdl"]
        );
        assert!(plan.findings.is_empty());
    }
}
