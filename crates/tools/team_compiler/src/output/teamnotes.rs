//! `teamnotes.txt` (`team_compiler/pipeline.md` "2. Per-export serial steps", item 5 "Notes
//! collection"): the root `notes.txt` of every export a compile keeps, gathered into one file
//! in the output folder for the member compiling the cup.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::Context;

use crate::paths::replace_file;

/// The collected notes' file name in the output folder.
pub(crate) const FILE_NAME: &str = "teamnotes.txt";

/// The text of `teamnotes.txt` for `notes` (team name as `/co/`, the note's text), in their
/// order; `None` when there is no note. Each entry is a header line `--- /co/ ---`, then the
/// note with its line ends made LF and its leading blank lines and trailing whitespace
/// removed; one empty line separates two entries, and the text ends with one LF.
pub(crate) fn render(notes: &[(String, String)]) -> Option<String> {
    if notes.is_empty() {
        return None;
    }
    let entries: Vec<String> = notes
        .iter()
        .map(|(team, note)| format!("--- {team} ---\n{}\n", note_body(note)))
        .collect();
    Some(entries.join("\n"))
}

/// `note` as an entry holds it: CRLF and lone CR made LF, the blank lines before its first
/// line of text and the whitespace after its last removed. The first line's indentation stays.
fn note_body(note: &str) -> String {
    let lf = note.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = lf
        .split('\n')
        .skip_while(|line| line.trim().is_empty())
        .collect();
    lines.join("\n").trim_end().to_owned()
}

/// Puts `text` in place at `path`, through a temporary file beside it renamed onto it, so
/// `path` holds the previous file or the whole new one; with no text, removes a previous
/// file (a missing one is fine). Each failure names the file it concerns.
pub(crate) fn write(path: &Path, text: Option<&str>) -> anyhow::Result<()> {
    let Some(text) = text else {
        return match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
            Err(error) => {
                Err(error).with_context(|| format!("{}: cannot remove it", path.display()))
            }
        };
    };
    replace_file(path, text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::scratch;

    /// `notes` as `render` takes them.
    fn entries(notes: &[(&str, &str)]) -> Vec<(String, String)> {
        notes
            .iter()
            .map(|(team, note)| ((*team).to_owned(), (*note).to_owned()))
            .collect()
    }

    #[test]
    fn each_note_is_under_its_team_s_header_its_line_ends_lf_and_its_blank_edges_removed() {
        let notes = entries(&[
            ("/a/", "\r\n\r\nFirst line\r\nsecond line\r\n\r\n"),
            ("/co/", "Hello"),
        ]);
        assert_eq!(
            render(&notes).as_deref(),
            Some("--- /a/ ---\nFirst line\nsecond line\n\n--- /co/ ---\nHello\n")
        );
    }

    #[test]
    fn a_lone_cr_ends_a_line_and_an_indented_first_line_keeps_its_indentation() {
        assert_eq!(
            render(&entries(&[("/co/", "one\rtwo\r")])).as_deref(),
            Some("--- /co/ ---\none\ntwo\n")
        );
        assert_eq!(
            render(&entries(&[("/co/", " \n\n  indented\n  kept  \n")])).as_deref(),
            Some("--- /co/ ---\n  indented\n  kept\n")
        );
    }

    #[test]
    fn no_note_renders_no_file() {
        assert_eq!(render(&[]), None);
    }

    /// The names of the files in `folder`, sorted.
    fn names(folder: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn the_text_replaces_a_previous_file_and_leaves_nothing_else_in_the_folder() {
        let temp = scratch("teamnotes_write");
        let path = temp.path().join(FILE_NAME);

        write(&path, Some("first\n")).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "first\n");

        write(&path, Some("second\n")).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second\n");
        assert_eq!(names(temp.path()), [FILE_NAME]);
    }

    #[test]
    fn no_text_removes_a_previous_file_and_is_fine_without_one() {
        let temp = scratch("teamnotes_remove");
        let path = temp.path().join(FILE_NAME);
        fs::write(&path, "previous\n").unwrap();

        write(&path, None).unwrap();
        assert!(!path.exists());

        write(&path, None).unwrap();
        assert_eq!(names(temp.path()), Vec::<String>::new());
    }

    #[test]
    fn a_path_that_cannot_be_written_is_an_error_naming_it() {
        let temp = scratch("teamnotes_blocked");
        let path = temp.path().join(FILE_NAME);
        // A folder in the file's place, with a file in it so no platform replaces it.
        fs::create_dir_all(path.join("kept")).unwrap();

        for text in [Some("note\n"), None] {
            let error = write(&path, text).unwrap_err();
            let chain = format!("{error:#}");
            assert!(
                chain.starts_with(&format!("{}: ", path.display())),
                "{text:?}: {chain}"
            );
            assert!(path.join("kept").is_dir(), "{text:?}");
            assert_eq!(names(temp.path()), [FILE_NAME], "{text:?}");
        }
    }
}
