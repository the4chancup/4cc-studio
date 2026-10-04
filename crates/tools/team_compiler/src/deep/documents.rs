//! The deep pass's checks of a folder's small data files, each read whole and parsed: a face
//! folder's face diff (`player_folders.md` "`face_diff.xml`"), a kit's `config.toml`, a
//! player's `settings.toml` and a kit's or the root `colors.txt` (`player_folders.md` "Root
//! files", "Colors") (`team_compiler/messages.md`: `face_diff_invalid`, `xml_dif_conflict`,
//! `kit_config_invalid`, `settings_toml_invalid`, `color_entry_invalid`). None of their
//! findings is pass-through-eligible: a file that cannot be read leaves no value to keep, and
//! a refused `colors.txt` line is a Warning, which drops nothing.

use std::fmt;

use aesthetics_export::{
    ColorLineRefusal, ContentFinding, Disposition, FileDescriptor, IssueScope, KitFolder,
    PlayerFolder, read_colors_txt,
};
use kit_config::KitConfig;
use pes_savefile::settings_toml::PlayerSettings;
use vtree::ScopePath;

use super::{read, relative};
use crate::face_diff;
use crate::messages::Code;
use crate::plan::subset::{FolderModels, PlayerFile, player_file};
use crate::reader::ContentSource;

/// The form a folder gives its face diff in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FaceDiffForm {
    /// A `face_diff.bin`, packed as it is.
    Bin,
    /// A `face_diff.xml`, the `face_diff.bin` as base64 text.
    Xml,
}

/// The form of face diff a file of `role` is, or `None` for any other file. The roles are
/// planning's (`player_file`), so a file planning gives no package (a `face_diff.bin` in
/// `boots/`, or in a folder with no face) is not read.
fn face_diff_form(role: Option<PlayerFile>) -> Option<FaceDiffForm> {
    match role? {
        PlayerFile::Packed {
            name: "face_diff.bin",
            ..
        } => Some(FaceDiffForm::Bin),
        PlayerFile::FaceDiffXml => Some(FaceDiffForm::Xml),
        PlayerFile::Packed { .. }
        | PlayerFile::Model { .. }
        | PlayerFile::CommonModel { .. }
        | PlayerFile::Skeleton { .. }
        | PlayerFile::SlotlessSkeleton
        | PlayerFile::Texture(..) => None,
    }
}

/// The face diff findings of the model folder at `folder` holding `files`, whose models are
/// `models`, over the files planning gives a face diff's role (`player_file`), each dropping
/// the folder before any ID is planned for it. A second copy in `face/`, which planning
/// leaves out, is checked too: a broken file in the folder is worth its finding, whichever
/// copy would be packed. A folder holding both forms gives its face diff twice:
/// `xml_dif_conflict`, naming the `.xml`, and neither file is read. Otherwise each is read
/// and checked, a `face_diff.bin` as it is and a `face_diff.xml` decoded, and one the game
/// cannot read is `face_diff_invalid` with the reason.
pub(super) fn face_diff_findings(
    content: &ContentSource,
    folder: &ScopePath,
    files: &[FileDescriptor],
    models: &FolderModels,
) -> Vec<ContentFinding> {
    let face_diffs: Vec<(&FileDescriptor, FaceDiffForm)> = files
        .iter()
        .filter_map(|file| {
            face_diff_form(player_file(folder, file, models)).map(|form| (file, form))
        })
        .collect();
    let scope = IssueScope::Folder(folder.clone());
    let finding = |code: Code, context| ContentFinding {
        code: code.as_str(),
        scope: scope.clone(),
        context,
        disposition: Disposition::DropFolder,
        pass_through_eligible: false,
    };
    let xml = face_diffs
        .iter()
        .find(|(_, form)| *form == FaceDiffForm::Xml);
    let holds_bin = face_diffs
        .iter()
        .any(|(_, form)| *form == FaceDiffForm::Bin);
    if let Some((xml, _)) = xml
        && holds_bin
    {
        return vec![finding(
            Code::XmlDifConflict,
            vec![("file", relative(&xml.path, folder))],
        )];
    }
    let mut findings = Vec::new();
    for (file, form) in face_diffs {
        let bytes = match read(content, file, &scope, Disposition::DropFolder) {
            Ok(bytes) => bytes,
            Err(unread) => {
                findings.push(unread);
                continue;
            }
        };
        let checked = match form {
            FaceDiffForm::Bin => face_diff::check(&bytes),
            FaceDiffForm::Xml => face_diff::from_xml(&bytes).map(drop),
        };
        if let Err(error) = checked {
            findings.push(finding(
                Code::FaceDiffInvalid,
                vec![
                    ("file", relative(&file.path, folder)),
                    ("reason", error.to_string()),
                ],
            ));
        }
    }
    findings
}

/// `kit_config_invalid` when `kit` has a `config.toml` that is not UTF-8 or that
/// `KitConfig::from_toml` refuses: the kit is dropped, its textures with it, since the config
/// names them.
pub(super) fn kit_config_finding(
    content: &ContentSource,
    kit: &KitFolder,
) -> Option<ContentFinding> {
    let file = kit.config.as_ref()?;
    toml_finding(
        content,
        file,
        Code::KitConfigInvalid,
        &IssueScope::Folder(kit.path.clone()),
        Disposition::DropFolder,
        KitConfig::from_toml,
    )
}

/// `settings_toml_invalid` when `player` has a `settings.toml` that is not UTF-8 or that
/// `PlayerSettings::parse` refuses: the file alone is dropped, the folder's models still
/// compile, and the savefile keeps the player's values.
pub(super) fn settings_finding(
    content: &ContentSource,
    player: &PlayerFolder,
) -> Option<ContentFinding> {
    let file = player.settings.as_ref()?;
    toml_finding(
        content,
        file,
        Code::SettingsTomlInvalid,
        &IssueScope::File(file.path.clone()),
        Disposition::DropFile,
        PlayerSettings::parse,
    )
}

/// One `color_entry_invalid` per line the `colors.txt` `file` refuses (`read_colors_txt`, keeping
/// `capacity` colors), on the file and kept: the line is skipped, nothing is dropped. The
/// context gives the line's number and the reason. A file that cannot be read is
/// `source_read_failed` on the file instead, dropping it.
pub(super) fn colors_findings(
    content: &ContentSource,
    file: &FileDescriptor,
    capacity: usize,
) -> Vec<ContentFinding> {
    let scope = IssueScope::File(file.path.clone());
    let bytes = match read(content, file, &scope, Disposition::DropFile) {
        Ok(bytes) => bytes,
        Err(unread) => return vec![unread],
    };
    read_colors_txt(&bytes, capacity)
        .refused
        .into_iter()
        .map(|refused| {
            let reason = match refused.reason {
                ColorLineRefusal::NotOneColor => "not one color".to_owned(),
                ColorLineRefusal::PastCapacity => format!("more than {capacity} colors"),
            };
            ContentFinding {
                code: Code::ColorEntryInvalid.as_str(),
                scope: scope.clone(),
                context: vec![("line", refused.line.to_string()), ("reason", reason)],
                disposition: Disposition::Keep,
                pass_through_eligible: false,
            }
        })
        .collect()
}

/// `code` on `scope` with `disposition` when the TOML file `file`, directly in its folder, is
/// not UTF-8 text or `parse` refuses it: the context names the file and the error. A file that
/// cannot be read is `source_read_failed` instead.
fn toml_finding<T, E: fmt::Display>(
    content: &ContentSource,
    file: &FileDescriptor,
    code: Code,
    scope: &IssueScope,
    disposition: Disposition,
    parse: impl FnOnce(&str) -> Result<T, E>,
) -> Option<ContentFinding> {
    let bytes = match read(content, file, scope, disposition) {
        Ok(bytes) => bytes,
        Err(unread) => return Some(unread),
    };
    let error = match std::str::from_utf8(&bytes) {
        Ok(text) => parse(text).err()?.to_string(),
        Err(_) => "the file is not UTF-8 text".to_owned(),
    };
    Some(ContentFinding {
        code: code.as_str(),
        scope: scope.clone(),
        context: vec![("file", file.path.name().to_owned()), ("error", error)],
        disposition,
        pass_through_eligible: false,
    })
}

#[cfg(test)]
mod tests {
    use aesthetics_export::{ContentFinding, Disposition, IssueScope};
    use kit_config::KitConfig;
    use pes_savefile::settings_toml::PlayerSettings;
    use vtree::ScopePath;

    use crate::deep::tests::{findings_of, fixture, folder, path, texture, tracer_file};
    use crate::testing::scratch;

    /// A Fox model in which `fmdl`'s check finds nothing (the tracer's right glove), for a
    /// folder whose models only have to be there.
    fn clean_model() -> Vec<u8> {
        tracer_file("glove_r.fmdl")
    }

    /// The face diff fixture `name` (`tests/fixtures/face_diff/README.md`).
    fn face_diff(name: &str) -> Vec<u8> {
        fixture(&format!("face_diff/{name}"))
    }

    /// `dif.bin` one byte shorter than the 944 bytes its header gives.
    fn cut_face_diff() -> Vec<u8> {
        face_diff("dif.bin")[..943].to_vec()
    }

    /// The reason `face_diff_invalid` gives for `cut_face_diff`.
    const CUT_REASON: &str = "the face diff is 943 bytes long, but its header gives 944: the game would read past its end";

    /// `dif.xml` with one payload character replaced by one base64 does not use.
    fn corrupt_face_diff_xml() -> Vec<u8> {
        let mut xml = face_diff("dif.xml");
        let payload = xml.windows(4).position(|bytes| bytes == b"RkFD").unwrap();
        assert!(xml[payload + 100].is_ascii_alphanumeric());
        xml[payload + 100] = b'*';
        xml
    }

    /// The finding `code` on the folder `scope`, dropping it and never passed through, with
    /// `context`.
    fn dropping(
        code: &'static str,
        scope: &str,
        context: &[(&'static str, &str)],
    ) -> ContentFinding {
        ContentFinding {
            code,
            scope: folder(scope),
            context: context
                .iter()
                .map(|(key, value)| (*key, (*value).to_owned()))
                .collect(),
            disposition: Disposition::DropFolder,
            pass_through_eligible: false,
        }
    }

    #[test]
    fn a_face_diff_bin_shorter_than_its_header_gives_drops_its_folder_and_a_whole_one_passes() {
        let temp = scratch("deep_face_diff_bin");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/face_high.fmdl", clean_model()),
                ("Players/03 - A/face_diff.bin", face_diff("dif.bin")),
                ("Players/05 - B/face_high.fmdl", clean_model()),
                ("Players/05 - B/face/face_diff.bin", cut_face_diff()),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [dropping(
                "face_diff_invalid",
                "Players/05 - B",
                &[("file", "face/face_diff.bin"), ("reason", CUT_REASON)]
            )]
        );
    }

    #[test]
    fn a_face_diff_xml_whose_base64_is_corrupt_drops_its_folder_and_a_valid_one_passes() {
        let temp = scratch("deep_face_diff_xml");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/face_high.fmdl", clean_model()),
                ("Players/03 - A/face_diff.xml", face_diff("dif.xml")),
                ("Players/05 - B/face_high.fmdl", clean_model()),
                ("Players/05 - B/face_diff.xml", corrupt_face_diff_xml()),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [dropping(
                "face_diff_invalid",
                "Players/05 - B",
                &[
                    ("file", "face_diff.xml"),
                    ("reason", "the base64 text holds '*' where base64 cannot")
                ]
            )]
        );
    }

    #[test]
    fn a_folder_giving_its_face_diff_in_both_forms_is_a_conflict_and_neither_is_read() {
        let temp = scratch("deep_face_diff_twice");
        // The `.bin` is cut and the `.xml` corrupt: neither is reported, being unread.
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/face_high.fmdl", clean_model()),
                ("Players/03 - A/face/face_diff.bin", cut_face_diff()),
                ("Players/03 - A/face_diff.xml", corrupt_face_diff_xml()),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [dropping(
                "xml_dif_conflict",
                "Players/03 - A",
                &[("file", "face_diff.xml")]
            )]
        );
    }

    #[test]
    fn a_shared_face_folder_s_cut_face_diff_drops_that_folder() {
        let temp = scratch("deep_face_diff_shared");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/Round.face", Vec::new()),
                ("Faces/Round/hair_high.fmdl", clean_model()),
                ("Faces/Round/face_diff.bin", cut_face_diff()),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [dropping(
                "face_diff_invalid",
                "Faces/Round",
                &[("file", "face_diff.bin"), ("reason", CUT_REASON)]
            )]
        );
    }

    #[test]
    fn a_face_diff_bin_planning_gives_no_role_is_not_read() {
        let temp = scratch("deep_face_diff_no_role");
        // In `boots/`, a face file has no package to go in; in a folder with no face model
        // and no face link, neither.
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/face_high.fmdl", clean_model()),
                ("Players/03 - A/boots/face_diff.bin", cut_face_diff()),
                ("Players/05 - B/kit_boots.fmdl", clean_model()),
                ("Players/05 - B/face_diff.bin", cut_face_diff()),
            ],
            &[],
        );
        assert_eq!(findings, []);
    }

    /// `kit_config_invalid` on `Kits/p1` with `error`.
    fn kit_config_invalid(error: &str) -> ContentFinding {
        dropping(
            "kit_config_invalid",
            "Kits/p1",
            &[("file", "config.toml"), ("error", error)],
        )
    }

    /// The deep pass's findings on a kit `p1` whose `config.toml` holds `config`.
    fn kit_findings(name: &str, config: &[u8]) -> Vec<ContentFinding> {
        let temp = scratch(name);
        findings_of(
            temp.path(),
            &[
                ("Kits/p1/kit.png", texture("kit.png")),
                ("Kits/p1/config.toml", config.to_vec()),
            ],
            &[],
        )
    }

    #[test]
    fn a_kit_config_that_does_not_parse_drops_its_kit_and_a_valid_one_passes() {
        let wrong_type = KitConfig::from_toml("shirt = 144").unwrap_err().to_string();
        assert_eq!(
            kit_findings("deep_kit_config_wrong_type", b"shirt = 144"),
            [kit_config_invalid(&wrong_type)]
        );
        assert_eq!(
            kit_findings("deep_kit_config_not_utf8", b"\xFF\xFE"),
            [kit_config_invalid("the file is not UTF-8 text")]
        );
        let tracer = fixture("tracer/studio/egg Tracer/Kits/g1/config.toml");
        assert_eq!(kit_findings("deep_kit_config_valid", &tracer), []);
    }

    /// The deep pass's findings on slot 03's folder holding a face model and a
    /// `settings.toml` of `settings`.
    fn settings_findings(name: &str, settings: &[u8]) -> Vec<ContentFinding> {
        let temp = scratch(name);
        findings_of(
            temp.path(),
            &[
                ("Players/03 - A/face_high.fmdl", clean_model()),
                ("Players/03 - A/settings.toml", settings.to_vec()),
            ],
            &[],
        )
    }

    #[test]
    fn a_settings_toml_that_does_not_parse_drops_that_file_alone_and_the_template_passes() {
        let wrong_type = PlayerSettings::parse("name = 5").unwrap_err().to_string();
        assert_eq!(
            settings_findings("deep_settings_wrong_type", b"name = 5"),
            [ContentFinding {
                code: "settings_toml_invalid",
                scope: IssueScope::File(ScopePath::new("Players/03 - A/settings.toml").unwrap()),
                context: vec![("file", "settings.toml".to_owned()), ("error", wrong_type)],
                disposition: Disposition::DropFile,
                pass_through_eligible: false,
            }]
        );
        let not_utf8 = settings_findings("deep_settings_not_utf8", b"\xFF\xFE");
        assert_eq!(not_utf8.len(), 1, "{not_utf8:?}");
        assert_eq!(
            not_utf8[0].context[1],
            ("error", "the file is not UTF-8 text".to_owned())
        );
        let template = PlayerSettings::default().to_toml().unwrap();
        assert_eq!(
            settings_findings("deep_settings_template", template.as_bytes()),
            []
        );
        assert_eq!(settings_findings("deep_settings_empty", b""), []);
    }

    /// The tracer's kit texture, which the deep pass finds nothing in.
    fn tracer_kit() -> Vec<u8> {
        fixture("tracer/studio/egg Tracer/Kits/g1/kit.dds")
    }

    /// `color_entry_invalid` on the `colors.txt` at `file`, kept, for `line` and `reason`.
    fn color_entry_invalid(file: &str, line: &str, reason: &str) -> ContentFinding {
        ContentFinding {
            code: "color_entry_invalid",
            scope: IssueScope::File(path(file)),
            context: vec![("line", line.to_owned()), ("reason", reason.to_owned())],
            disposition: Disposition::Keep,
            pass_through_eligible: false,
        }
    }

    /// The deep pass's findings on a kit `p1` holding the tracer's `kit.dds` and `files`
    /// (path, contents) beside it.
    fn colors_findings(name: &str, files: &[(&str, &[u8])]) -> Vec<ContentFinding> {
        let temp = scratch(name);
        let mut written = vec![("Kits/p1/kit.dds", tracer_kit())];
        written.extend(files.iter().map(|(path, bytes)| (*path, bytes.to_vec())));
        findings_of(temp.path(), &written, &[])
    }

    #[test]
    fn a_kit_s_colors_txt_line_that_gives_no_color_is_kept_with_its_number_and_reason() {
        assert_eq!(
            colors_findings(
                "deep_kit_colors_clean",
                &[("Kits/p1/colors.txt", b"#c11200\n")]
            ),
            []
        );
        assert_eq!(
            colors_findings(
                "deep_kit_colors_bad_line",
                &[("Kits/p1/colors.txt", b"#c11200\nbad\n")]
            ),
            [color_entry_invalid(
                "Kits/p1/colors.txt",
                "2",
                "not one color"
            )]
        );
        assert_eq!(
            colors_findings(
                "deep_kit_colors_three",
                &[("Kits/p1/colors.txt", b"#c11200\n#414141\n211 74 79\n")]
            ),
            [color_entry_invalid(
                "Kits/p1/colors.txt",
                "3",
                "more than 2 colors"
            )]
        );
    }

    #[test]
    fn the_root_colors_txt_keeps_four_colors_and_reports_a_fifth() {
        assert_eq!(
            colors_findings(
                "deep_team_colors_four",
                &[("colors.txt", b"#c11200\n#414141\n211 74 79\n1 2 3\n")]
            ),
            []
        );
        assert_eq!(
            colors_findings(
                "deep_team_colors_five",
                &[("colors.txt", b"#c11200\n#414141\n211 74 79\n1 2 3\n4 5 6\n")]
            ),
            [color_entry_invalid("colors.txt", "5", "more than 4 colors")]
        );
    }

    #[test]
    fn a_kit_s_colors_stand_between_its_config_and_its_textures_and_the_root_s_come_last() {
        let findings = colors_findings(
            "deep_colors_order",
            &[
                ("colors.txt", b"bad\n"),
                ("logo.png", b"not an image"),
                ("Kits/p1/kit_back.png", &texture("tiny.png")),
                ("Kits/p1/colors.txt", b"bad\n"),
                ("Kits/p1/config.toml", b"shirt = 144"),
            ],
        );
        let order: Vec<(&str, IssueScope)> = findings
            .iter()
            .map(|finding| (finding.code, finding.scope.clone()))
            .collect();
        assert_eq!(
            order,
            [
                ("kit_config_invalid", folder("Kits/p1")),
                (
                    "color_entry_invalid",
                    IssueScope::File(path("Kits/p1/colors.txt"))
                ),
                ("texture_too_small", folder("Kits/p1")),
                ("logo_file_invalid", IssueScope::File(path("logo.png"))),
                ("color_entry_invalid", IssueScope::File(path("colors.txt"))),
            ]
        );
    }

    #[test]
    fn a_folder_s_face_diff_and_settings_stand_with_it_and_a_kit_s_config_before_its_textures() {
        let temp = scratch("deep_documents_order");
        let findings = findings_of(
            temp.path(),
            &[
                ("Kits/p1/kit_back.png", texture("tiny.png")),
                ("Kits/p1/kit.png", texture("kit.png")),
                ("Kits/p1/config.toml", b"shirt = 144".to_vec()),
                ("Players/05 - B/face_high.fmdl", clean_model()),
                ("Players/05 - B/face_diff.bin", cut_face_diff()),
                ("Players/03 - A/settings.toml", b"name = 5".to_vec()),
                ("Players/03 - A/portrait.png", texture("tiny.png")),
                ("Players/03 - A/skin.png", texture("tiny.png")),
                ("Players/03 - A/face_high.fmdl", clean_model()),
                ("Players/03 - A/face_diff.bin", cut_face_diff()),
            ],
            &[],
        );
        let order: Vec<(&str, &str)> = findings
            .iter()
            .map(|finding| (finding.code, finding.context[0].1.as_str()))
            .collect();
        assert_eq!(
            order,
            [
                ("texture_too_small", "skin.png"),
                ("face_diff_invalid", "face_diff.bin"),
                ("texture_too_small", "portrait.png"),
                ("settings_toml_invalid", "settings.toml"),
                ("face_diff_invalid", "face_diff.bin"),
                ("kit_config_invalid", "config.toml"),
                ("texture_too_small", "kit_back.png"),
            ]
        );
        assert_eq!(findings[4].scope, folder("Players/05 - B"));
    }
}
