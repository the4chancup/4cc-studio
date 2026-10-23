//! Opening and saving: a file Save wrote opens with no banner
//! (RE-FILE-01), a hand-edited logic shows the banner and is kept
//! byte for byte (RE-FILE-02), comments inside `RULESET` go at save
//! (RE-FILE-05), and a failed write is reported with nothing beside
//! the file (RE-FILE-07).

use super::*;

/// The modified-logic file: the official text with one logic comment
/// changed (the file still loads, its logic is `Modified`).
fn modified_logic_file() -> String {
    official_text().replacen(
        "// The generic interpreter",
        "// The interpreter, its comment edited",
        1,
    )
}

// RE-FILE-01
#[test]
fn a_file_save_wrote_opens_with_no_banner() {
    let (ctx, _events, path, mut view) = official_opened("file_01");
    // Save is available on an unedited document.
    assert!(save_to(&mut view, &ctx, &path));

    let mut view = opened_view(&ctx, &path);
    let mut window = Headless::new();
    let frame = window.draw(&mut view, &ctx);
    assert!(
        view.document
            .as_ref()
            .is_some_and(|document| { matches!(document.logic, crate::document::Logic::Current) }),
        "the file opens in the forms with the current logic"
    );
    assert!(!contains_label(&frame.nodes, "checking code"), "no banner");
}

// RE-FILE-02
#[test]
fn a_modified_logic_file_shows_the_banner_and_save_keeps_its_code() {
    let dir = crate::test_support::scratch("file_02");
    let text = modified_logic_file();
    let kept = aatf::file_sections(&text).logic.expect("the section");
    let path = write_file(&dir, "rules.rhai", &text);
    let (ctx, _events, mut window, mut view) = fresh();
    open_path(&mut view, &ctx, &path);
    let frame = window.draw(&mut view, &ctx);
    assert!(
        contains_label(&frame.nodes, "checking code was changed by hand"),
        "the banner says the file's checking code is not this build's"
    );

    // An edit (General's name) and Save: the file's logic kept byte for
    // byte, the RULESET holding the edit.
    edit_ruleset(&mut view, |ruleset| ruleset.name = "Renamed".to_string());
    assert!(save_to(&mut view, &ctx, &path));
    let written = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        aatf::file_sections(&written)
            .logic
            .as_deref()
            .map(str::trim_end),
        Some(kept.trim_end()),
        "the file's own logic, byte for byte"
    );
    let rules = aatf::Rules::load(&written).expect("the saved file loads");
    assert_eq!(rules.ruleset().name, "Renamed", "the edit is in it");

    // Reopened: the banner again.
    let mut view = opened_view(&ctx, &path);
    let frame = window.draw(&mut view, &ctx);
    assert!(
        contains_label(&frame.nodes, "checking code was changed by hand"),
        "the banner again"
    );
}

// RE-FILE-05
#[test]
fn a_save_with_no_edit_writes_the_same_values_and_no_comments() {
    // The DBC7 block with the comments inside its RULESET, assembled as
    // a file (current logic, the empty-checks stub).
    let aatf_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/aatf");
    let block = std::fs::read_to_string(aatf_dir.join("rulesets/dbc7.rhai")).unwrap();
    assert!(
        block.contains("// No card or position rule"),
        "premise: comments inside RULESET"
    );
    let logic = aatf::file_sections(&official_text())
        .logic
        .expect("the current section");
    let stub = "// ==== CUSTOM CHECKS: optional, hand-written; the Ruleset editor keeps this section as it is ====\nfn custom_checks(team) { [] }";
    let text = format!("{}\n\n{}\n\n{}\n", block.trim_end(), logic.trim_end(), stub);
    let dir = crate::test_support::scratch("file_05");
    let path = write_file(&dir, "dbc7.rhai", &text);

    let (ctx, _events) = crate::test_support::context();
    let mut view = opened_view(&ctx, &path);
    assert!(save_to(&mut view, &ctx, &path));
    let written = std::fs::read_to_string(&path).unwrap();
    let data = &written[..written.find("// ==== LOGIC").expect("the logic banner")];
    assert!(
        !data.contains("// No card or position rule"),
        "none of the comments is written"
    );
    let rules = aatf::Rules::load(&written).expect("the saved file loads");
    assert_eq!(
        rules.ruleset(),
        &aatf::Ruleset::from_data_block(&block).expect("the block reads"),
        "the ruleset reads back equal"
    );
}

// RE-FILE-07
#[test]
fn save_leaves_no_sibling_and_reports_a_read_only_file() {
    let (ctx, _events, path, mut view) = official_opened("file_07");
    edit_ruleset(&mut view, |ruleset| ruleset.name = "Renamed".to_string());
    assert!(save_to(&mut view, &ctx, &path));
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    assert!(
        !Path::new(&tmp).exists(),
        "the temporary sibling renamed into place"
    );

    // The read-only leg: the write is refused, the file is byte for
    // byte as it was, nothing is left beside it, the document stays
    // dirty.
    let before = std::fs::read(&path).unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&path, permissions).unwrap();
    edit_ruleset(&mut view, |ruleset| {
        ruleset.name = "Renamed again".to_string()
    });
    let saved = save_to(&mut view, &ctx, &path);
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    #[expect(
        clippy::permissions_set_readonly_false,
        reason = "a test fixture must get its writable bit back or the scratch folder cannot be cleared"
    )]
    permissions.set_readonly(false);
    std::fs::set_permissions(&path, permissions).unwrap();
    assert!(!saved, "Save reports the failure");
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "byte for byte as it was"
    );
    assert!(!Path::new(&tmp).exists(), "no other file beside it");
    assert!(
        view.document.as_ref().expect("the document").is_dirty(),
        "the document stays unsaved"
    );
    let mut window = Headless::new();
    let frame = window.draw(&mut view, &ctx);
    assert!(
        labels(&frame.nodes)
            .iter()
            .any(|label| label.contains("rules.rhai") && label.contains("read-only")),
        "the window names the file and the error"
    );
}

// plans/ruleset_editor.md "Settings": a table this build cannot read is
// not written over (its recent files stay as they were, not emptied).
#[test]
fn a_broken_settings_table_is_not_written_over() {
    let (_dir, path) = scratch_file("settings_broken", "rules.rhai");
    let (ctx, _events) = crate::test_support::context();
    let mut broken = toml::Table::new();
    broken.insert("recent_files".to_string(), toml::Value::Integer(3));
    ctx.set_tool_settings(crate::messages::TOOL_ID, broken);
    let mut view = RulesetView::new();
    open_path(&mut view, &ctx, &path);
    assert!(view.document.is_some(), "the file still opens");
    let table = ctx.tool_settings(crate::messages::TOOL_ID);
    assert_eq!(
        table["recent_files"],
        toml::Value::Integer(3),
        "the table is left as it was"
    );
}
