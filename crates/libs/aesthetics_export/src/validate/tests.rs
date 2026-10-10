use super::*;
use crate::testing::{context_with, report, report_with};
use crate::validate::IssueScope;
use crate::{ExportCoverage, FileKind, ModelFormat, SharedKind};
use vtree::ScopePath;

fn issue_codes(report: &ValidationReport) -> Vec<(&'static str, Disposition)> {
    report
        .issues
        .iter()
        .map(|issue| (issue.code, issue.disposition))
        .collect()
}

// TC-STR-04
#[test]
fn an_export_with_no_usable_content_is_dropped_empty() {
    let report = report("egg Midcup", &[("readme.txt", 10)], &[], &[]);
    assert_eq!(
        issue_codes(&report),
        vec![("export_empty", Disposition::DropExport)]
    );
    assert_eq!(report.issues[0].scope, IssueScope::Export);
    assert!(report.validated.is_none());
}

#[test]
fn an_ambiguous_or_conflicting_root_reports_no_export_empty() {
    for files in [
        &[("a/Players/x/face_high.fmdl", 10), ("b/Kits/p1/kit.dds", 9)][..],
        &[
            ("notes.txt", 5),
            ("wrapper/notes.txt", 5),
            ("wrapper/Players/03 - A/face_high.fmdl", 10),
        ][..],
    ] {
        let report = report("egg Midcup", files, &[], &[]);
        assert!(
            report
                .issues
                .iter()
                .all(|issue| issue.code != "export_empty"),
            "{files:?}"
        );
    }
}

// TC-ID-04
#[test]
fn a_stem_with_no_team_token_is_dropped_unknown() {
    let report = report("--- ", &[("Players/03 - A/face_high.fmdl", 10)], &[], &[]);
    assert_eq!(
        issue_codes(&report),
        vec![("team_name_unknown", Disposition::DropExport)]
    );
    assert_eq!(report.issues[0].scope, IssueScope::Export);
    assert_eq!(report.issues[0].context, vec![("team_name", String::new())]);
    assert!(report.validated.is_none());
}

#[test]
fn a_team_export_without_its_coverage_tag_is_dropped() {
    let report = report(
        "co Spring 2026",
        &[("Players/03 - A/face_high.fmdl", 10)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("export_tag_missing", Disposition::DropExport)]
    );
    assert_eq!(report.issues[0].scope, IssueScope::Export);
    assert_eq!(
        report.issues[0].context,
        vec![("name", "co Spring 2026".to_owned())]
    );
    assert!(report.validated.is_none());
}

#[test]
fn an_old_layout_export_is_export_layout_old_alone() {
    for kit_configs in ["Kit Configs", "kit configs"] {
        let report = report(
            "co Spring 2026",
            &[
                ("Faces/01 - A/face.model", 10),
                (&format!("{kit_configs}/co_DEF_1st_realUni.bin"), 10),
                ("Kit Textures/u0714p1.dds", 10),
                ("Portraits/player_01.dds", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("export_layout_old", Disposition::DropExport)],
            "{kit_configs}"
        );
        assert_eq!(report.issues[0].scope, IssueScope::Export);
        assert_eq!(
            report.issues[0].context,
            vec![("folder", kit_configs.to_owned())]
        );
        assert!(report.validated.is_none());
    }
}

#[test]
fn a_tagged_team_export_carries_its_coverage() {
    for (name, coverage) in [
        ("co Full Spring 2026", ExportCoverage::Full),
        ("co midcup day 5", ExportCoverage::Midcup),
    ] {
        let report = report(name, &[("Players/03 - A/face_high.fmdl", 10)], &[], &[]);
        assert_eq!(issue_codes(&report), vec![], "{name}");
        assert_eq!(report.validated.unwrap().coverage, coverage, "{name}");
    }
}

#[test]
fn a_referee_export_needs_no_tag_and_is_full() {
    let report = report(
        "refs Spring 2026",
        &[("players.txt", 10)],
        &["Players/Keeper"],
        &[("players.txt", Ok(b"01 Keeper"))],
    );
    assert_eq!(issue_codes(&report), vec![]);
    assert_eq!(report.validated.unwrap().coverage, ExportCoverage::Full);
}

#[test]
fn a_player_folder_builds_with_its_links_and_markers() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/hair.dds", 9),
            ("Players/03 - A/gloves/glove_l.fmdl", 10),
            ("Players/03 - A/common/skin.dds", 9),
            ("Players/03 - A/Crocs.boots.txt", 0),
            ("Players/03 - A/ingame_face", 0),
            ("Players/03 - A/fpc_on", 0),
            ("Players/03 - A/portrait.dds", 9),
            ("Players/03 - A/settings.toml", 20),
            ("Boots/Crocs/boots.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    let folder = &report.validated.unwrap().players[0];
    assert_eq!(folder.player_name, "A");
    assert_eq!(
        folder.links,
        vec![SharedLink {
            kind: SharedKind::Boots,
            name: "Crocs".to_owned(),
        }]
    );
    assert!(folder.ingame_face);
    assert_eq!(folder.fpc, Some(FpcDirective::On));
    assert_eq!(
        folder.portrait.as_ref().map(|f| f.path.as_str()),
        Some("Players/03 - A/portrait.dds")
    );
    assert_eq!(
        folder.settings.as_ref().map(|f| f.path.as_str()),
        Some("Players/03 - A/settings.toml")
    );
    // Pipeline content: loose files and reserved-subfolder files alike.
    assert_eq!(
        folder
            .files
            .iter()
            .map(|f| f.path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "Players/03 - A/common/skin.dds",
            "Players/03 - A/gloves/glove_l.fmdl",
            "Players/03 - A/hair.dds",
        ]
    );
    assert_eq!(folder.files[1].kind, FileKind::Model(ModelFormat::Fmdl));
}

#[test]
fn only_the_portrait_stem_is_the_portrait() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/hair.dds", 9),
            ("Players/03 - A/portrait.dds", 9),
        ],
        &[],
        &[],
    );
    let folder = &report.validated.unwrap().players[0];
    assert_eq!(
        folder.portrait.as_ref().map(|file| file.path.as_str()),
        Some("Players/03 - A/portrait.dds")
    );
    assert_eq!(
        folder
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        vec!["Players/03 - A/hair.dds"]
    );
}

// TC-STR-13
#[test]
fn two_textures_sharing_a_stem_drop_the_folder() {
    for files in [
        &[
            ("Players/03 - A/hair.dds", 9),
            ("Players/03 - A/hair.png", 9),
        ][..],
        &[
            ("Players/03 - A/hair.dds", 9),
            ("Players/03 - A/common/hair.dds", 9),
        ][..],
    ] {
        let report = report("egg Midcup", files, &[], &[]);
        let issues: Vec<(&'static str, Disposition)> = report
            .issues
            .iter()
            .map(|issue| (issue.code, issue.disposition))
            .collect();
        assert_eq!(
            issues,
            vec![("texture_stem_conflict", Disposition::DropFolder)]
        );
        assert_eq!(
            report.issues[0].scope,
            IssueScope::Folder(ScopePath::new("Players/03 - A").unwrap())
        );
        assert!(report.validated.unwrap().players.is_empty(), "{files:?}");
    }
}

#[test]
fn fpc_off_alone_is_an_off_directive() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/face_high.fmdl", 10),
            ("Players/03 - A/fpc_off", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(
        report.validated.unwrap().players[0].fpc,
        Some(FpcDirective::Off)
    );
}

#[test]
fn a_bare_fpc_marker_is_an_on_directive() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/face_high.fmdl", 10),
            ("Players/03 - A/fpc", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    assert_eq!(
        report.validated.unwrap().players[0].fpc,
        Some(FpcDirective::On)
    );
}

#[test]
fn the_dotted_marker_spellings_are_disallowed_files() {
    for (files, folder_path, file) in [
        (
            &[
                ("Players/03 - A/face_high.fmdl", 10),
                ("Players/03 - A/fpc.on", 0),
            ][..],
            "Players/03 - A",
            "fpc.on",
        ),
        (
            &[("Kits/p1/kit.dds", 9), ("Kits/p1/icon.txt", 2)][..],
            "Kits/p1",
            "icon.txt",
        ),
    ] {
        let report = report("egg Midcup", files, &[], &[]);
        assert_eq!(
            issue_codes(&report),
            vec![("file_type_disallowed", Disposition::DropFolder)],
            "{file}"
        );
        assert_eq!(report.issues[0].scope, folder(folder_path));
        assert_eq!(report.issues[0].context, vec![("file", file.to_owned())]);
    }
}

// TC-STR-12
#[test]
fn both_fpc_markers_drop_the_folder() {
    // A roster-mapped player dropped by its own finding.
    let report = report(
        "egg Midcup",
        &[
            ("players.txt", 5),
            ("Players/A/hair.dds", 9),
            ("Players/A/fpc_on", 0),
            ("Players/A/fpc_off", 0),
        ],
        &[],
        &[("players.txt", Ok(b"03 A"))],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("fpc_conflict", Disposition::DropFolder)]
    );
    let validated = report.validated.unwrap();
    assert!(validated.players.is_empty());
    let ValidatedRoster::Team(map) = &validated.roster else {
        panic!("a team export");
    };
    assert!(map.is_empty());
}

#[test]
fn shared_folders_and_common_files_build_unchecked() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/Longhair.face", 0),
            ("Players/03 - A/Crocs.boots", 0),
            ("Players/03 - A/Keeper gloves.gloves", 0),
            ("Players/03 - A/hair.dds", 9),
            ("Faces/Longhair/face_high.fmdl", 10),
            ("Boots/Crocs/boots.fmdl", 10),
            ("Gloves/Keeper gloves/glove_l.fmdl", 10),
            ("Common/hair.dds", 9),
            ("Collars/collar_101.fmdl", 10),
        ],
        &[],
        &[],
    );
    let validated = report.validated.unwrap();
    assert_eq!(validated.faces[0].folder_name, "Longhair");
    assert_eq!(
        validated.faces[0].files[0].path.as_str(),
        "Faces/Longhair/face_high.fmdl"
    );
    assert_eq!(validated.boots[0].folder_name, "Crocs");
    assert_eq!(validated.gloves[0].folder_name, "Keeper gloves");
    assert_eq!(validated.common[0].path.as_str(), "Common/hair.dds");
    assert_eq!(validated.common[0].kind, FileKind::Texture);
    assert_eq!(
        validated.collars[0].path.as_str(),
        "Collars/collar_101.fmdl"
    );
}

fn folder(path: &str) -> IssueScope {
    IssueScope::Folder(ScopePath::new(path).unwrap())
}

// TC-STR-05
#[test]
fn a_shared_link_resolves_with_or_without_the_txt_tail() {
    for link_file in ["Crocs.boots", "Crocs.boots.txt"] {
        let path = format!("Players/03 - A/{link_file}");
        let report = report(
            "egg Midcup",
            &[
                ("Players/03 - A/hair.dds", 9),
                (path.as_str(), 0),
                ("Boots/Crocs/boots.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![], "{link_file}");
        let player = &report.validated.unwrap().players[0];
        assert_eq!(
            player.links,
            vec![SharedLink {
                kind: SharedKind::Boots,
                name: "Crocs".to_owned(),
            }],
            "{link_file}"
        );
    }
}

// TC-STR-06
#[test]
fn a_link_to_a_missing_shared_folder_drops_the_player() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/hair.dds", 9),
            ("Players/03 - A/Nowhere.boots", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("link_target_missing", Disposition::DropFolder)]
    );
    assert_eq!(report.issues[0].scope, folder("Players/03 - A"));
    assert_eq!(
        report.issues[0].context,
        vec![("link", "Nowhere.boots".to_owned())]
    );
    assert!(report.validated.unwrap().players.is_empty());
}

// TC-STR-07
#[test]
fn two_links_of_one_kind_drop_the_player() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/hair.dds", 9),
            ("Players/03 - A/Crocs.boots", 0),
            ("Players/03 - A/Mud.boots", 0),
        ],
        &[],
        &[],
    );
    // Every finding is reported: the duplicate, and each missing target.
    assert_eq!(
        issue_codes(&report),
        vec![
            ("shared_link_duplicate", Disposition::DropFolder),
            ("link_target_missing", Disposition::DropFolder),
            ("link_target_missing", Disposition::DropFolder),
        ]
    );
    assert_eq!(report.issues[0].context, vec![("kind", "boots".to_owned())]);
    assert!(report.validated.unwrap().players.is_empty());
}

// TC-STR-08
#[test]
fn a_shared_folder_no_surviving_player_links_is_orphaned() {
    let report = report(
        "egg Midcup",
        &[
            ("players.txt", 5),
            ("Boots/Solo/boots.fmdl", 10),
            ("Boots/Duo/boots.fmdl", 10),
            ("Players/Droppable/Duo.boots", 0),
        ],
        &["Players/A", "Players/Droppable"],
        &[("players.txt", Ok(b"03 A"))],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("player_unlisted", Disposition::DropFolder),
            ("shared_folder_orphaned", Disposition::DropFolder),
            ("shared_folder_orphaned", Disposition::DropFolder),
        ]
    );
    assert_eq!(report.issues[0].scope, folder("Players/Droppable"));
    assert_eq!(report.issues[1].scope, folder("Boots/Duo"));
    assert_eq!(report.issues[2].scope, folder("Boots/Solo"));
    let validated = report.validated.unwrap();
    assert_eq!(validated.players.len(), 1);
    assert!(validated.boots.is_empty());
}

#[test]
fn a_disallowed_file_drops_strict_and_keeps_lenient() {
    let files = &[
        ("Players/03 - A/hair.dds", 9u64),
        ("Players/03 - A/readme.txt", 20),
    ];
    let strict = report("egg Midcup", files, &[], &[]);
    assert_eq!(
        issue_codes(&strict),
        vec![("file_type_disallowed", Disposition::DropFolder)]
    );
    assert!(strict.validated.unwrap().players.is_empty());

    let lenient = report_with(&context_with(false, false), "egg Midcup", files, &[], &[]);
    assert_eq!(
        issue_codes(&lenient),
        vec![("file_type_disallowed", Disposition::Keep)]
    );
    let player = &lenient.validated.unwrap().players[0];
    assert_eq!(
        player
            .files
            .iter()
            .map(|file| file.path.name().to_owned())
            .collect::<Vec<_>>(),
        vec!["hair.dds", "readme.txt"]
    );
}

// TC-STR-10
#[test]
fn both_ingame_face_spellings_mark_the_folder() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/ingame_face", 0),
            ("Players/03 - A/hair.dds", 9),
            ("Players/04 - B/ingame_face.txt", 0),
            ("Players/04 - B/hair.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    let validated = report.validated.unwrap();
    assert!(validated.players[0].ingame_face);
    assert!(validated.players[1].ingame_face);
}

// TC-STR-11
#[test]
fn ingame_face_with_explicit_face_content_drops_the_folder() {
    let local = report(
        "egg Midcup",
        &[
            ("Players/03 - A/ingame_face", 0),
            ("Players/03 - A/face_high.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&local),
        vec![("ingame_face_explicit_face_model", Disposition::DropFolder)]
    );
    assert!(local.validated.unwrap().players.is_empty());

    let linked = report(
        "egg Midcup",
        &[
            ("Players/03 - A/ingame_face", 0),
            ("Players/03 - A/Long.face", 0),
            ("Faces/Long/face_high.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&linked),
        vec![
            ("ingame_face_explicit_face_model", Disposition::DropFolder),
            ("shared_folder_orphaned", Disposition::DropFolder),
        ]
    );
    let validated = linked.validated.unwrap();
    assert!(validated.players.is_empty());
    assert!(validated.faces.is_empty());
}

// TC-STR-14
#[test]
fn ingame_face_acts_by_subfolder_category() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/ingame_face", 0),
            ("Players/03 - A/boots/hair_high.fmdl", 10),
            ("Players/03 - A/gloves/glove_l.fmdl", 10),
            ("Players/03 - A/common/skin.dds", 9),
            ("Players/04 - B/ingame_face", 0),
            ("Players/04 - B/face/hair_high.fmdl", 10),
            ("Players/05 - C/ingame_face", 0),
            ("Players/05 - C/common/x.fmdl", 10),
            ("Players/05 - C/extra/x.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("ingame_face_explicit_face_model", Disposition::DropFolder),
            ("file_type_disallowed", Disposition::DropFolder),
            ("file_type_disallowed", Disposition::DropFolder),
        ]
    );
    assert_eq!(report.issues[0].scope, folder("Players/04 - B"));
    assert_eq!(report.issues[1].scope, folder("Players/05 - C"));
    assert_eq!(
        report.issues[1].context,
        vec![("file", "common/x.fmdl".to_owned())]
    );
    assert_eq!(report.issues[2].scope, folder("Players/05 - C"));
    assert_eq!(
        report.issues[2].context,
        vec![("file", "extra/x.dds".to_owned())]
    );
    // The boots/gloves/common files are those categories' parts: no
    // finding for A.
    let validated = report.validated.unwrap();
    assert_eq!(
        validated
            .players
            .iter()
            .map(|player| player.path.as_str())
            .collect::<Vec<_>>(),
        vec!["Players/03 - A"]
    );
}

// TC-STR-15
#[test]
fn a_common_link_resolves_or_drops_the_player() {
    let report = report(
        "egg Midcup",
        &[
            ("Common/torso.fmdl", 10),
            ("Players/01 - A/torso.fmdl.common", 0),
            ("Players/02 - B/torso.fmdl.common.txt", 0),
            ("Players/03 - C/missing.fmdl.common", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("common_link_missing", Disposition::DropFolder)]
    );
    assert_eq!(report.issues[0].scope, folder("Players/03 - C"));
    assert_eq!(
        report.issues[0].context,
        vec![
            ("link", "missing.fmdl.common".to_owned()),
            ("path", "Common/missing.fmdl".to_owned()),
        ]
    );
    let validated = report.validated.unwrap();
    assert_eq!(
        validated
            .players
            .iter()
            .map(|player| player.path.as_str())
            .collect::<Vec<_>>(),
        vec!["Players/01 - A", "Players/02 - B"]
    );
    // Resolved `.common` links stay as link-kind files.
    assert_eq!(validated.players[0].files[0].kind, FileKind::CommonLink);
}

/// The default context with the installed CPKs holding the team's Common textures `stems`,
/// folded as the consumer folds them.
fn installed(stems: &[&str]) -> ValidationContext {
    ValidationContext {
        installed_common_textures: stems.iter().map(|stem| vtree::fold_name(stem)).collect(),
        ..crate::testing::context()
    }
}

/// The file names of the validated export's player `index`.
fn player_file_names(report: &ValidationReport, index: usize) -> Vec<String> {
    report.validated.as_ref().unwrap().players[index]
        .files
        .iter()
        .map(|file| file.path.name().to_owned())
        .collect()
}

#[test]
fn a_texture_link_whose_texture_an_installed_cpk_holds_stays_with_no_finding() {
    // The set is folded by its producer and the link's stem by validation, so either spelling
    // matches.
    for (link, held) in [("hair.dds.common", "hair"), ("Hair.dds.common", "HAIR")] {
        let report = report_with(
            &installed(&[held]),
            "egg Midcup",
            &[
                ("Players/03 - A/face_high.fmdl", 10),
                (&format!("Players/03 - A/{link}"), 0),
            ],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![], "{link}");
        assert_eq!(
            player_file_names(&report, 0),
            vec!["face_high.fmdl".to_owned(), link.to_owned()],
            "{link}"
        );
    }
}

#[test]
fn a_model_link_is_common_link_missing_whatever_the_installed_cpks_hold() {
    let report = report_with(
        &installed(&["face_high"]),
        "egg Midcup",
        &[("Players/03 - A/face_high.fmdl.common", 0)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("common_link_missing", Disposition::DropFolder)]
    );
    assert_eq!(report.issues[0].scope, folder("Players/03 - A"));
}

#[test]
fn with_no_installed_texture_a_texture_link_with_no_target_is_common_link_missing() {
    for context in [installed(&[]), installed(&["skin"])] {
        let report = report_with(
            &context,
            "egg Midcup",
            &[
                ("Players/03 - A/face_high.fmdl", 10),
                ("Players/03 - A/hair.dds.common", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("common_link_missing", Disposition::DropFolder)]
        );
        assert_eq!(
            report.issues[0].context,
            vec![
                ("link", "hair.dds.common".to_owned()),
                ("path", "Common/hair.dds".to_owned()),
            ]
        );
    }
}

// TC-STR-16
#[test]
fn a_dropped_link_target_drops_its_players_and_pass_through_keeps() {
    let files = &[
        ("Faces/Base/x.exe", 4u64),
        ("Players/03 - A/Base.face", 0),
        ("Players/03 - A/hair.dds", 9),
    ];
    let strict = report("egg Midcup", files, &[], &[]);
    assert_eq!(
        issue_codes(&strict),
        vec![
            ("file_type_disallowed", Disposition::DropFolder),
            ("link_target_dropped", Disposition::DropFolder),
        ]
    );
    assert_eq!(strict.issues[0].scope, folder("Faces/Base"));
    assert_eq!(strict.issues[1].scope, folder("Players/03 - A"));
    assert_eq!(
        strict.issues[1].context,
        vec![
            ("link", "Base.face".to_owned()),
            ("target", "Faces/Base".to_owned()),
            ("finding", "file_type_disallowed".to_owned()),
        ]
    );
    let validated = strict.validated.unwrap();
    assert!(validated.players.is_empty());
    assert!(validated.faces.is_empty());

    // With pass_through the target stays, no link_target_dropped.
    let kept = report_with(&context_with(true, true), "egg Midcup", files, &[], &[]);
    assert_eq!(
        issue_codes(&kept),
        vec![("file_type_disallowed", Disposition::Keep)]
    );
    assert!(kept.issues[0].passed_through);
    let validated = kept.validated.unwrap();
    assert_eq!(validated.players.len(), 1);
    assert_eq!(validated.faces.len(), 1);
    assert_eq!(
        validated.players[0].links,
        vec![SharedLink {
            kind: SharedKind::Face,
            name: "Base".to_owned(),
        }]
    );

    // A conflict is never pass-through-eligible: both drop.
    let conflicted = report_with(
        &context_with(true, true),
        "egg Midcup",
        &[
            ("Faces/Base/hair.dds", 9),
            ("Faces/Base/hair.png", 9),
            ("Players/03 - A/Base.face", 0),
            ("Players/03 - A/hair.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&conflicted),
        vec![
            ("texture_stem_conflict", Disposition::DropFolder),
            ("link_target_dropped", Disposition::DropFolder),
        ]
    );
    assert!(conflicted.issues.iter().all(|issue| !issue.passed_through));
    let validated = conflicted.validated.unwrap();
    assert!(validated.players.is_empty());
    assert!(validated.faces.is_empty());
}

// TC-STR-17
#[test]
fn a_wrongly_named_boots_model_drops_folder_and_linker() {
    let report = report(
        "egg Midcup",
        &[
            ("Boots/Crocs/torso.fmdl", 10),
            ("Boots/Mud/kit_boots.fmdl", 10),
            ("Players/01 - A/Crocs.boots", 0),
            ("Players/01 - A/hair.dds", 9),
            ("Players/02 - B/Mud.boots", 0),
            ("Players/02 - B/hair.dds", 9),
            ("Players/03 - C/torso.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("fmdl_name_invalid", Disposition::DropFolder),
            ("link_target_dropped", Disposition::DropFolder),
        ]
    );
    assert_eq!(report.issues[0].scope, folder("Boots/Crocs"));
    assert_eq!(
        report.issues[0].context,
        vec![("file", "torso.fmdl".to_owned())]
    );
    assert_eq!(report.issues[1].scope, folder("Players/01 - A"));
    let validated = report.validated.unwrap();
    assert_eq!(
        validated
            .players
            .iter()
            .map(|player| player.path.as_str())
            .collect::<Vec<_>>(),
        vec!["Players/02 - B", "Players/03 - C"]
    );
    assert_eq!(
        validated
            .boots
            .iter()
            .map(|folder| folder.folder_name.as_str())
            .collect::<Vec<_>>(),
        vec!["Mud"]
    );
}

#[test]
fn pass_through_keeps_a_dangling_link_player_minus_the_link() {
    let report = report_with(
        &context_with(true, true),
        "egg Midcup",
        &[
            ("Players/03 - A/Nowhere.boots", 0),
            ("Players/03 - A/hair.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("link_target_missing", Disposition::Keep)]
    );
    assert!(report.issues[0].passed_through);
    let player = &report.validated.unwrap().players[0];
    // The player is kept; the missing reference is not.
    assert!(player.links.is_empty());
}

#[test]
fn a_common_file_off_the_allowlist_drops_or_keeps() {
    let strict = report("egg Midcup", &[("Common/x.exe", 4)], &[], &[]);
    assert_eq!(
        issue_codes(&strict),
        vec![("common_file_disallowed", Disposition::DropFile)]
    );
    assert_eq!(
        strict.issues[0].scope,
        IssueScope::File(ScopePath::new("Common/x.exe").unwrap())
    );
    assert!(strict.validated.unwrap().common.is_empty());

    let lenient = report_with(
        &context_with(false, false),
        "egg Midcup",
        &[("Common/x.exe", 4)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&lenient),
        vec![("common_file_disallowed", Disposition::Keep)]
    );
    assert_eq!(lenient.validated.unwrap().common.len(), 1);
}

#[test]
fn only_model_files_directly_in_collars_are_kept() {
    let files = [
        ("Collars/collar_12.fmdl", 10),
        ("Collars/collar_13.model", 10),
        ("Collars/collar_14.glb", 10),
        ("Collars/collar_15.gltf", 10),
        ("Collars/collar_12.dds", 9),
        ("Collars/collar_12.mtl", 9),
        ("Collars/old/collar_16.fmdl", 10),
    ];
    let strict = report("co Midcup", &files, &[], &[]);
    let disallowed = |path: &str, file: &str| ValidationIssue {
        code: "file_type_disallowed",
        scope: IssueScope::File(ScopePath::new(path).unwrap()),
        context: vec![("file", file.to_owned())],
        disposition: Disposition::DropFile,
        passed_through: false,
    };
    assert_eq!(
        strict.issues,
        [
            disallowed("Collars/collar_12.dds", "collar_12.dds"),
            disallowed("Collars/collar_12.mtl", "collar_12.mtl"),
            disallowed("Collars/old/collar_16.fmdl", "old/collar_16.fmdl"),
        ]
    );
    let mut kept: Vec<String> = strict
        .validated
        .unwrap()
        .collars
        .iter()
        .map(|file| file.path.as_str().to_owned())
        .collect();
    kept.sort_unstable();
    assert_eq!(
        kept,
        [
            "Collars/collar_12.fmdl",
            "Collars/collar_13.model",
            "Collars/collar_14.glb",
            "Collars/collar_15.gltf",
        ]
    );

    // With the strict check off, the files are only noted and stay.
    let lenient = report_with(&context_with(false, false), "co Midcup", &files, &[], &[]);
    assert_eq!(
        issue_codes(&lenient),
        vec![("file_type_disallowed", Disposition::Keep); 3]
    );
    assert_eq!(lenient.validated.unwrap().collars.len(), files.len());
}

#[test]
fn pass_through_keeps_the_resolved_common_link_and_prunes_the_other() {
    let report = report_with(
        &context_with(true, true),
        "egg Midcup",
        &[
            ("players.txt", 5),
            ("Common/torso.fmdl", 10),
            ("Players/A/torso.fmdl.common", 0),
            ("Players/A/missing.fmdl.common", 0),
            ("Players/A/hair.dds", 9),
        ],
        &["Players/A"],
        &[("players.txt", Ok(b"03 A"))],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("common_link_missing", Disposition::Keep)]
    );
    assert!(report.issues[0].passed_through);
    let player = &report.validated.unwrap().players[0];
    assert_eq!(
        player
            .files
            .iter()
            .map(|file| file.path.name().to_owned())
            .collect::<Vec<_>>(),
        vec!["hair.dds", "torso.fmdl.common"]
    );
}

#[test]
fn boots_and_fcl_hair_models_are_no_explicit_face_content() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/ingame_face", 0),
            ("Players/03 - A/kit_boots.fmdl", 10),
            ("Players/03 - A/x_fcl_hair.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    assert_eq!(report.validated.unwrap().players.len(), 1);
}

#[test]
fn wrong_suffixed_models_drop_their_shared_folders_and_linkers() {
    let report = report(
        "egg Midcup",
        &[
            ("Boots/X/glove_l.fmdl", 10),
            ("Gloves/Y/kit_boots.fmdl", 10),
            ("Players/01 - A/X.boots", 0),
            ("Players/01 - A/hair.dds", 9),
            ("Players/02 - B/Y.gloves", 0),
            ("Players/02 - B/hair.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("fmdl_name_invalid", Disposition::DropFolder),
            ("fmdl_name_invalid", Disposition::DropFolder),
            ("link_target_dropped", Disposition::DropFolder),
            ("link_target_dropped", Disposition::DropFolder),
        ]
    );
    assert_eq!(report.issues[0].scope, folder("Boots/X"));
    assert_eq!(report.issues[1].scope, folder("Gloves/Y"));
    let validated = report.validated.unwrap();
    assert!(validated.players.is_empty());
    assert!(validated.boots.is_empty());
    assert!(validated.gloves.is_empty());
}

#[test]
fn a_nested_common_model_file_is_admitted_but_no_link_target() {
    let report = report(
        "egg Midcup",
        &[
            ("Common/sub/torso.fmdl", 10),
            ("Common/sub/notes.txt", 4),
            ("Players/03 - A/torso.fmdl.common", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("common_link_missing", Disposition::DropFolder),
            ("common_file_disallowed", Disposition::DropFile),
        ]
    );
    assert_eq!(report.issues[0].scope, folder("Players/03 - A"));
    assert_eq!(
        report.issues[1].scope,
        IssueScope::File(ScopePath::new("Common/sub/notes.txt").unwrap())
    );
    let validated = report.validated.unwrap();
    let common: Vec<&str> = validated
        .common
        .iter()
        .map(|file| file.path.as_str())
        .collect();
    assert_eq!(common, vec!["Common/sub/torso.fmdl"]);
}

#[test]
fn reserved_folders_admit_only_model_content_and_common_links() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/face/settings.toml", 20),
            ("Players/04 - B/boots/torso.fmdl.common", 0),
            ("Common/torso.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("file_type_disallowed", Disposition::DropFolder)]
    );
    assert_eq!(report.issues[0].scope, folder("Players/03 - A"));
    assert_eq!(
        report.issues[0].context,
        vec![("file", "face/settings.toml".to_owned())]
    );
    let validated = report.validated.unwrap();
    assert_eq!(validated.players.len(), 1);
}

#[test]
fn ingame_face_and_common_links_act_by_target_and_position() {
    let report = report(
        "egg Midcup",
        &[
            ("Common/face_high.fmdl", 10),
            ("Common/torso.fmdl", 10),
            ("Common/face_high.dds", 9),
            ("Players/01 - A/ingame_face", 0),
            ("Players/01 - A/face_high.fmdl.common", 0),
            ("Players/02 - B/ingame_face", 0),
            ("Players/02 - B/torso.fmdl.common", 0),
            ("Players/03 - C/ingame_face", 0),
            ("Players/03 - C/face_high.dds.common", 0),
            ("Players/04 - D/ingame_face", 0),
            ("Players/04 - D/boots/face_high.fmdl.common", 0),
        ],
        &[],
        &[],
    );
    // Only A's explicit-face-model link contradicts the marker.
    assert_eq!(
        issue_codes(&report),
        vec![("ingame_face_explicit_face_model", Disposition::DropFolder)]
    );
    assert_eq!(report.issues[0].scope, folder("Players/01 - A"));
    let validated = report.validated.unwrap();
    assert_eq!(
        validated
            .players
            .iter()
            .map(|player| player.path.as_str())
            .collect::<Vec<_>>(),
        vec!["Players/02 - B", "Players/03 - C", "Players/04 - D"]
    );
}

#[test]
fn a_shared_folder_with_different_kinds_of_one_stem_is_fine() {
    let report = report(
        "egg Midcup",
        &[
            ("Faces/Base/hair.dds", 9),
            ("Faces/Base/hair.xml", 10),
            ("Players/03 - A/Base.face", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    let validated = report.validated.unwrap();
    assert_eq!(validated.players.len(), 1);
    assert_eq!(validated.faces.len(), 1);
}

#[test]
fn a_link_file_below_an_unreserved_subfolder_is_no_link() {
    let report = report_with(
        &context_with(false, false),
        "egg Midcup",
        &[
            ("Players/03 - A/extra/Crocs.boots", 0),
            ("Boots/Crocs/boots.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("file_type_disallowed", Disposition::Keep),
            ("shared_folder_orphaned", Disposition::DropFolder),
        ]
    );
    let player = &report.validated.unwrap().players[0];
    assert!(player.links.is_empty());
}

#[test]
fn a_common_link_in_common_is_no_link_either() {
    let report = report_with(
        &context_with(false, false),
        "egg Midcup",
        &[("Players/03 - A/common/x.fmdl.common", 0)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("file_type_disallowed", Disposition::Keep)]
    );
}

#[test]
fn a_dropped_target_names_its_own_first_finding() {
    let report = report(
        "egg Midcup",
        &[
            ("Players/01 - P/fpc_on", 0),
            ("Players/01 - P/fpc_off", 0),
            ("Players/01 - P/hair.dds", 9),
            ("Players/02 - Q/Base.face", 0),
            ("Players/02 - Q/hair.dds", 9),
            ("Faces/Base/notes.txt", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("fpc_conflict", Disposition::DropFolder),
            ("file_type_disallowed", Disposition::DropFolder),
            ("link_target_dropped", Disposition::DropFolder),
        ]
    );
    assert_eq!(
        report.issues[2].context,
        vec![
            ("link", "Base.face".to_owned()),
            ("target", "Faces/Base".to_owned()),
            ("finding", "file_type_disallowed".to_owned()),
        ]
    );
}

#[test]
fn a_player_dropped_by_its_own_finding_reports_no_dropped_target() {
    let report = report(
        "egg Midcup",
        &[
            ("players.txt", 5),
            ("Players/A/fpc_on", 0),
            ("Players/A/fpc_off", 0),
            ("Players/A/Base.face", 0),
            ("Players/A/hair.dds", 9),
            ("Faces/Base/x.exe", 4),
        ],
        &[],
        &[("players.txt", Ok(b"03 A"))],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("fpc_conflict", Disposition::DropFolder),
            ("file_type_disallowed", Disposition::DropFolder),
        ]
    );
}

fn file_scope(path: &str) -> IssueScope {
    IssueScope::File(ScopePath::new(path).unwrap())
}

#[test]
fn kit_slots_and_labels_parse() {
    let report = report(
        "egg Midcup",
        &[("Kits/p1 - Lakers/kit.dds", 9), ("Kits/g1/kit.dds", 9)],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    let kits = &report.validated.unwrap().kits;
    let lakers = &kits.kits[&kit_config::KitSlot::P1];
    assert_eq!(lakers.path.as_str(), "Kits/p1 - Lakers");
    assert_eq!(lakers.label.as_deref(), Some("Lakers"));
    let goalie = &kits.kits[&kit_config::KitSlot::G1];
    assert_eq!(goalie.path.as_str(), "Kits/g1");
    assert_eq!(goalie.label, None);
}

// TC-KIT-02
#[test]
fn two_folders_of_one_slot_are_both_dropped() {
    let report = report(
        "egg Midcup",
        &[("Kits/p1/kit.dds", 9), ("Kits/p1 - Lakers/kit.dds", 9)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("kit_slot_duplicate", Disposition::DropFolder),
            ("kit_slot_duplicate", Disposition::DropFolder),
        ]
    );
    assert_eq!(report.issues[0].scope, folder("Kits/p1"));
    assert_eq!(report.issues[1].scope, folder("Kits/p1 - Lakers"));
    assert!(report.validated.unwrap().kits.kits.is_empty());
}

// TC-KIT-03
#[test]
fn a_folder_name_with_no_kit_slot_is_invalid() {
    let report = report(
        "egg Midcup",
        &[],
        &["Kits/p10", "Kits/x1", "Kits/home"],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("kit_folder_invalid", Disposition::DropFolder),
            ("kit_folder_invalid", Disposition::DropFolder),
            ("kit_folder_invalid", Disposition::DropFolder),
        ]
    );
    assert_eq!(report.issues[0].scope, folder("Kits/home"));
    assert_eq!(report.issues[1].scope, folder("Kits/p10"));
    assert_eq!(report.issues[2].scope, folder("Kits/x1"));
    assert!(report.validated.unwrap().kits.kits.is_empty());
}

#[test]
fn an_invalid_kit_head_gets_no_own_findings() {
    let report = report("egg Midcup", &[("Kits/x1/back.dds", 9)], &[], &[]);
    assert_eq!(
        issue_codes(&report),
        vec![("kit_folder_invalid", Disposition::DropFolder)]
    );
}

// TC-KIT-04
#[test]
fn a_kit_inherits_only_the_stems_it_lacks() {
    let report = report(
        "egg Midcup",
        &[
            ("Kits/all/kit_back.dds", 9),
            ("Kits/all/kit_name.dds", 9),
            ("Kits/p2/kit_name.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("kit_textures_inherited", Disposition::Keep)]
    );
    assert_eq!(report.issues[0].scope, folder("Kits/p2"));
    assert_eq!(report.issues[0].context, vec![("stems", "back".to_owned())]);
    let kit = &report.validated.unwrap().kits.kits[&kit_config::KitSlot::P2];
    assert_eq!(
        kit.textures
            .iter()
            .map(|texture| (texture.stem.as_str(), texture.source))
            .collect::<Vec<_>>(),
        vec![
            ("kit_back", KitTextureSource::Shared),
            ("kit_name", KitTextureSource::Own),
        ]
    );
}

// TC-KIT-05
#[test]
fn all_files_and_an_unused_all_folder() {
    let ignored = report("egg Midcup", &[("Kits/all/config.toml", 10)], &[], &[]);
    assert_eq!(
        issue_codes(&ignored),
        vec![
            ("kit_all_file_ignored", Disposition::DropFile),
            ("kit_all_unused", Disposition::Keep),
        ]
    );
    assert_eq!(ignored.issues[0].scope, file_scope("Kits/all/config.toml"));

    let unused = report("egg Midcup", &[("Kits/all/kit_back.dds", 9)], &[], &[]);
    assert_eq!(
        issue_codes(&unused),
        vec![("kit_all_unused", Disposition::Keep)]
    );
    assert_eq!(unused.issues[0].scope, folder("Kits/all"));
    assert_eq!(unused.validated.unwrap().kits.shared.len(), 1);
}

// TC-KIT-06
#[test]
fn both_layout_markers_drop_the_kit() {
    let report = report(
        "egg Midcup",
        &[
            ("Kits/p1/kit.dds", 9),
            ("Kits/p1/pre-fox", 0),
            ("Kits/p1/fox", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("kit_layout_conflict", Disposition::DropFolder)]
    );
    assert!(report.validated.unwrap().kits.kits.is_empty());
}

/// The kit an empty folder at `path` gives: nothing of its own, nothing inherited.
fn empty_kit(path: &str) -> KitFolder {
    KitFolder {
        path: ScopePath::new(path).unwrap(),
        label: None,
        config: None,
        colors: None,
        icon: None,
        layout: None,
        textures: Vec::new(),
    }
}

#[test]
fn a_full_export_without_kits_gets_an_empty_p1_and_g1() {
    let report = report(
        "co Full Spring",
        &[("Players/03 - A/face_high.fmdl", 10)],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    let kits = report.validated.unwrap().kits.kits;
    assert_eq!(
        kits,
        BTreeMap::from([
            (kit_config::KitSlot::P1, empty_kit("Kits/p1")),
            (kit_config::KitSlot::G1, empty_kit("Kits/g1")),
        ])
    );
}

#[test]
fn a_full_export_s_empty_kits_inherit_all_s_textures() {
    let report = report(
        "co Full Spring",
        &[
            ("Players/03 - A/face_high.fmdl", 10),
            ("Kits/all/kit.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("kit_textures_inherited", Disposition::Keep),
            ("kit_textures_inherited", Disposition::Keep),
        ],
        "and no kit_all_unused: the empty kits use all/"
    );
    assert_eq!(report.issues[0].scope, folder("Kits/p1"));
    assert_eq!(report.issues[1].scope, folder("Kits/g1"));
    let kits = report.validated.unwrap().kits;
    for slot in [kit_config::KitSlot::P1, kit_config::KitSlot::G1] {
        let kit = &kits.kits[&slot];
        assert_eq!(kit.textures.len(), 1, "{slot:?}");
        assert_eq!(kit.textures[0].stem, "kit");
        assert_eq!(kit.textures[0].file, kits.shared[0]);
        assert_eq!(kit.textures[0].source, KitTextureSource::Shared);
    }
}

#[test]
fn a_full_export_with_a_player_kit_and_g1_gets_no_empty_kit() {
    let report = report(
        "co Full Spring",
        &[("Kits/p2/kit.dds", 9), ("Kits/g1/kit.dds", 9)],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    let kits = report.validated.unwrap().kits.kits;
    assert_eq!(
        kits.keys().copied().collect::<Vec<_>>(),
        [kit_config::KitSlot::P2, kit_config::KitSlot::G1]
    );
    assert_eq!(kits[&kit_config::KitSlot::P2].textures.len(), 1);
}

#[test]
fn a_full_export_whose_only_player_kit_is_dropped_gets_an_empty_p1() {
    let report = report(
        "co Full Spring",
        &[
            ("Kits/p1/kit.dds", 9),
            ("Kits/p1/pre-fox", 0),
            ("Kits/p1/fox", 0),
            ("Kits/g1/kit.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("kit_layout_conflict", Disposition::DropFolder)]
    );
    let kits = report.validated.unwrap().kits.kits;
    assert_eq!(kits[&kit_config::KitSlot::P1], empty_kit("Kits/p1"));
    assert_eq!(kits.len(), 2);
}

#[test]
fn a_midcup_or_referee_export_gets_no_empty_kit() {
    let midcup = report(
        "co Midcup Day 5",
        &[("Players/03 - A/face_high.fmdl", 10)],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&midcup), vec![]);
    assert!(midcup.validated.unwrap().kits.kits.is_empty());

    let referees = report(
        "refs Spring 2026",
        &[("players.txt", 10)],
        &["Players/Keeper"],
        &[("players.txt", Ok(b"01 Keeper"))],
    );
    assert_eq!(issue_codes(&referees), vec![]);
    assert!(referees.validated.unwrap().kits.kits.is_empty());
}

// TC-KIT-07
#[test]
fn a_texture_without_the_kit_prefix_drops_only_itself() {
    let report = report(
        "egg Midcup",
        &[("Kits/p1/back.dds", 9), ("Kits/p1/kit.dds", 9)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("kit_texture_name_invalid", Disposition::DropFile)]
    );
    assert_eq!(report.issues[0].scope, file_scope("Kits/p1/back.dds"));
    let kit = &report.validated.unwrap().kits.kits[&kit_config::KitSlot::P1];
    assert_eq!(kit.textures.len(), 1);
    assert_eq!(kit.textures[0].stem, "kit");
}

#[test]
fn an_icon_marker_names_the_kit_icon() {
    for (marker, icon) in [
        ("icon_7", 7),
        ("icon_07.txt", 7),
        ("icon_0", 0),
        ("icon_23", 23),
    ] {
        let path = format!("Kits/p1/{marker}");
        let report = report(
            "egg Midcup",
            &[(path.as_str(), 0), ("Kits/p1/kit.dds", 9)],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![], "{marker}");
        assert_eq!(
            report.validated.unwrap().kits.kits[&kit_config::KitSlot::P1].icon,
            Some(icon),
            "{marker}"
        );
    }
}

// TC-KIT-08
#[test]
fn an_out_of_range_icon_is_invalid_and_the_kit_kept() {
    for marker in ["icon_24", "icon_25", "icon_99999999999999999999"] {
        let path = format!("Kits/p1/{marker}");
        let report = report(
            "egg Midcup",
            &[(path.as_str(), 0), ("Kits/p1/kit.dds", 9)],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("kit_icon_invalid", Disposition::DropFile)],
            "{marker}"
        );
        assert_eq!(report.issues[0].scope, file_scope(&path));
        let kit = &report.validated.unwrap().kits.kits[&kit_config::KitSlot::P1];
        assert_eq!(kit.icon, None, "{marker}");
    }
}

#[test]
fn two_icon_markers_are_each_invalid() {
    for (first, second) in [("icon_3", "icon_4"), ("icon_3", "icon_3.txt")] {
        let first = format!("Kits/p1/{first}");
        let second = format!("Kits/p1/{second}");
        let report = report(
            "egg Midcup",
            &[
                (first.as_str(), 0),
                (second.as_str(), 0),
                ("Kits/p1/kit.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("kit_icon_invalid", Disposition::DropFile),
                ("kit_icon_invalid", Disposition::DropFile),
            ],
            "{second}"
        );
        assert_eq!(report.issues[0].scope, file_scope(&first));
        assert_eq!(report.issues[1].scope, file_scope(&second));
        let kit = &report.validated.unwrap().kits.kits[&kit_config::KitSlot::P1];
        assert_eq!(kit.icon, None, "{second}");
    }
}

#[test]
fn an_icon_marker_outside_a_kit_folder_is_no_icon() {
    let in_all = report(
        "egg Midcup",
        &[("Kits/all/icon_5", 0), ("Kits/p1/kit.dds", 9)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&in_all),
        vec![("kit_all_file_ignored", Disposition::DropFile)]
    );
    assert_eq!(in_all.issues[0].scope, file_scope("Kits/all/icon_5"));

    // In a player folder an icon marker is a kit marker out of place, as `pre-fox` is.
    let in_player = |marker: &str| {
        let path = format!("Players/03 - A/{marker}");
        let report = report(
            "egg Midcup",
            &[(path.as_str(), 0), ("Players/03 - A/hair.dds", 9)],
            &[],
            &[],
        );
        report
            .issues
            .iter()
            .map(|issue| (issue.code, issue.disposition, issue.context.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        in_player("icon_5"),
        vec![(
            "file_type_disallowed",
            Disposition::DropFolder,
            vec![("file", "icon_5".to_owned())]
        )]
    );
    assert_eq!(
        in_player("pre-fox"),
        vec![(
            "file_type_disallowed",
            Disposition::DropFolder,
            vec![("file", "pre-fox".to_owned())]
        )]
    );
}

// TC-KIT-09
#[test]
fn stem_conflicts_drop_the_folder_they_are_in() {
    let own = report(
        "egg Midcup",
        &[("Kits/p1/kit.png", 9), ("Kits/p1/kit.dds", 9)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&own),
        vec![("texture_stem_conflict", Disposition::DropFolder)]
    );
    assert!(own.validated.unwrap().kits.kits.is_empty());

    let shared = report(
        "egg Midcup",
        &[
            ("Kits/all/kit_back.png", 9),
            ("Kits/all/kit_back.dds", 9),
            ("Kits/p2/kit.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&shared),
        vec![("texture_stem_conflict", Disposition::DropFolder)]
    );
    assert_eq!(shared.issues[0].scope, folder("Kits/all"));
    let kits = &shared.validated.unwrap().kits;
    assert!(kits.shared.is_empty());
    assert_eq!(kits.kits[&kit_config::KitSlot::P2].textures.len(), 1);

    let overridden = report(
        "egg Midcup",
        &[("Kits/all/kit_name.dds", 9), ("Kits/p3/kit_name.png", 9)],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&overridden), vec![]);
}

// TC-ROOT-01
#[test]
fn unexpected_root_files_and_folders_are_dropped() {
    let base = report(
        "egg Midcup",
        &[
            ("extra.bin", 4),
            ("readme.TXT", 20),
            ("Players/players.txt", 10),
            ("Players/03 - A/hair.dds", 9),
        ],
        &["wrapper"],
        &[],
    );
    assert_eq!(
        issue_codes(&base),
        vec![
            ("root_file_unexpected", Disposition::DropFile),
            ("root_file_unexpected", Disposition::DropFolder),
            ("root_file_unexpected", Disposition::DropFile),
        ]
    );
    assert_eq!(base.issues[0].scope, file_scope("extra.bin"));
    assert_eq!(base.issues[1].scope, folder("wrapper"));
    assert_eq!(base.issues[2].scope, file_scope("Players/players.txt"));

    // A referee export's ref_lists.txt is admitted.
    let refs = report(
        "refs Cup",
        &[("players.txt", 10), ("ref_lists.txt", 10)],
        &["Players/Keeper"],
        &[
            ("players.txt", Ok(b"01 Keeper")),
            ("ref_lists.txt", Ok(&b"R1"[..])),
        ],
    );
    assert!(
        refs.issues
            .iter()
            .all(|issue| issue.code != "root_file_unexpected")
    );
}

// TC-ROOT-03
#[test]
fn logo_files_get_one_of_each_role() {
    let invalid = report("egg Midcup", &[("logo_zoom.png", 9)], &[], &[]);
    assert_eq!(
        issue_codes(&invalid),
        vec![("logo_file_invalid", Disposition::DropFile)]
    );
    assert_eq!(invalid.validated.unwrap().logo, None);

    let duplicate = report("egg Midcup", &[("logo.png", 9), ("logo.dds", 9)], &[], &[]);
    assert_eq!(
        issue_codes(&duplicate),
        vec![("logo_role_duplicate", Disposition::DropFile)]
    );
    assert_eq!(duplicate.issues[0].scope, file_scope("logo.png"));
    assert_eq!(
        duplicate.issues[0].context,
        vec![("other", "logo.dds".to_owned())]
    );
    assert_eq!(duplicate.validated.unwrap().logo, None);

    let small_only = report("egg Midcup", &[("logo_small.png", 9)], &[], &[]);
    assert_eq!(
        issue_codes(&small_only),
        vec![("logo_small_without_main", Disposition::DropFile)]
    );
    assert_eq!(small_only.validated.unwrap().logo, None);

    let pair = report(
        "egg Midcup",
        &[("logo.png", 9), ("logo_small.png", 9)],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&pair), vec![]);
    let logo = pair.validated.unwrap().logo.unwrap();
    assert_eq!(logo.main.file.path.as_str(), "logo.png");
    assert_eq!(logo.small.unwrap().file.path.as_str(), "logo_small.png");
}

// TC-ROOT-04
#[test]
fn a_bad_portrait_name_drops_only_that_file() {
    let report = report(
        "egg Midcup",
        &[
            ("Portraits/player_24.dds", 9),
            ("Portraits/player_03.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("portrait_name_invalid", Disposition::DropFile)]
    );
    assert_eq!(
        report.issues[0].scope,
        file_scope("Portraits/player_24.dds")
    );
    assert_eq!(
        report
            .validated
            .unwrap()
            .portraits
            .keys()
            .map(|slot| slot.get())
            .collect::<Vec<_>>(),
        vec![3]
    );
}

#[test]
fn notes_are_read_or_dropped() {
    // A non-empty note is kept and reported.
    let note = report(
        "egg Midcup",
        &[("notes.txt", 12), ("Players/03 - A/hair.dds", 9)],
        &[],
        &[("notes.txt", Ok(b"good cup"))],
    );
    assert_eq!(issue_codes(&note), vec![("notes_found", Disposition::Keep)]);
    assert_eq!(
        note.validated.unwrap().root.notes.unwrap().path.as_str(),
        "notes.txt"
    );

    // Not UTF-8: the note is dropped, the export still valid.
    let bad = report(
        "egg Midcup",
        &[("notes.txt", 4), ("Players/03 - A/hair.dds", 9)],
        &[],
        &[("notes.txt", Ok(&b"\xff\xfe"[..]))],
    );
    assert_eq!(
        issue_codes(&bad),
        vec![("notes_encoding_invalid", Disposition::DropFile)]
    );
    let validated = bad.validated.unwrap();
    assert_eq!(validated.root.notes, None);
    assert_eq!(validated.players.len(), 1);

    // Unreadable: source_read_failed, still validated.
    let denied = report(
        "egg Midcup",
        &[("notes.txt", 5), ("Players/03 - A/hair.dds", 9)],
        &[],
        &[("notes.txt", Err("denied"))],
    );
    assert_eq!(
        issue_codes(&denied),
        vec![("source_read_failed", Disposition::DropFile)]
    );
    assert_eq!(
        denied.issues[0].context,
        vec![("reason", "denied".to_owned())]
    );
    assert!(denied.validated.is_some());

    // Whitespace only: nothing.
    let empty = report(
        "egg Midcup",
        &[("notes.txt", 3), ("Players/03 - A/hair.dds", 9)],
        &[],
        &[("notes.txt", Ok(b"  \n"))],
    );
    assert_eq!(issue_codes(&empty), vec![]);
    assert_eq!(empty.validated.unwrap().root.notes, None);
}

#[test]
fn an_all_folder_beside_only_invalid_kits_is_unused() {
    let report = report("egg Midcup", &[("Kits/all/kit.dds", 9)], &["Kits/x1"], &[]);
    assert_eq!(
        issue_codes(&report),
        vec![
            ("kit_folder_invalid", Disposition::DropFolder),
            ("kit_all_unused", Disposition::Keep),
        ]
    );
    assert_eq!(report.issues[1].scope, folder("Kits/all"));
    assert_eq!(report.validated.unwrap().kits.shared.len(), 1);
}

#[test]
fn two_portraits_of_one_slot_conflict_each() {
    let report = report(
        "egg Midcup",
        &[
            ("Portraits/player_03.dds", 9),
            ("Portraits/player_03.png", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("texture_stem_conflict", Disposition::DropFile),
            ("texture_stem_conflict", Disposition::DropFile),
        ]
    );
    assert_eq!(
        report.issues[0].scope,
        file_scope("Portraits/player_03.dds")
    );
    assert_eq!(
        report.issues[1].scope,
        file_scope("Portraits/player_03.png")
    );
    assert!(report.validated.unwrap().portraits.is_empty());
}

#[test]
fn a_logo_pair_reads_its_fit_tags() {
    let report = report(
        "egg Midcup",
        &[("logo_crop.png", 9), ("logo_small_fit.dds", 9)],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    let logo = report.validated.unwrap().logo.unwrap();
    assert_eq!(logo.main.fit, Some(LogoFit::Crop));
    assert_eq!(logo.small.unwrap().fit, Some(LogoFit::Fit));
}

#[test]
fn a_non_texture_in_a_kit_drops_the_kit_strict() {
    let report = report(
        "egg Midcup",
        &[("Kits/p1/kit.dds", 9), ("Kits/p1/hair.fmdl", 10)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("file_type_disallowed", Disposition::DropFolder)]
    );
    assert_eq!(report.issues[0].scope, folder("Kits/p1"));
    assert!(report.validated.unwrap().kits.kits.is_empty());
}

#[test]
fn a_kit_file_below_a_subfolder_offends() {
    let report = report(
        "egg Midcup",
        &[("Kits/p1/kit.dds", 9), ("Kits/p1/extra/kit_back.dds", 9)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("file_type_disallowed", Disposition::DropFolder)]
    );
    assert_eq!(
        report.issues[0].context,
        vec![("file", "extra/kit_back.dds".to_owned())]
    );
}

#[test]
fn a_kit_s_config_and_colors_below_a_subfolder_are_not_its_own() {
    // Lenient, so the folder stays and its kit can be read.
    let report = report_with(
        &context_with(false, false),
        "egg Midcup",
        &[
            ("Kits/p1/kit.dds", 9),
            ("Kits/p1/sub/config.toml", 9),
            ("Kits/p1/sub/colors.txt", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("file_type_disallowed", Disposition::Keep),
            ("file_type_disallowed", Disposition::Keep),
        ]
    );
    let kit = &report.validated.unwrap().kits.kits[&kit_config::KitSlot::P1];
    assert_eq!(kit.config, None);
    assert_eq!(kit.colors, None);
}

#[test]
fn layout_markers_pick_the_kit_layout() {
    let report = report(
        "egg Midcup",
        &[
            ("Kits/p1/kit.dds", 9),
            ("Kits/p1/fox", 0),
            ("Kits/p2/kit.dds", 9),
            ("Kits/p2/pre-fox", 0),
            ("Kits/p3/kit.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    let kits = &report.validated.unwrap().kits;
    assert_eq!(
        kits.kits[&kit_config::KitSlot::P1].layout,
        Some(KitLayout::Fox)
    );
    assert_eq!(
        kits.kits[&kit_config::KitSlot::P2].layout,
        Some(KitLayout::PreFox)
    );
    assert_eq!(kits.kits[&kit_config::KitSlot::P3].layout, None);
}

#[test]
fn a_portrait_number_must_be_exactly_two_digits() {
    let report = report("egg Midcup", &[("Portraits/player_003.dds", 9)], &[], &[]);
    assert_eq!(
        issue_codes(&report),
        vec![("portrait_name_invalid", Disposition::DropFile)]
    );
    assert!(report.validated.unwrap().portraits.is_empty());
}

#[test]
fn a_logo_candidate_that_is_no_texture_is_invalid() {
    let report = report(
        "egg Midcup",
        &[("logo.txt", 9), ("Players/03 - A/hair.dds", 9)],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("logo_file_invalid", Disposition::DropFile)]
    );
    assert_eq!(report.validated.unwrap().logo, None);
}

#[test]
fn a_root_colors_file_is_the_team_colors() {
    let report = report(
        "egg Midcup",
        &[("colors.txt", 10), ("Players/03 - A/hair.dds", 9)],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    assert_eq!(
        report
            .validated
            .unwrap()
            .root
            .team_colors
            .unwrap()
            .path
            .as_str(),
        "colors.txt"
    );
}

#[test]
fn referee_files_are_unexpected_on_a_team_export() {
    let report = report(
        "egg Midcup",
        &[
            ("refs.txt", 10),
            ("ref_lists.txt", 10),
            ("ref_marker.dds", 9),
            ("Players/03 - A/hair.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("root_file_unexpected", Disposition::DropFile),
            ("root_file_unexpected", Disposition::DropFile),
            ("root_file_unexpected", Disposition::DropFile),
        ]
    );
    assert_eq!(report.issues[0].scope, file_scope("ref_lists.txt"));
    assert_eq!(report.issues[1].scope, file_scope("ref_marker.dds"));
    assert_eq!(report.issues[2].scope, file_scope("refs.txt"));
    assert_eq!(report.validated.unwrap().root.referee_marker, None);
}

#[test]
fn a_ref_marker_is_admitted_on_a_refs_export() {
    let report = report(
        "refs Cup",
        &[("players.txt", 10), ("ref_marker.dds", 9)],
        &["Players/Keeper"],
        &[("players.txt", Ok(b"01 Keeper"))],
    );
    assert!(
        report
            .issues
            .iter()
            .all(|issue| issue.code != "root_file_unexpected")
    );
    assert_eq!(
        report
            .validated
            .unwrap()
            .root
            .referee_marker
            .unwrap()
            .path
            .as_str(),
        "ref_marker.dds"
    );
}

#[test]
fn a_stem_ending_in_space_folds_without_panicking() {
    // `skin .png`'s stem is `skin ` — no valid path segment, still a name.
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/skin .png", 9),
            ("Players/03 - A/face_high.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&report), vec![]);
    assert_eq!(report.validated.unwrap().players.len(), 1);
}

#[test]
fn a_link_name_ending_in_space_misses_its_target() {
    // `Crocs .boots` names `Crocs ` — `Boots/Crocs` exists but does not
    // match; the name folds rather than panicking.
    let report = report(
        "egg Midcup",
        &[
            ("Players/03 - A/Crocs .boots", 0),
            ("Boots/Crocs/boots.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("link_target_missing", Disposition::DropFolder),
            ("shared_folder_orphaned", Disposition::DropFolder),
        ]
    );
}

#[test]
fn a_flattened_folder_spelling_collision_is_a_conflict() {
    // `Docs/b.txt` flattens onto `docs/`'s spelling — rejected, no panic.
    let report = report(
        "egg Midcup",
        &[
            ("docs/a.txt", 4),
            ("wrapper/Docs/b.txt", 4),
            ("wrapper/Players/03 - A/face_high.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("nested_root_conflict", Disposition::DropExport)]
    );
    assert!(report.validated.is_none());
}

#[test]
fn a_texture_common_link_joins_the_stem_namespace() {
    let linked = report(
        "egg Midcup",
        &[
            ("Common/hair.png", 9),
            ("Players/03 - A/hair.dds", 9),
            ("Players/03 - A/hair.png.common", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&linked),
        vec![("texture_stem_conflict", Disposition::DropFolder)]
    );
    assert_eq!(linked.issues[0].scope, folder("Players/03 - A"));
    assert!(linked.validated.unwrap().players.is_empty());

    // A model link is not a texture: same stem, no conflict.
    let model_link = report(
        "egg Midcup",
        &[
            ("Common/torso.fmdl", 10),
            ("Players/03 - A/torso.dds", 9),
            ("Players/03 - A/torso.fmdl.common", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&model_link), vec![]);
}

#[test]
fn common_textures_conflict_each_other_and_drop_the_linker() {
    let report = report(
        "egg Midcup",
        &[
            ("Common/hair.dds", 9),
            ("Common/hair.png", 9),
            ("Players/03 - A/hair.png.common", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("texture_stem_conflict", Disposition::DropFile),
            ("texture_stem_conflict", Disposition::DropFile),
            ("link_target_dropped", Disposition::DropFolder),
        ]
    );
    assert_eq!(report.issues[0].scope, file_scope("Common/hair.dds"));
    assert_eq!(report.issues[1].scope, file_scope("Common/hair.png"));
    assert_eq!(report.issues[2].scope, folder("Players/03 - A"));
    let validated = report.validated.unwrap();
    assert!(validated.common.is_empty());
    assert!(validated.players.is_empty());
}

#[test]
fn two_collisions_on_one_loose_path_report_once() {
    // Both flattened names claim `docs/`'s spelling: one finding.
    let report = report(
        "egg Midcup",
        &[
            ("docs/a.txt", 4),
            ("wrapper/Docs/b.txt", 4),
            ("wrapper/Docs/c.txt", 4),
            ("wrapper/Players/03 - A/face_high.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("nested_root_conflict", Disposition::DropExport)]
    );
    assert!(report.validated.is_none());
}

#[test]
fn a_common_link_below_common_is_no_texture() {
    // `common/` is not a link position: the file is disallowed there but
    // never joins the stem namespace.
    let report = report(
        "egg Midcup",
        &[
            ("Common/hair.png", 9),
            ("Players/03 - A/hair.dds", 9),
            ("Players/03 - A/common/hair.png.common", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("file_type_disallowed", Disposition::DropFolder)]
    );
}

#[test]
fn a_kept_disallowed_common_file_stays_in_the_players_files() {
    // A `.common` file below `common/` is never a link position: its
    // disallowed finding keeps it (strict off), so it stays in `files`.
    let report = report_with(
        &context_with(false, false),
        "egg Midcup",
        &[
            ("Common/torso.fmdl", 10),
            ("Players/03 - A/face_high.fmdl", 10),
            ("Players/03 - A/common/torso.fmdl.common", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("file_type_disallowed", Disposition::Keep)]
    );
    let player = &report.validated.unwrap().players[0];
    assert!(
        player
            .files
            .iter()
            .any(|f| f.path.as_str() == "Players/03 - A/common/torso.fmdl.common"),
        "{:?}",
        player
            .files
            .iter()
            .map(|f| f.path.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn an_undecided_root_reports_no_logo_finding() {
    let report = report(
        "egg Midcup",
        &[
            ("logo.txt", 5),
            ("a/Players/03 - A/face_high.fmdl", 10),
            ("b/Players/03 - A/face_high.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("nested_root_ambiguous", Disposition::DropExport)]
    );
    assert!(report.validated.is_none());
}

#[test]
fn an_empty_doubled_players_layer_is_dropped_with_its_layer() {
    // `Players/Players/` alone under `Players/` is the doubled layer's own shell:
    // gone with the layer, not a folder named `Players` claiming a slot.
    let report = report("egg Midcup", &[], &["Players", "Players/Players"], &[]);
    assert_eq!(
        issue_codes(&report),
        vec![
            ("nested_folders_fixed", Disposition::Keep),
            ("export_empty", Disposition::DropExport),
        ]
    );
    assert!(report.validated.is_none());
    assert!(
        report
            .parsed
            .draft
            .players
            .iter()
            .all(|folder| folder.path.name() != "Players"),
        "{:?}",
        report
            .parsed
            .draft
            .players
            .iter()
            .map(|folder| folder.path.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn an_empty_doubled_kits_layer_is_dropped_with_its_layer() {
    let report = report("egg Midcup", &[], &["Kits", "Kits/Kits"], &[]);
    assert_eq!(
        issue_codes(&report),
        vec![
            ("nested_folders_fixed", Disposition::Keep),
            ("export_empty", Disposition::DropExport),
        ]
    );
    assert!(report.validated.is_none());
    assert!(
        report
            .parsed
            .draft
            .kits
            .iter()
            .all(|folder| folder.path.name() != "Kits")
    );
}

#[test]
fn an_empty_doubled_folder_beside_siblings_is_an_invalid_folder() {
    // `Players/` holding `Players/` *and* a real player: `Players/Players/` is not
    // the doubled layer (it is not the only entry) — it is a folder named
    // `Players`, an invalid player name, like a `Kits/Kits/` beside `p1`.
    let players = report(
        "egg Midcup",
        &[("Players/03 - A/face_high.fmdl", 10)],
        &["Players/Players"],
        &[],
    );
    assert_eq!(
        issue_codes(&players),
        vec![("player_folder_number_invalid", Disposition::DropFolder)]
    );
    assert_eq!(
        players.validated.unwrap().players[0].path.as_str(),
        "Players/03 - A"
    );

    let kits = report("egg Midcup", &[("Kits/p1/kit.dds", 9)], &["Kits/Kits"], &[]);
    assert_eq!(
        issue_codes(&kits),
        vec![("kit_folder_invalid", Disposition::DropFolder)]
    );
    assert_eq!(
        kits.validated.unwrap().kits.kits.keys().collect::<Vec<_>>(),
        vec![&kit_config::KitSlot::P1]
    );
}

#[test]
fn an_undecided_root_reports_no_roster_finding() {
    // A referee export whose two usable children both hold their own roster and
    // players: the ambiguity leaves the root undecided, so no `players.txt`
    // finding stands beside it.
    let ambiguous = report(
        "refs Cup",
        &[
            ("a/refs.txt", 10),
            ("a/Players/Keeper/face_high.fmdl", 10),
            ("b/refs.txt", 10),
            ("b/Players/Other/face_high.fmdl", 10),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&ambiguous),
        vec![("nested_root_ambiguous", Disposition::DropExport)]
    );

    // A loose `players.txt` colliding with the flattened one is a conflict; the
    // loose file's lines name no folder that exists, but no roster finding is
    // reported beside the conflict.
    let conflicted = report(
        "egg Midcup",
        &[
            ("players.txt", 6),
            ("wrapper/players.txt", 6),
            ("wrapper/Players/03 - A/face_high.fmdl", 10),
        ],
        &[],
        &[("players.txt", Ok(b"03 A"))],
    );
    assert_eq!(
        issue_codes(&conflicted),
        vec![("nested_root_conflict", Disposition::DropExport)]
    );

    // A `players.txt` at the undecided root that is not UTF-8: its own finding is
    // a roster finding, reported nowhere either.
    let invalid = report(
        "refs Cup",
        &[
            ("players.txt", 4),
            ("a/refs.txt", 10),
            ("a/Players/Keeper/face_high.fmdl", 10),
            ("b/refs.txt", 10),
            ("b/Players/Other/face_high.fmdl", 10),
        ],
        &[],
        &[("players.txt", Ok(&[0xff, 0xfe][..]))],
    );
    assert_eq!(
        issue_codes(&invalid),
        vec![("nested_root_ambiguous", Disposition::DropExport)]
    );
}

#[test]
fn a_flattened_wrapper_of_empty_folders_leaves_no_root_folder() {
    let report = report(
        "egg Midcup",
        &[],
        &["wrapper", "wrapper/Kits", "wrapper/Kits/p1"],
        &[],
    );
    assert_eq!(
        issue_codes(&report),
        vec![("nested_folders_fixed", Disposition::Keep)]
    );
    assert!(report.parsed.draft.root_folders.is_empty());
    assert_eq!(
        report
            .validated
            .unwrap()
            .kits
            .kits
            .keys()
            .collect::<Vec<_>>(),
        vec![&kit_config::KitSlot::P1]
    );
}

/// The default context with the target `version`.
fn context_for(version: pes_version::PesVersion) -> ValidationContext {
    ValidationContext {
        version,
        strict_file_type_check: true,
        pass_through: false,
        installed_common_textures: std::collections::BTreeSet::new(),
    }
}

#[test]
fn a_pre_fox_shared_boots_model_without_its_suffix_is_model_name_invalid() {
    let unsuffixed = report_with(
        &context_for(pes_version::PesVersion::Pes17),
        "egg Midcup",
        &[
            ("Boots/Mud/hat.model", 10),
            ("Players/03 - A/Mud.boots", 0),
            ("Players/03 - A/hair.dds", 9),
        ],
        &[],
        &[],
    );
    assert_eq!(
        issue_codes(&unsuffixed),
        vec![
            ("model_name_invalid", Disposition::DropFolder),
            ("link_target_dropped", Disposition::DropFolder),
        ]
    );
    assert_eq!(unsuffixed.issues[0].scope, folder("Boots/Mud"));
    assert_eq!(
        unsuffixed.issues[0].context,
        vec![("file", "hat.model".to_owned())]
    );
    assert_eq!(unsuffixed.issues[1].scope, folder("Players/03 - A"));
    assert!(unsuffixed.validated.unwrap().boots.is_empty());

    let suffixed = report_with(
        &context_for(pes_version::PesVersion::Pes17),
        "egg Midcup",
        &[
            ("Boots/Mud/kit_boots.model", 10),
            ("Players/03 - A/Mud.boots", 0),
        ],
        &[],
        &[],
    );
    assert_eq!(issue_codes(&suffixed), vec![]);
    assert_eq!(suffixed.validated.unwrap().boots.len(), 1);
}

#[test]
fn an_edit_hair_file_drops_its_folder_on_pre_fox_only() {
    let files = [
        ("Players/05 - A/face_edithair.xml", 10),
        ("Players/05 - A/face/hair.xml", 10),
        ("Players/05 - A/face_high.model", 10),
        ("Players/07 - B/Longhair.face", 0),
        ("Players/07 - B/hair.dds", 9),
        ("Faces/Longhair/HAIR.XML", 10),
        ("Faces/Longhair/hair_high.model", 10),
    ];
    let pre_fox = report_with(
        &context_for(pes_version::PesVersion::Pes17),
        "egg Midcup",
        &files,
        &[],
        &[],
    );
    let edithair = |scope: &str, file: &str| ValidationIssue {
        code: "edithair_unsupported",
        scope: folder(scope),
        context: vec![("file", file.to_owned())],
        disposition: Disposition::DropFolder,
        passed_through: false,
    };
    assert_eq!(
        pre_fox.issues[..3],
        // In the folder's file order: `/` sorts before `_`.
        [
            edithair("Players/05 - A", "face/hair.xml"),
            edithair("Players/05 - A", "face_edithair.xml"),
            edithair("Faces/Longhair", "HAIR.XML"),
        ]
    );
    assert_eq!(
        issue_codes(&pre_fox)[3..],
        [("link_target_dropped", Disposition::DropFolder)]
    );
    assert_eq!(pre_fox.issues[3].scope, folder("Players/07 - B"));
    let validated = pre_fox.validated.unwrap();
    assert!(validated.players.is_empty());
    assert!(validated.faces.is_empty());

    let fox = report_with(
        &context_for(pes_version::PesVersion::Pes21),
        "egg Midcup",
        &files,
        &[],
        &[],
    );
    assert_eq!(issue_codes(&fox), vec![]);
    let validated = fox.validated.unwrap();
    assert_eq!(validated.players.len(), 2);
    assert_eq!(validated.faces.len(), 1);
}

/// A `vertex_too_far_from_origin` content finding on `scope` (the consumer's code, not one of
/// `ISSUE_CODES`), never pass-through-eligible.
fn far_vertex(scope: IssueScope, disposition: Disposition) -> ContentFinding {
    ContentFinding {
        code: "vertex_too_far_from_origin",
        scope,
        context: vec![("file", "boots.fmdl".to_owned()), ("count", "1".to_owned())],
        disposition,
        pass_through_eligible: false,
    }
}

/// `far_vertex`'s issue, as validation reports it when nothing keeps the item.
fn far_vertex_issue(scope: IssueScope, disposition: Disposition) -> ValidationIssue {
    ValidationIssue {
        code: "vertex_too_far_from_origin",
        scope,
        context: vec![("file", "boots.fmdl".to_owned()), ("count", "1".to_owned())],
        disposition,
        passed_through: false,
    }
}

/// `report`'s export with `findings` added, with the default context.
fn with_findings(files: &[(&str, u64)], findings: Vec<ContentFinding>) -> ValidationReport {
    report("egg Midcup", files, &[], &[])
        .with_content_findings(findings, &crate::testing::context())
}

/// The paths of the validated players.
fn player_paths(validated: &ValidatedAestheticsExport) -> Vec<&str> {
    validated
        .players
        .iter()
        .map(|player| player.path.as_str())
        .collect()
}

#[test]
fn no_content_findings_leave_the_report_as_validate_made_it() {
    // A parse issue (the nested root), structure issues and a cascade: none is doubled.
    let files = &[
        ("wrapper/Players/03 - A/boots.fmdl", 10),
        ("wrapper/Players/03 - A/readme.txt", 1),
        ("wrapper/Players/05 - B/Crocs.boots", 0),
        ("wrapper/Boots/Crocs/x.exe", 1),
        ("wrapper/Common/hair.dds", 9),
        ("wrapper/Kits/p1/kit.dds", 9),
    ];
    let first = report("egg Midcup", files, &[], &[]);
    assert_eq!(
        issue_codes(&first),
        vec![
            ("nested_folders_fixed", Disposition::Keep),
            ("file_type_disallowed", Disposition::DropFolder),
            ("file_type_disallowed", Disposition::DropFolder),
            ("link_target_dropped", Disposition::DropFolder),
        ]
    );
    let again = first
        .clone()
        .with_content_findings(Vec::new(), &crate::testing::context());
    assert_eq!(again, first);
}

#[test]
fn a_content_finding_dropping_a_player_folder_takes_it_off_players_and_the_roster() {
    let report = with_findings(
        &[
            ("Players/03 - A/boots.fmdl", 10),
            ("Players/05 - B/boots.fmdl", 10),
        ],
        vec![far_vertex(
            folder("Players/03 - A"),
            Disposition::DropFolder,
        )],
    );
    assert_eq!(
        report.issues,
        vec![far_vertex_issue(
            folder("Players/03 - A"),
            Disposition::DropFolder
        )]
    );
    let validated = report.validated.unwrap();
    assert_eq!(player_paths(&validated), vec!["Players/05 - B"]);
    let ValidatedRoster::Team(map) = &validated.roster else {
        panic!("a team export");
    };
    assert_eq!(
        map.iter()
            .map(|(slot, index)| (slot.get(), index.0))
            .collect::<Vec<_>>(),
        vec![(5, 0)]
    );
}

#[test]
fn a_content_finding_dropping_a_shared_folder_cascades_to_its_linkers_and_orphans() {
    let report = with_findings(
        &[
            ("Boots/Crocs/boots.fmdl", 10),
            ("Gloves/Grip/glove_l.fmdl", 10),
            ("Players/03 - A/Crocs.boots", 0),
            ("Players/03 - A/Grip.gloves", 0),
            ("Players/05 - B/boots.fmdl", 10),
        ],
        vec![far_vertex(folder("Boots/Crocs"), Disposition::DropFolder)],
    );
    assert_eq!(
        report.issues,
        vec![
            far_vertex_issue(folder("Boots/Crocs"), Disposition::DropFolder),
            ValidationIssue {
                code: "link_target_dropped",
                scope: folder("Players/03 - A"),
                context: vec![
                    ("link", "Crocs.boots".to_owned()),
                    ("target", "Boots/Crocs".to_owned()),
                    ("finding", "vertex_too_far_from_origin".to_owned()),
                ],
                disposition: Disposition::DropFolder,
                passed_through: false,
            },
            ValidationIssue {
                code: "shared_folder_orphaned",
                scope: folder("Gloves/Grip"),
                context: vec![],
                disposition: Disposition::DropFolder,
                passed_through: false,
            },
        ]
    );
    let validated = report.validated.unwrap();
    assert_eq!(player_paths(&validated), vec!["Players/05 - B"]);
    assert!(validated.boots.is_empty());
    assert!(validated.gloves.is_empty());
}

#[test]
fn a_content_finding_dropping_a_common_file_drops_the_player_linking_it() {
    let report = with_findings(
        &[
            ("Common/legs.fmdl", 10),
            ("Common/hair.dds", 9),
            ("Players/03 - A/legs.fmdl.common", 0),
            ("Players/05 - B/boots.fmdl", 10),
        ],
        vec![far_vertex(
            file_scope("Common/legs.fmdl"),
            Disposition::DropFile,
        )],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("vertex_too_far_from_origin", Disposition::DropFile),
            ("link_target_dropped", Disposition::DropFolder),
        ]
    );
    assert_eq!(report.issues[1].scope, folder("Players/03 - A"));
    assert_eq!(
        report.issues[1].context,
        vec![
            ("link", "legs.fmdl.common".to_owned()),
            ("target", "Common/legs.fmdl".to_owned()),
            ("finding", "vertex_too_far_from_origin".to_owned()),
        ]
    );
    let validated = report.validated.unwrap();
    assert_eq!(player_paths(&validated), vec!["Players/05 - B"]);
    assert_eq!(
        validated
            .common
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        vec!["Common/hair.dds"]
    );
}

#[test]
fn pass_through_keeps_an_eligible_content_finding_s_folder_and_drops_a_not_eligible_one() {
    let context = context_with(true, true);
    let eligible = ContentFinding {
        pass_through_eligible: true,
        ..far_vertex(folder("Players/03 - A"), Disposition::DropFolder)
    };
    let report = report_with(
        &context,
        "egg Midcup",
        &[
            ("Players/03 - A/boots.fmdl", 10),
            ("Players/05 - B/boots.fmdl", 10),
        ],
        &[],
        &[],
    )
    .with_content_findings(
        vec![
            eligible,
            far_vertex(folder("Players/05 - B"), Disposition::DropFolder),
        ],
        &context,
    );
    assert_eq!(
        report.issues,
        vec![
            ValidationIssue {
                passed_through: true,
                ..far_vertex_issue(folder("Players/03 - A"), Disposition::Keep)
            },
            far_vertex_issue(folder("Players/05 - B"), Disposition::DropFolder),
        ]
    );
    assert_eq!(
        player_paths(&report.validated.unwrap()),
        vec!["Players/03 - A"]
    );
}

#[test]
fn a_content_finding_dropping_a_kit_folder_takes_the_kit_off() {
    let report = with_findings(
        &[("Kits/p1/kit.dds", 9), ("Kits/p2 - Away/kit.dds", 9)],
        vec![far_vertex(folder("Kits/p1"), Disposition::DropFolder)],
    );
    let validated = report.validated.unwrap();
    assert_eq!(
        validated
            .kits
            .kits
            .values()
            .map(|kit| kit.path.as_str())
            .collect::<Vec<_>>(),
        vec!["Kits/p2 - Away"]
    );
}

#[test]
fn a_content_finding_dropping_a_portrait_or_collar_file_takes_only_that_file_off() {
    let report = with_findings(
        &[
            ("Players/03 - A/boots.fmdl", 10),
            ("Portraits/player_03.dds", 9),
            ("Portraits/player_05.dds", 9),
            ("Collars/collar_101.fmdl", 10),
            ("Collars/collar_102.fmdl", 10),
        ],
        vec![
            far_vertex(file_scope("Portraits/player_03.dds"), Disposition::DropFile),
            far_vertex(file_scope("Collars/collar_101.fmdl"), Disposition::DropFile),
        ],
    );
    let validated = report.validated.unwrap();
    assert_eq!(
        validated
            .portraits
            .iter()
            .map(|(slot, file)| (slot.get(), file.path.as_str()))
            .collect::<Vec<_>>(),
        vec![(5, "Portraits/player_05.dds")]
    );
    assert_eq!(
        validated
            .collars
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        vec!["Collars/collar_102.fmdl"]
    );
}

#[test]
fn a_content_finding_dropping_a_player_s_settings_or_portrait_keeps_the_folder_without_it() {
    let files = &[
        ("Players/03 - A/boots.fmdl", 10),
        ("Players/03 - A/portrait.png", 9),
        ("Players/03 - A/settings.toml", 20),
    ];
    let without_settings = with_findings(
        files,
        vec![far_vertex(
            file_scope("Players/03 - A/settings.toml"),
            Disposition::DropFile,
        )],
    );
    let player = &without_settings.validated.unwrap().players[0];
    assert_eq!(player.settings, None);
    assert_eq!(
        player.portrait.as_ref().map(|file| file.path.as_str()),
        Some("Players/03 - A/portrait.png")
    );
    assert_eq!(
        player
            .files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        vec!["Players/03 - A/boots.fmdl"]
    );

    let without_portrait = with_findings(
        files,
        vec![far_vertex(
            file_scope("Players/03 - A/portrait.png"),
            Disposition::DropFile,
        )],
    );
    let player = &without_portrait.validated.unwrap().players[0];
    assert_eq!(player.portrait, None);
    assert_eq!(
        player.settings.as_ref().map(|file| file.path.as_str()),
        Some("Players/03 - A/settings.toml")
    );
}

#[test]
fn a_content_finding_dropping_a_colors_file_takes_only_that_file_off() {
    let files = &[
        ("Players/03 - A/boots.fmdl", 10),
        ("Kits/p1/kit.dds", 9),
        ("Kits/p1/colors.txt", 16),
        ("colors.txt", 16),
    ];
    let without_kit_colors = with_findings(
        files,
        vec![far_vertex(
            file_scope("Kits/p1/colors.txt"),
            Disposition::DropFile,
        )],
    )
    .validated
    .unwrap();
    let kit = &without_kit_colors.kits.kits[&kit_config::KitSlot::P1];
    assert_eq!(kit.colors, None);
    assert_eq!(
        kit.textures
            .iter()
            .map(|texture| texture.file.path.as_str())
            .collect::<Vec<_>>(),
        vec!["Kits/p1/kit.dds"]
    );
    assert_eq!(
        without_kit_colors
            .root
            .team_colors
            .as_ref()
            .map(|file| file.path.as_str()),
        Some("colors.txt")
    );

    let without_team_colors = with_findings(
        files,
        vec![far_vertex(file_scope("colors.txt"), Disposition::DropFile)],
    )
    .validated
    .unwrap();
    assert_eq!(without_team_colors.root.team_colors, None);
    assert_eq!(
        without_team_colors.kits.kits[&kit_config::KitSlot::P1]
            .colors
            .as_ref()
            .map(|file| file.path.as_str()),
        Some("Kits/p1/colors.txt")
    );
}

#[test]
fn a_content_finding_dropping_either_logo_file_drops_the_logo() {
    let files = &[
        ("Players/03 - A/boots.fmdl", 10),
        ("logo.png", 9),
        ("logo_small.png", 9),
    ];
    assert!(
        report("egg Midcup", files, &[], &[])
            .validated
            .unwrap()
            .logo
            .is_some()
    );
    for name in ["logo.png", "logo_small.png"] {
        let report = with_findings(
            files,
            vec![far_vertex(file_scope(name), Disposition::DropFile)],
        );
        assert_eq!(report.validated.unwrap().logo, None, "{name}");
    }
}

#[test]
fn a_content_finding_dropping_the_export_leaves_no_sanitized_export() {
    let report = with_findings(
        &[("Players/03 - A/boots.fmdl", 10)],
        vec![far_vertex(IssueScope::Export, Disposition::DropExport)],
    );
    assert!(report.validated.is_none());
}

#[test]
fn content_findings_stand_after_the_folders_own_findings_and_before_the_cascade() {
    let report = with_findings(
        &[
            ("Boots/Crocs/boots.fmdl", 10),
            ("Players/03 - A/Crocs.boots", 0),
            ("Players/07 - C/readme.txt", 1),
            ("Kits/p1/kit.dds", 9),
            ("Kits/p1/icon_99", 0),
        ],
        vec![far_vertex(folder("Boots/Crocs"), Disposition::DropFolder)],
    );
    assert_eq!(
        issue_codes(&report),
        vec![
            ("file_type_disallowed", Disposition::DropFolder),
            ("vertex_too_far_from_origin", Disposition::DropFolder),
            ("link_target_dropped", Disposition::DropFolder),
            ("kit_icon_invalid", Disposition::DropFile),
        ]
    );
}
