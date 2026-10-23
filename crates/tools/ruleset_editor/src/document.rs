//! One rules file being edited: the egui-free document the view and the CLI
//! share. `Document::open` runs `Rules::load` (validation steps 1 to 3,
//! `spec/aatf_rules.md` "Loading and validation"), then reads the logic
//! section's identity; `save` writes the file only through the same
//! validation, so an invalid ruleset is never written.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use aatf::{LogicIdentity, Rules, Ruleset, assemble_file, file_sections};

/// One rules file being edited: its ruleset, its logic and its custom checks
/// as read.
#[derive(Debug)]
pub(crate) struct Document {
    /// Where it was read from or last saved; `None` for one never saved.
    pub(crate) path: Option<PathBuf>,
    /// What the forms edit.
    pub(crate) ruleset: Ruleset,
    /// The file's logic section.
    pub(crate) logic: Logic,
    /// The CUSTOM CHECKS section as read, written back byte for byte but its
    /// trailing whitespace (the file's layout); `None`: no checks.
    pub(crate) custom_checks: Option<String>,
}

/// The file's logic section, against the logic this build ships.
#[derive(Debug)]
pub(crate) enum Logic {
    /// The current logic, written fresh on every save.
    Current,
    /// An older official version, kept as read until `update_logic`.
    Older {
        /// The version the file's section hashes to.
        version: u32,
        /// The section as read, banner included, written back byte for byte
        /// but its trailing whitespace.
        section: String,
    },
    /// Not Studio's: kept as read on every save.
    Modified {
        /// The section as read, banner included, written back byte for byte
        /// but its trailing whitespace; empty when the file carries no LOGIC banner at all, and such a
        /// file opens but cannot be saved: the rewrite would hold no logic.
        section: String,
    },
}

/// Why a rules file did not open.
#[derive(Debug, thiserror::Error)]
pub(crate) enum OpenError {
    /// The file could not be read.
    #[error("{}: cannot read the file: {error}", .path.display())]
    Read {
        /// The file.
        path: PathBuf,
        /// The io error.
        error: io::Error,
    },
    /// It read but failed load-time validation
    /// (`spec/aatf_rules.md` "Loading and validation").
    #[error("{0}")]
    Invalid(#[from] aatf::LoadError),
}

/// Why a rules file was not written.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SaveError {
    /// The file could not be written.
    #[error("{}: cannot write the file: {error}", .path.display())]
    Write {
        /// The file.
        path: PathBuf,
        /// The io error.
        error: io::Error,
    },
    /// The assembled file failed the same validation `open` runs; nothing is
    /// written.
    #[error("{0}")]
    Invalid(#[from] aatf::LoadError),
}

impl Document {
    /// Reads and validates `path` (`Rules::load`: steps 1 to 3); a file that
    /// fails is not opened.
    pub(crate) fn open(path: &Path) -> Result<Document, OpenError> {
        let text = fs::read_to_string(path).map_err(|error| OpenError::Read {
            path: path.to_path_buf(),
            error,
        })?;
        let rules = Rules::load(&text).map_err(OpenError::Invalid)?;
        let sections = file_sections(&text);
        let logic = match rules.logic() {
            LogicIdentity::Current => Logic::Current,
            LogicIdentity::Official { version } => Logic::Older {
                version,
                section: sections.logic.unwrap_or_default(),
            },
            LogicIdentity::Modified => Logic::Modified {
                section: sections.logic.unwrap_or_default(),
            },
        };
        Ok(Document {
            path: Some(path.to_path_buf()),
            ruleset: rules.ruleset().clone(),
            logic,
            custom_checks: sections.custom_checks,
        })
    }

    /// Gives an older official logic the current one; `false` (nothing
    /// changed) for a current or modified one.
    pub(crate) fn update_logic(&mut self) -> bool {
        match self.logic {
            Logic::Older { .. } => {
                self.logic = Logic::Current;
                true
            }
            Logic::Current | Logic::Modified { .. } => false,
        }
    }

    /// Assembles the file (`aatf::assemble_file`), validates it as `open`
    /// does, and writes it to a temporary sibling renamed over `path`;
    /// nothing is written when validation fails.
    pub(crate) fn save(&mut self, path: &Path) -> Result<(), SaveError> {
        let section = match &self.logic {
            Logic::Current => None,
            Logic::Older { section, .. } | Logic::Modified { section } => Some(section.as_str()),
        };
        let text = assemble_file(&self.ruleset, section, self.custom_checks.as_deref());
        Rules::load(&text).map_err(SaveError::Invalid)?;
        write_text(path, &text).map_err(|error| SaveError::Write {
            path: path.to_path_buf(),
            error,
        })?;
        self.path = Some(path.to_path_buf());
        Ok(())
    }
}

/// `text` written to `path` through a temporary sibling (`<path>.tmp`)
/// renamed over it; on failure the temporary is removed and the error
/// returned, `path` as it was. A read-only `path` is refused before anything
/// is written (`PermissionDenied`).
pub(crate) fn write_text(path: &Path, text: &str) -> io::Result<()> {
    // A rename replaces a read-only file on Linux, where the folder's
    // permission is what counts, while Windows refuses it: checking first
    // keeps one rule on both (RE-FILE-07).
    if fs::metadata(path).is_ok_and(|metadata| metadata.permissions().readonly()) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the file is read-only",
        ));
    }
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let write_and_swap = || -> io::Result<()> {
        fs::write(&tmp, text)?;
        fs::rename(&tmp, path)
    };
    if let Err(error) = write_and_swap() {
        // Best effort: the original error is the one to report.
        if let Err(cleanup) = fs::remove_file(&tmp) {
            log::debug!("could not remove {}: {cleanup}", tmp.display());
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::scratch;

    /// The embedded official ruleset's text as an assembled file.
    fn official_text() -> String {
        assemble_file(Rules::official().ruleset(), None, None)
    }

    /// The same text with one logic comment edited: the file still loads,
    /// its logic is `Modified`.
    fn modified_text() -> String {
        official_text().replacen(
            "// The generic interpreter",
            "// The interpreter, its comment edited",
            1,
        )
    }

    /// `text` written to `dir/name`; the path.
    fn write_file(dir: &Path, name: &str, text: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, text).unwrap();
        path
    }

    /// Open the assembled official file: the current logic, no checks, an
    /// equal ruleset, nothing dirty.
    #[test]
    fn open_reads_the_ruleset_and_a_current_logic() {
        let dir = scratch("open_current");
        let path = write_file(&dir, "rules.rhai", &official_text());
        let doc = Document::open(&path).expect("opens");
        assert!(matches!(doc.logic, Logic::Current));
        assert_eq!(doc.custom_checks, None);
        assert_eq!(&doc.ruleset, Rules::official().ruleset());
    }

    /// Open a file whose logic comment was edited: `Modified`, its section
    /// kept as read.
    #[test]
    fn open_marks_a_hand_edited_logic_as_modified() {
        let dir = scratch("open_modified");
        let text = modified_text();
        let expected = file_sections(&text).logic.unwrap();
        let path = write_file(&dir, "rules.rhai", &text);
        let doc = Document::open(&path).expect("opens");
        match &doc.logic {
            Logic::Modified { section } => {
                assert_eq!(section.trim_end(), expected.trim_end())
            }
            other => panic!("expected Modified, got {other:?}"),
        }
    }

    /// A file that fails to load is not opened: `Invalid` carrying the
    /// `LoadError`.
    #[test]
    fn a_file_that_fails_to_load_is_not_opened() {
        let dir = scratch("open_broken");
        let path = write_file(&dir, "rules.rhai", "const RULESET = #{ name: \"x\",\n");
        match Document::open(&path) {
            Err(OpenError::Invalid(aatf::LoadError::Parse { line, .. })) => {
                assert!(line >= 1)
            }
            other => panic!("expected Invalid(Parse), got {other:?}"),
        }
        let missing = dir.join("missing.rhai");
        assert!(matches!(
            Document::open(&missing),
            Err(OpenError::Read { .. })
        ));
    }

    /// A file whose logic has no LOGIC banner opens as `Modified` with an
    /// empty section; saving it is refused (`MissingFunction`: the rewrite
    /// would hold no logic), the file unchanged.
    #[test]
    fn a_bannerless_file_opens_but_cannot_be_saved() {
        let dir = scratch("bannerless");
        let text = official_text().replacen(
            "// ==== LOGIC (ruleset-logic 1): the Ruleset editor replaces this section ====\n",
            "",
            1,
        );
        assert!(!text.contains("// ==== LOGIC"), "premise: no banner");
        let path = write_file(&dir, "rules.rhai", &text);
        let mut doc = Document::open(&path).expect("opens");
        match &doc.logic {
            Logic::Modified { section } => assert!(section.is_empty()),
            other => panic!("expected Modified, got {other:?}"),
        }
        let before = fs::read(&path).unwrap();
        match doc.save(&path) {
            Err(SaveError::Invalid(aatf::LoadError::MissingFunction { .. })) => {}
            other => panic!("expected Invalid(MissingFunction), got {other:?}"),
        }
        assert_eq!(fs::read(&path).unwrap(), before, "the file is unchanged");
    }

    /// Save writes a file that reopens with an equal ruleset, the same logic
    /// status and the same checks.
    #[test]
    fn save_writes_a_file_that_reopens_equal() {
        let dir = scratch("save_round_trip");
        let path = write_file(&dir, "rules.rhai", &official_text());
        let mut doc = Document::open(&path).expect("opens");
        doc.ruleset.tiers[0].rate = 98;
        let out = dir.join("edited.rhai");
        doc.save(&out).expect("saves");
        let reopened = Document::open(&out).expect("reopens");
        assert_eq!(reopened.ruleset, doc.ruleset);
        assert!(matches!(reopened.logic, Logic::Current));
        assert_eq!(reopened.custom_checks, doc.custom_checks);
    }

    /// A modified logic section and VTL11's custom checks are written back
    /// byte for byte (trailing whitespace aside).
    #[test]
    fn a_modified_logic_and_custom_checks_are_kept_byte_for_byte() {
        let aatf_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/aatf");
        let block = fs::read_to_string(aatf_dir.join("rulesets/vtl11.rhai")).unwrap();
        let ruleset = Ruleset::from_data_block(&block).unwrap();
        let checks = file_sections(&block)
            .custom_checks
            .expect("VTL11 has checks");
        let current = file_sections(&official_text()).logic.unwrap();
        let modified = current.replacen(
            "// The generic interpreter",
            "// The interpreter, its comment edited",
            1,
        );
        let dir = scratch("save_keeps_sections");
        let path = write_file(
            &dir,
            "rules.rhai",
            &assemble_file(&ruleset, Some(&modified), Some(&checks)),
        );
        let mut doc = Document::open(&path).expect("opens");
        assert!(matches!(doc.logic, Logic::Modified { .. }));
        doc.save(&path).expect("saves");
        let written = file_sections(&fs::read_to_string(&path).unwrap());
        assert_eq!(
            written.logic.as_deref().map(str::trim_end),
            Some(modified.trim_end()),
            "the file's own logic, byte for byte"
        );
        assert_eq!(
            written.custom_checks.as_deref().map(str::trim_end),
            Some(checks.trim_end()),
            "the custom checks, byte for byte"
        );
        let reopened = Document::open(&path).expect("reopens");
        assert!(matches!(reopened.logic, Logic::Modified { .. }));
    }

    /// Checks written inside the logic section (no CUSTOM CHECKS banner)
    /// load and `open` marks the logic `Modified`, but `save` refuses: the
    /// stub the assembly appends is a second `custom_checks`, which Rhai
    /// refuses to parse, so the member's checks are never shadowed by a
    /// file that loads. The pin guards that premise.
    #[test]
    fn checks_inside_the_logic_are_refused_on_save() {
        let dir = scratch("checks_in_logic");
        let stub = "// ==== CUSTOM CHECKS: optional, hand-written; the Ruleset editor keeps this section as it is ====\nfn custom_checks(team) { [] }";
        let text = official_text();
        assert!(text.contains(stub), "premise: the stub is appended");
        // VTL11's goalkeeper count, after the logic with no banner.
        let check = "fn custom_checks(team) {
    let found = [];
    let goalkeepers = 0;
    for player in team.players {
        if player.position == \"GK\" { goalkeepers += 1; }
    }
    if goalkeepers > 2 {
        found.push(#{ kind: \"Custom\", text: \"More than two goalkeepers\" });
    }
    found
}";
        let path = write_file(&dir, "rules.rhai", &text.replacen(stub, check, 1));
        let mut doc = Document::open(&path).expect("opens");
        assert!(
            matches!(doc.logic, Logic::Modified { .. }),
            "the file's logic is not this build's"
        );
        assert_eq!(doc.custom_checks, None, "no custom-checks section read");
        let before = fs::read(&path).unwrap();
        match doc.save(&path) {
            Err(SaveError::Invalid(aatf::LoadError::Parse { message, .. }))
                if message.contains("custom_checks") => {}
            other => panic!("expected the duplicate custom_checks refused, got {other:?}"),
        }
        assert_eq!(fs::read(&path).unwrap(), before, "the file is unchanged");
    }

    /// An invalid ruleset (two tiers sharing an id) is never written: the
    /// error is `Invalid` and the existing file is unchanged.
    #[test]
    fn an_invalid_ruleset_is_never_written() {
        let dir = scratch("save_refused");
        let path = write_file(&dir, "rules.rhai", &official_text());
        let before = fs::read(&path).unwrap();
        let mut doc = Document::open(&path).expect("opens");
        doc.ruleset.tiers[1].id = doc.ruleset.tiers[0].id.clone();
        match doc.save(&path) {
            Err(SaveError::Invalid(aatf::LoadError::Rules(_))) => {}
            other => panic!("expected Invalid(Rules), got {other:?}"),
        }
        assert_eq!(fs::read(&path).unwrap(), before, "the file is unchanged");
    }

    /// After a save the temporary sibling is renamed into place, not left
    /// beside the file.
    #[test]
    fn a_save_leaves_no_temporary_sibling() {
        let dir = scratch("save_no_tmp");
        let path = write_file(&dir, "rules.rhai", &official_text());
        let mut doc = Document::open(&path).expect("opens");
        doc.ruleset.name = "Renamed".to_string();
        doc.save(&path).expect("saves");
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        assert!(
            !Path::new(&tmp).exists(),
            "the temporary sibling is renamed, not left"
        );
    }

    /// A read-only file is not replaced, on Linux as on Windows: the save
    /// fails, the file is byte for byte as it was, and no temporary
    /// sibling is left.
    #[test]
    fn a_read_only_file_is_not_replaced() {
        let dir = scratch("save_read_only");
        let path = write_file(&dir, "rules.rhai", &official_text());
        let before = fs::read(&path).unwrap();
        let mut doc = Document::open(&path).expect("opens");
        doc.ruleset.name = "Renamed".to_string();
        let mut read_only = fs::metadata(&path).unwrap().permissions();
        read_only.set_readonly(true);
        fs::set_permissions(&path, read_only).unwrap();
        let result = doc.save(&path);
        // The file gets its writable bit back before any assertion, or the
        // scratch folder could not be cleared for the next run.
        let mut writable = fs::metadata(&path).unwrap().permissions();
        #[expect(
            clippy::permissions_set_readonly_false,
            reason = "a test fixture must get its writable bit back or the scratch folder cannot be cleared"
        )]
        writable.set_readonly(false);
        fs::set_permissions(&path, writable).unwrap();
        match result {
            Err(SaveError::Write { error, .. }) => {
                assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
            }
            other => panic!("expected a write failure, got {other:?}"),
        }
        assert_eq!(fs::read(&path).unwrap(), before, "the file is unchanged");
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        assert!(!Path::new(&tmp).exists(), "no temporary sibling is left");
    }
}
