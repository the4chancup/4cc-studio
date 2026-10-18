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

/// One color entry of a Note section (step 5): the line as written, trimmed,
/// and its colors, what follows the line's last `:`.
pub(crate) struct NoteEntry {
    /// The line as written, trimmed.
    pub(crate) line: String,
    /// What follows the line's last `:`, trimmed.
    pub(crate) colors: String,
}

/// Which color section of the Note the reading is inside (step 5's grammar):
/// the kit entries are read their own way (6.3), the team ones here.
enum Section {
    /// Before `Team Colours:` (or `Kit Colours:`): entries belong to none.
    None,
    /// The team section: `Team Colours:`'s entries.
    Team,
    /// The kit section: `Kit Colours:`'s entries.
    Kit,
}

/// The team section's entries, in Note order.
///
/// The reading (step 5): blank lines skipped; a line's words split on
/// whitespace; a line whose first word is `team` and whose second starts
/// with `co`, both in any letter case, opens the team section; one whose
/// first word is `kit` and second starts with `co` opens the kit section,
/// closing the team one; a line starting, as written, with `player`, `other`
/// or `notes` in any letter case ends the reading; inside the team section
/// an entry is a line whose first non-blank character is `-`. A `-` line
/// before any header belongs to no section.
pub(crate) fn team_entries(note_text: &str) -> Vec<NoteEntry> {
    let mut section = Section::None;
    let mut entries = Vec::new();
    for line in note_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lowered = line.to_ascii_lowercase();
        if ["player", "other", "notes"]
            .iter()
            .any(|word| lowered.starts_with(word))
        {
            break;
        }
        let mut words = trimmed.split_whitespace();
        let (first, second) = (words.next().unwrap_or(""), words.next().unwrap_or(""));
        if second.to_ascii_lowercase().starts_with("co") {
            if first.eq_ignore_ascii_case("team") {
                section = Section::Team;
                continue;
            }
            if first.eq_ignore_ascii_case("kit") {
                section = Section::Kit;
                continue;
            }
        }
        if matches!(section, Section::Team) && trimmed.starts_with('-') {
            let colors = line
                .rsplit_once(':')
                .map(|(_, colors)| colors.trim())
                .unwrap_or("");
            entries.push(NoteEntry {
                line: trimmed.to_owned(),
                colors: colors.to_owned(),
            });
        }
    }
    entries
}

/// The "Other Notes" section's text: every line after the first line starting
/// with `Other Notes:` in any letter case, joined by `\n`, leading and
/// trailing blank lines trimmed; `None` when nothing but whitespace remains.
pub(crate) fn other_notes(note_text: &str) -> Option<String> {
    let after = note_text
        .lines()
        .skip_while(|line| !line.to_ascii_lowercase().starts_with("other notes:"))
        .skip(1);
    let lines: Vec<&str> = after.collect();
    let (start, end) = (
        lines.iter().position(|line| !line.trim().is_empty()),
        lines.iter().rposition(|line| !line.trim().is_empty()),
    );
    match (start, end) {
        (Some(start), Some(end)) => Some(lines[start..=end].join("\n")),
        _ => None,
    }
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

    /// The real `/4ccg/` Note (the `co Spring` fixture, its `Team:` line
    /// repointed at `/co/`): four team colors, three player kits and a
    /// goalkeeper, and a two-line "Other Notes:" section.
    const NOTE: &str = include_str!("../tests/fixtures/co Spring/co Note.txt");

    #[test]
    fn the_team_entries_are_the_team_sections_dash_lines_in_order() {
        let entries = team_entries(NOTE);
        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.line.as_str(), entry.colors.as_str()))
                .collect::<Vec<_>>(),
            [
                ("- 1st: 048 146 082", "048 146 082"),
                ("- 2nd: 221 000 000", "221 000 000"),
                ("- 3rd: 000 000 000", "000 000 000"),
                ("- 4th: 000 000 000", "000 000 000"),
            ]
        );
    }

    #[test]
    fn other_notes_is_the_text_after_the_header() {
        let notes = other_notes(NOTE).expect("the 4ccg Note holds Other Notes");
        assert_eq!(notes.lines().count(), 2);
        assert!(notes.starts_with("The Cracks Are Forming"));
        assert!(notes.ends_with("import them."), "{notes:?}");
    }

    #[test]
    fn a_header_in_another_letter_case_opens_the_section() {
        let entries = team_entries("TEAM COLOURS:\r\n- 1st: 1 2 3\r\n");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].colors, "1 2 3");
    }

    #[test]
    fn a_kit_header_closes_the_team_section() {
        // The kit entries are not team entries (6.3 reads them its own way).
        let entries = team_entries(
            "Team Colours:\n- 1st: 1 2 3\nKit Colours:\n- 1st player: 4 5 6\n- 1st GK: 7 8 9",
        );
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].colors, "1 2 3");
    }

    #[test]
    fn the_stop_words_end_the_reading() {
        for stop in ["Player IDs:", "Other Notes:", "Notes:"] {
            let text = format!("Team Colours:\n- 1st: 1 2 3\n{stop} x\n- 2nd: 4 5 6");
            assert_eq!(team_entries(&text).len(), 1, "{stop}");
        }
    }

    #[test]
    fn a_dash_line_before_the_header_belongs_to_no_section_and_a_bad_entry_is_returned() {
        let entries = team_entries("- early: 9 9 9\nTeam Colours:\n- 1st: blue\n");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].line, "- 1st: blue");
        assert_eq!(entries[0].colors, "blue");
    }

    #[test]
    fn a_whitespace_only_or_absent_other_notes_is_none() {
        assert_eq!(other_notes("x\nOther Notes:\n   \n\t\n"), None);
        assert_eq!(other_notes("Team: /co/\n"), None);
    }

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
