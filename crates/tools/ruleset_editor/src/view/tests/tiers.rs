//! The Tiers section (RE-GUI-03 to RE-GUI-06, RE-FILE-04 and
//! RE-FILE-06): a rate edit shows in the preview and dirties the
//! document, a handle drag reorders the tiers, a shared id marks the
//! rail and blocks Save, the medal checkbox moves a tier's count into
//! the systems and back; and the per-stat, free-count, id-rename,
//! colour-alpha and removal behaviour the scenarios do not reach.

use std::collections::BTreeMap;

use aatf::Ruleset;

use super::*;

/// The embedded ruleset `name` gives (the start page's templates).
fn template(name: &str) -> Ruleset {
    aatf::templates()
        .into_iter()
        .find(|template| template.ruleset.name == name)
        .unwrap_or_else(|| panic!("a template named {name}"))
        .ruleset
}

/// The document's ruleset.
fn ruleset(view: &RulesetView) -> &Ruleset {
    &view.document.as_ref().expect("a document").ruleset
}

/// The document's tier ids in order.
fn tier_ids(view: &RulesetView) -> Vec<&str> {
    ruleset(view)
        .tiers
        .iter()
        .map(|tier| tier.id.as_str())
        .collect()
}

/// The SpinButton node whose value is `value`.
fn spin<'a>(nodes: &'a [accesskit::Node], value: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some(value))
        .unwrap_or_else(|| panic!("a SpinButton with value {value:?}"))
}

/// The SpinButton to the right of the label `label` on its row (a
/// free-cards count sits beside its group's words).
fn spin_right_of<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    let bounds = node(nodes, label).bounds().expect("a drawn label");
    nodes
        .iter()
        .find(|node| {
            node.role() == Role::SpinButton
                && node
                    .bounds()
                    .is_some_and(|b| b.x0 >= bounds.x1 - 1.0 && (b.y0 - bounds.y0).abs() < 20.0)
        })
        .unwrap_or_else(|| panic!("a SpinButton right of {label:?}"))
}

/// `text` typed into the focused field, replacing what it held:
/// Ctrl+A selects the old text, then the new text takes its place.
fn type_replacing(window: &mut Headless, view: &mut RulesetView, ctx: &ToolContext, text: &str) {
    window.frame(vec![key(egui::Key::A, egui::Modifiers::COMMAND)], |ui| {
        show(ui, view, ctx)
    });
    window.frame(vec![egui::Event::Text(text.to_string())], |ui| {
        show(ui, view, ctx)
    });
}

/// How many nodes of `role` are labeled (or valued) `text`: every
/// label's text appears on its widget's node and again on a `TextRun`
/// child, so a role filter counts each text once.
fn count_role(nodes: &[accesskit::Node], role: Role, text: &str) -> usize {
    nodes_labeled(nodes, text)
        .into_iter()
        .filter(|node| node.role() == role)
        .count()
}

/// `frame` drew `expected` nodes of `role` labeled `text` (the
/// scenario's count assertions share this).
fn assert_count(frame: &Frame, role: Role, text: &str, expected: usize) {
    assert_eq!(
        count_role(&frame.nodes, role, text),
        expected,
        "{role:?} labeled {text:?}"
    );
}

/// The `role` node labeled (or valued) `text`.
fn role_labeled<'a>(nodes: &'a [accesskit::Node], role: Role, text: &str) -> &'a accesskit::Node {
    nodes_labeled(nodes, text)
        .into_iter()
        .find(|node| node.role() == role)
        .unwrap_or_else(|| panic!("a {role:?} labeled {text:?}"))
}

/// The first node `pred` accepts, `what` named in the panic.
fn find<'a>(
    nodes: &'a [accesskit::Node],
    what: &'static str,
    pred: impl Fn(&accesskit::Node) -> bool,
) -> &'a accesskit::Node {
    nodes.iter().find(|node| pred(node)).expect(what)
}

/// Each medal system's count of `tier` in the systems' order (`None`
/// when the system does not count it).
fn medal_counts(view: &RulesetView, tier: &str) -> Vec<Option<u8>> {
    ruleset(view)
        .medal_systems
        .iter()
        .map(|system| system.counts.get(tier).copied())
        .collect()
}

/// `frame`'s labels include every `fragment` (the refusal dialog's
/// listing).
fn assert_shown(frame: &Frame, fragments: &[&str]) {
    for fragment in fragments {
        assert!(
            contains_label(&frame.nodes, fragment),
            "missing {fragment:?}"
        );
    }
}

/// A click on the `role` node labeled `text`, on a settled frame.
fn click_role(
    window: &mut Headless,
    view: &mut RulesetView,
    ctx: &ToolContext,
    role: Role,
    text: &str,
) {
    let frame = settled(window, view, ctx);
    click(window, role_labeled(&frame.nodes, role, text), view, ctx);
}

/// Draws two frames and returns the second: a `Grid`'s first draw is
/// an invisible sizing pass with degenerate bounds, so node lookups
/// and clicks run on the frame after it.
fn settled(window: &mut Headless, view: &mut RulesetView, ctx: &ToolContext) -> Frame {
    window.draw(view, ctx);
    window.draw(view, ctx)
}

/// The id fields' values in drawn order, top to bottom (the table's
/// order).
fn drawn_tier_ids(nodes: &[accesskit::Node], ids: &[&str]) -> Vec<String> {
    let mut found: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|n| n.role() == Role::TextInput && n.value().is_some_and(|v| ids.contains(&v)))
        .collect();
    sort_top_down(&mut found);
    found
        .iter()
        .filter_map(|n| n.value().map(str::to_owned))
        .collect()
}

// RE-GUI-03
#[test]
fn a_rate_edit_shows_in_the_preview_and_marks_the_document_unsaved() {
    let (ctx, _events, _path, mut view) = official_opened("gui_03_rate");
    // The window is wide enough that the whole table sits beside the
    // preview pane (the default 800-wide one would clip the Rate
    // column out of the visible area).
    let mut window = Headless::sized(1800.0, 900.0);

    // Show preview on through the toolbar toggle, the Tiers section
    // open, and Silver's rate 92 changed to 90 in the table.
    let frame = window.draw(&mut view, &ctx);
    click_labeled(&mut window, &frame.nodes, "Show preview", &mut view, &ctx);
    view.section = Section::Tiers;
    let frame = settled(&mut window, &mut view, &ctx);
    assert_eq!(
        ruleset(&view).tiers[1].rate,
        92,
        "premise: Silver's rate is 92"
    );
    click(&mut window, spin(&frame.nodes, "92"), &mut view, &ctx);
    type_replacing(&mut window, &mut view, &ctx, "90");
    // Enter ends the DragValue's keyboard gesture, sealing the edit.
    window.frame(vec![key(egui::Key::Enter, egui::Modifiers::NONE)], |ui| {
        show(ui, &mut view, &ctx)
    });

    assert_eq!(ruleset(&view).tiers[1].rate, 90, "Silver's rate edited");
    assert!(
        view.document.as_ref().unwrap().is_dirty(),
        "the document is marked unsaved"
    );
    let line = ruleset(&view)
        .to_data_block()
        .lines()
        .find(|line| line.contains("id: \"silver\""))
        .expect("the silver line")
        .to_string();
    assert!(
        line.contains("rate: 90"),
        "the silver line reads rate 90: {line:?}"
    );
    assert_eq!(
        view.changed.iter().collect::<Vec<_>>(),
        vec![&line],
        "the silver line alone is highlighted"
    );

    // A view over the same settings starts with Show preview off.
    assert!(
        !RulesetView::new().show_preview,
        "Show preview off at start"
    );
}

// RE-GUI-04
#[test]
fn a_handle_drag_reorders_the_tiers() {
    let (ctx, _events, path, mut view) = official_opened("gui_04_drag");
    let mut window = Headless::new();
    view.section = Section::Tiers;

    let frame = settled(&mut window, &mut view, &ctx);
    assert_eq!(
        tier_ids(&view),
        ["gold", "silver", "bronze", "goalkeeper", "regular"]
    );
    let handles: Vec<&accesskit::Node> = nodes_labeled(&frame.nodes, "⠇")
        .into_iter()
        .filter(|node| node.role() == Role::Label)
        .collect();
    assert_eq!(handles.len(), 5, "one handle per tier");

    // Bronze's handle (the third row's) dragged onto Silver's row.
    let from = centre(handles[2]);
    let onto = centre(handles[1]);
    window.frame(pointer_at(from, true), |ui| show(ui, &mut view, &ctx));
    window.frame(vec![egui::Event::PointerMoved(onto)], |ui| {
        show(ui, &mut view, &ctx)
    });
    window.frame(pointer_at(onto, false), |ui| show(ui, &mut view, &ctx));

    assert_eq!(
        tier_ids(&view),
        ["gold", "bronze", "silver", "goalkeeper", "regular"],
        "bronze takes silver's place"
    );
    let frame = settled(&mut window, &mut view, &ctx);
    assert_eq!(
        drawn_tier_ids(
            &frame.nodes,
            &["gold", "silver", "bronze", "goalkeeper", "regular"]
        ),
        ["gold", "bronze", "silver", "goalkeeper", "regular"],
        "the table lists bronze before silver"
    );

    assert!(save_to(&mut view, &ctx, &path), "the save writes");
    let rules =
        aatf::Rules::load(&std::fs::read_to_string(&path).unwrap()).expect("the saved file loads");
    assert_eq!(
        rules
            .ruleset()
            .tiers
            .iter()
            .map(|tier| tier.id.as_str())
            .collect::<Vec<_>>(),
        ["gold", "bronze", "silver", "goalkeeper", "regular"],
        "the saved file's tiers"
    );
}

// RE-GUI-05
#[test]
fn a_shared_tier_id_marks_the_rail_and_blocks_save() {
    let (ctx, _events, path, mut view) = official_opened("gui_05_id");
    let mut window = Headless::new();
    view.section = Section::Tiers;

    // Silver's id typed to "gold": two tiers share the id.
    click_role(&mut window, &mut view, &ctx, Role::TextInput, "silver");
    type_replacing(&mut window, &mut view, &ctx, "gold");
    assert_eq!(ruleset(&view).tiers[1].id, "gold", "premise: the id took");

    let frame = window.draw(&mut view, &ctx);
    assert_eq!(
        rail_labels(&frame.nodes)[1],
        "Tiers ⚠",
        "the Tiers entry marked"
    );

    // Save writes no file; the dialog lists the shared id.
    let before = std::fs::read(&path).unwrap();
    let frame = window.draw(&mut view, &ctx);
    click_labeled(&mut window, &frame.nodes, "Save", &mut view, &ctx);
    assert_file_unchanged(&path, &before);
    let frame = window.draw(&mut view, &ctx);
    assert_shown(&frame, &["Not saved", "\"gold\""]);

    // The dialog's OK dismisses it, then Undo clears the mark (the
    // Edit menu's Undo — the field's own undo would need focus, which
    // the menu click takes first).
    click_labeled(&mut window, &frame.nodes, "OK", &mut view, &ctx);
    menu_click(&mut window, &mut view, &ctx, "Edit", "Undo");
    let frame = window.draw(&mut view, &ctx);
    assert_eq!(rail_labels(&frame.nodes)[1], "Tiers", "the mark is gone");
    assert_eq!(ruleset(&view).tiers[1].id, "silver");
}

// RE-GUI-06
#[test]
fn the_medal_checkbox_moves_a_tier_into_and_out_of_the_systems() {
    let (ctx, _events, mut window, mut view) = fresh();
    open_document(&mut view, template("DBC7"));
    view.section = Section::Tiers;
    let dbc7_systems = ruleset(&view).medal_systems.clone();

    let frame = settled(&mut window, &mut view, &ctx);
    // Gold and Silver counted, the regular alone the rest.
    assert_count(&frame, Role::Label, "by medal system", 2);
    assert_count(&frame, Role::Label, "the rest", 1);

    // The Gold tier's medal checkbox cleared.
    click_role(&mut window, &mut view, &ctx, Role::CheckBox, "Gold");
    assert_eq!(
        medal_counts(&view, "gold"),
        [None; 4],
        "no system counts gold"
    );
    let frame = settled(&mut window, &mut view, &ctx);
    // Gold's count shows "the rest" with its checkbox, the systems'
    // Gold cells "—" under the cleared checkbox, silver alone counted.
    assert_count(&frame, Role::Label, "the rest", 2);
    assert_count(&frame, Role::CheckBox, "rest", 2);
    assert_count(&frame, Role::Label, "by medal system", 1);
    assert_count(&frame, Role::Label, "—", 8);
    assert_eq!(
        rail_labels(&frame.nodes)[1],
        "Tiers ⚠",
        "two rest tiers mark Tiers"
    );

    // Undone instead: the counts DBC7 had are back.
    ctrl_key(&mut window, &mut view, &ctx, egui::Key::Z);
    assert_eq!(
        ruleset(&view).medal_systems,
        dbc7_systems,
        "the counts DBC7 had"
    );
    let frame = settled(&mut window, &mut view, &ctx);
    assert_eq!(rail_labels(&frame.nodes)[1], "Tiers", "the mark is gone");

    // Cleared, then ticked again: Gold is by medal system once more,
    // each system counting it 0, the mark gone.
    click_role(&mut window, &mut view, &ctx, Role::CheckBox, "Gold");
    click_role(&mut window, &mut view, &ctx, Role::CheckBox, "Gold");
    assert_eq!(
        medal_counts(&view, "gold"),
        [Some(0); 4],
        "each system counts gold 0"
    );
    let frame = settled(&mut window, &mut view, &ctx);
    // Gold is by medal system once more, the marks gone.
    assert_count(&frame, Role::Label, "by medal system", 2);
    assert_eq!(rail_labels(&frame.nodes)[1], "Tiers", "the marks gone");
}

// RE-FILE-04
#[test]
fn a_label_edit_saves_vtl11s_file_byte_for_byte() {
    let dir = crate::test_support::scratch("file_04_label");
    let aatf_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/aatf");
    let block = std::fs::read_to_string(aatf_dir.join("rulesets/vtl11.rhai")).unwrap();
    let checks = aatf::file_sections(&block)
        .custom_checks
        .expect("VTL11 has checks");
    let vtl11 = aatf::Ruleset::from_data_block(&block).expect("VTL11 reads");
    let path = write_file(
        &dir,
        "vtl11.rhai",
        &aatf::assemble_file(&vtl11, None, Some(&checks)),
    );

    let (ctx, _events, mut window, mut view) = fresh();
    open_path(&mut view, &ctx, &path);
    view.section = Section::Tiers;

    // Silver's label edited in the table, then saved.
    click_role(&mut window, &mut view, &ctx, Role::TextInput, "Silver");
    type_replacing(&mut window, &mut view, &ctx, "Argent");
    let frame = window.draw(&mut view, &ctx);
    click_labeled(&mut window, &frame.nodes, "Save", &mut view, &ctx);

    // The saved file is `assemble_file` of the edited ruleset: the
    // header, the block holding the new label, the current logic and
    // the custom-checks section, byte for byte.
    let expected = aatf::assemble_file(&ruleset(&view).clone(), None, Some(&checks));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), expected);
}

// RE-FILE-06
#[test]
fn deleting_a_tier_the_custom_checks_read_blocks_save() {
    // The fixture's base is Spring 26: its specials' conditions name
    // the gold and silver tiers alone, so a fifth tier appended to it
    // (a count, nothing naming it) deletes without breaking a load
    // rule — the refusal then comes from the custom check's
    // `global::RULESET.tiers[4]` read alone.
    let mut spring26 = template("Spring 26");
    let mut extra = crate::document::tier("extra", "Extra", Some(0), 77);
    extra.cards = 0;
    spring26.tiers.push(extra);
    let checks = concat!(
        "// ==== CUSTOM CHECKS: optional, hand-written; the Ruleset editor keeps this section as it is ====\n",
        "fn custom_checks(team) {\n    let extra = global::RULESET.tiers[4];\n    []\n}"
    );
    let dir = crate::test_support::scratch("file_06_tier5");
    let path = write_file(
        &dir,
        "rules.rhai",
        &aatf::assemble_file(&spring26, None, Some(checks)),
    );

    let (ctx, _events, _window, mut view) = fresh();
    // The window is wide enough that the last column's ✕ is inside the
    // visible area.
    let mut window = Headless::sized(1600.0, 900.0);
    open_path(&mut view, &ctx, &path);
    assert_eq!(ruleset(&view).tiers.len(), 5, "premise: five tiers");
    view.section = Section::Tiers;

    // The fifth tier's ✕ clicked, then Save: nothing is written, the
    // refusal lists the sample team's script error.
    let frame = settled(&mut window, &mut view, &ctx);
    click_nth(&mut window, &frame.nodes, "✕", 4, &mut view, &ctx);
    assert_eq!(ruleset(&view).tiers.len(), 4, "the fifth tier is gone");

    let before = std::fs::read(&path).unwrap();
    let frame = window.draw(&mut view, &ctx);
    click_labeled(&mut window, &frame.nodes, "Save", &mut view, &ctx);
    assert_file_unchanged(&path, &before);
    let frame = window.draw(&mut view, &ctx);
    assert_shown(
        &frame,
        &["Not saved", "check_team fails on the sample team"],
    );
}

/// An id edit carries the tier's key in every medal system's counts
/// (the scenarios never rename a counted tier's id).
#[test]
fn the_id_edit_carries_the_tiers_medal_keys() {
    let (ctx, _events, mut window, mut view) = fresh();
    open_document(&mut view, template("DBC7"));
    view.section = Section::Tiers;

    click_role(&mut window, &mut view, &ctx, Role::TextInput, "gold");
    type_replacing(&mut window, &mut view, &ctx, "royal");
    assert_eq!(ruleset(&view).tiers[0].id, "royal", "the id took");
    let counts: Vec<BTreeMap<String, u8>> = ruleset(&view)
        .medal_systems
        .iter()
        .map(|system| system.counts.clone())
        .collect();
    assert_eq!(counts[0], BTreeMap::from([("royal".to_string(), 3)]));
    assert_eq!(
        counts[1],
        BTreeMap::from([("royal".to_string(), 2), ("silver".to_string(), 2)])
    );
    assert!(
        counts.iter().all(|map| !map.contains_key("gold")),
        "gold is gone from every system"
    );
}

/// A tier's ✕ drops its key from every medal system's counts.
#[test]
fn removing_a_tier_drops_its_medal_keys() {
    let (ctx, _events, _window, mut view) = fresh();
    // The window is wide enough that the last column's ✕ is inside the
    // visible area.
    let mut window = Headless::sized(1600.0, 900.0);
    open_document(&mut view, template("DBC7"));
    view.section = Section::Tiers;

    // The first row's ✕ (the tiers' ✕s sit above the systems' own).
    let frame = settled(&mut window, &mut view, &ctx);
    click_nth(&mut window, &frame.nodes, "✕", 0, &mut view, &ctx);
    assert_eq!(tier_ids(&view), ["silver", "regular"]);
    assert_eq!(
        medal_counts(&view, "gold"),
        [None; 4],
        "gold is gone from every system"
    );
}

/// An "own target" checkbox adds and removes the stat's `stats` entry
/// (`DECISIONS.md` 2026-10-24).
#[test]
fn a_per_stat_checkbox_adds_and_removes_the_stats_entry() {
    let (ctx, _events, mut window, mut view) = fresh_blank();
    view.section = Section::Tiers;

    // Blank's one tier's expander, opened; "Dribbling" ticked, then
    // unticked.
    let frame = settled(&mut window, &mut view, &ctx);
    click_labeled(
        &mut window,
        &frame.nodes,
        "Stat targets, free cards and conditions",
        &mut view,
        &ctx,
    );
    click_role(&mut window, &mut view, &ctx, Role::CheckBox, "Dribbling");
    let stats = &ruleset(&view).tiers[0].stats;
    assert_eq!(
        stats.keys().map(|stat| stat.label()).collect::<Vec<_>>(),
        ["dribbling"],
        "the entry added"
    );
    assert_eq!(
        stats.values().copied().collect::<Vec<_>>(),
        [75],
        "at the rate it followed"
    );
    click_role(&mut window, &mut view, &ctx, Role::CheckBox, "Dribbling");
    assert!(
        ruleset(&view).tiers[0].stats.is_empty(),
        "unticked, the entry is gone"
    );
}

/// A free-cards count set to 0 removes the group's `free` entry
/// (`DECISIONS.md` 2026-10-24).
#[test]
fn a_free_count_of_zero_removes_the_groups_entry() {
    let (ctx, _events, mut window, mut view) = fresh();
    open_document(&mut view, aatf::Rules::official().ruleset().clone());
    view.section = Section::Tiers;

    // The regular tier's expander (the last "Stat targets…" header):
    // its free Tricks count set to 0 removes the entry.
    let frame = settled(&mut window, &mut view, &ctx);
    let headers = nodes_labeled(&frame.nodes, "Stat targets, free cards and conditions");
    assert_eq!(headers.len(), 5, "one expander per tier");
    click(&mut window, headers[4], &mut view, &ctx);
    assert_eq!(
        ruleset(&view).tiers[4].free["tricks"],
        2,
        "premise: two free tricks"
    );
    let frame = settled(&mut window, &mut view, &ctx);
    click(
        &mut window,
        spin_right_of(&frame.nodes, "Tricks"),
        &mut view,
        &ctx,
    );
    type_replacing(&mut window, &mut view, &ctx, "0");
    assert!(
        !ruleset(&view).tiers[4].free.contains_key("tricks"),
        "0 removed the entry"
    );
}

/// A colour picked through the picker keeps the code's alpha (the
/// last two digits; the draft's `colour`).
#[test]
fn a_name_colour_pick_keeps_the_alpha() {
    let (ctx, _events, _window, mut view) = fresh();
    // The window is wide enough that the name-colour field is inside
    // the visible area.
    let mut window = Headless::sized(1600.0, 900.0);
    open_document(&mut view, aatf::Rules::official().ruleset().clone());
    edit_ruleset(&mut view, |ruleset| {
        ruleset.tiers[0].name_color = Some("cc990080".to_string())
    });
    view.section = Section::Tiers;

    // The picker's hex field takes "#112233": the written code keeps
    // the old alpha.
    let frame = settled(&mut window, &mut view, &ctx);
    let well = find(&frame.nodes, "the colour field", |node| {
        node.label()
            .is_some_and(|label| label.starts_with("color field"))
    });
    click(&mut window, well, &mut view, &ctx);
    let frame = settled(&mut window, &mut view, &ctx);
    let hex = find(&frame.nodes, "the picker's hex field", |node| {
        node.role() == Role::TextInput && node.value().is_some_and(|value| value.starts_with('#'))
    });
    click(&mut window, hex, &mut view, &ctx);
    type_replacing(&mut window, &mut view, &ctx, "#112233");
    window.frame(vec![key(egui::Key::Enter, egui::Modifiers::NONE)], |ui| {
        show(ui, &mut view, &ctx)
    });
    assert_eq!(
        ruleset(&view).tiers[0].name_color.as_deref(),
        Some("11223380"),
        "the picked colour keeps the code's alpha"
    );
}
