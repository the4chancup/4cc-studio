//! `4cc-studio help`, run through the real binary: the Markdown a chapter, the `--export`
//! files and a refusal's `error: ` line. The help reads no settings, so the binary runs as
//! built, in place.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn studio(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_4cc-studio"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The Team compiler's last catalog row (`notes_encoding_invalid`).
const NOTES_ENCODING_INVALID: &str = "| `notes_encoding_invalid` | Error | The root `notes.txt` is not valid UTF-8. | The note is left out; the export otherwise compiles. |";
/// The Export upgrader's last catalog row: the same code, the upgrader's own effect.
const UPGRADER_NOTES_ENCODING_INVALID: &str = "| `notes_encoding_invalid` | Error | The root `notes.txt` is not valid UTF-8. | The run is not clean, so nothing is written. |";

// ST-HELP-04
#[test]
fn a_chapter_prints_its_markdown_ending_with_the_messages_table() {
    let output = studio(&["help", "team-compiler"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let stdout = stdout(&output);
    assert!(stdout.starts_with("# Team compiler\n"), "{stdout}");
    assert!(stdout.contains("## Running from a terminal"), "{stdout}");
    assert!(
        stdout.contains(
            "| `export_extract_failed` | Error | The archive (`.zip` or `.7z`) cannot be opened, or a path inside it is unusable: it leads outside the archive, or two names differ only in letter case or Unicode form. | The export is not compiled. Repack the archive, or unpack it into a folder. |"
        ),
        "{stdout}"
    );
    assert_eq!(
        stdout.lines().last(),
        Some(NOTES_ENCODING_INVALID),
        "{stdout}"
    );
}

// ST-HELP-04
#[test]
fn the_export_writes_one_markdown_file_per_chapter() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("help_cli_export");
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    let output = studio(&["help", "--export", dir.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(output.stdout.is_empty(), "the export prints nothing");

    let mut files: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    files.sort();
    assert_eq!(
        files,
        [
            "export-upgrader.md",
            "first-run.md",
            "save-editor.md",
            "setting-up.md",
            "team-compiler.md"
        ]
    );

    // Each file is what `help <id>` prints; each tool's ends with its last catalog row.
    let last_rows = [
        ("team-compiler", NOTES_ENCODING_INVALID),
        (
            "save-editor",
            "| `fpc_custom_skin_on_non_fpc` | Warning | A custom skin on a player whose body is not hidden by the FPC hide boots ID from Settings hides the body but not the jersey. | Nothing is changed for you; hide the body too, or pick a preset skin. |",
        ),
        ("export-upgrader", UPGRADER_NOTES_ENCODING_INVALID),
    ];
    for (id, last_row) in last_rows {
        let printed = studio(&["help", id]);
        assert_eq!(printed.status.code(), Some(0), "{id}: {}", stderr(&printed));
        let file = fs::read_to_string(dir.join(format!("{id}.md"))).unwrap();
        assert_eq!(file, stdout(&printed), "{id}.md differs from `help {id}`");
        assert_eq!(file.lines().last(), Some(last_row), "{id}");
    }
    // The core chapters end with their last topic, having no catalog.
    for id in ["setting-up", "first-run"] {
        let printed = studio(&["help", id]);
        assert_eq!(printed.status.code(), Some(0), "{id}: {}", stderr(&printed));
        assert_eq!(
            fs::read_to_string(dir.join(format!("{id}.md"))).unwrap(),
            stdout(&printed),
            "{id}.md differs from `help {id}`"
        );
        assert!(!stdout(&printed).contains("## Messages"), "{id}");
    }
}

#[test]
fn an_unknown_chapter_is_refused_with_exit_2() {
    let output = studio(&["help", "nope"]);
    assert_eq!(output.status.code(), Some(2), "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).starts_with("error: no help chapter"),
        "{}",
        stderr(&output)
    );
}
