//! The Team Note (`plans/export_upgrader.md` step 6): the `.txt` at the export
//! root whose name holds `note`, and its `Team:` line, which names the team
//! the export belongs to.

/// Whether `name` (a root file's) is a Note: ends `.txt` and holds `note`,
/// both in any letter case.
pub(crate) fn is_note_name(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    lowered.ends_with(".txt") && lowered.contains("note")
}

/// The export's Note: of the root files that are one, the first in byte order
/// of the name; `None` when the export holds none. `files` is the export's
/// root-relative file list.
pub(crate) fn find(files: &[String]) -> Option<&str> {
    files
        .iter()
        .map(String::as_str)
        .filter(|path| !path.contains('/'))
        .filter(|name| is_note_name(name))
        .min()
}

/// The Note's text: read as UTF-8 (step 5's reading), a byte sequence that is
/// not UTF-8 reading as the replacement character — a Shift-JIS Note must not
/// fail.
pub(crate) fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The `Team:` line's value: the first line holding `Team:` in that letter
/// case, what follows it trimmed. An empty value counts as no line.
pub(crate) fn team_line(note_text: &str) -> Option<String> {
    for line in note_text.lines() {
        if let Some((_, value)) = line.split_once("Team:") {
            let value = value.trim();
            return (!value.is_empty()).then(|| value.to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_note_is_the_first_txt_holding_note_in_byte_order() {
        // Byte order puts "Team Notes.txt" ('T' = 0x54) before "co Note.txt"
        // ('c' = 0x63).
        let files = vec![
            "Other/note.txt".to_owned(), // not at the root
            "co Note.txt".to_owned(),
            "Team Notes.txt".to_owned(),
            "readme.txt".to_owned(),
        ];
        assert_eq!(find(&files), Some("Team Notes.txt"));
        // `.txt` and `note` in any letter case; a name missing either is out.
        let files = vec!["NOTES.TXT".to_owned(), "note.png".to_owned()];
        assert_eq!(find(&files), Some("NOTES.TXT"));
        let files = vec!["readme.txt".to_owned()];
        assert_eq!(find(&files), None);
    }

    #[test]
    fn the_team_line_is_the_first_holding_team_in_that_case() {
        assert_eq!(
            team_line("Team: /co/\n\nTeam Colours:\n- 1st: 048 146 082"),
            Some("/co/".to_owned())
        );
        // `team:` in another case is not the line; an empty value counts as
        // none.
        assert_eq!(team_line("team: /co/"), None);
        assert_eq!(team_line("Team:\nTeam Colours:"), None);
        // CRLF lines end before the value's trim.
        assert_eq!(team_line("Team: /co/\r\nNext"), Some("/co/".to_owned()));
        assert_eq!(team_line(""), None);
    }
}
