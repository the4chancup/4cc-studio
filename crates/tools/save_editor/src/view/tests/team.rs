use pes_savefile::model::TeamEntry;
use pes_savefile::schema::fields::TeamText;
use pes_savefile::schema::schema_for;

use super::*;

/// The registered position's full name, GK 0 to CF 12 (`session::team_ops`'
/// table, pinned here so a wrong spelling fails against a literal).
const POSITION_NAMES: [&str; 13] = [
    "Goalkeeper",
    "Center Back",
    "Left Back",
    "Right Back",
    "Defensive Midfielder",
    "Center Midfielder",
    "Left Midfielder",
    "Right Midfielder",
    "Attacking Midfielder",
    "Left Wing Forward",
    "Right Wing Forward",
    "Second Striker",
    "Centre Forward",
];

/// The node of `role` with `label`, or panic naming both.
fn labeled<'a>(nodes: &'a [accesskit::Node], role: Role, label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("no {role:?} {label:?}"))
}

/// Replaces the focused text control's contents with `text`: End moves the
/// caret past the text, the backspaces clear it, then the characters land
/// (`color_tools`' `type_text`).
fn type_text(window: &mut Headless, view: &mut SaveView, text: &str) {
    let mut events = vec![key(egui::Key::End, egui::Modifiers::NONE, true)];
    for _ in 0..16 {
        events.push(key(egui::Key::Backspace, egui::Modifiers::NONE, true));
    }
    events.push(egui::Event::Text(text.to_owned()));
    window.frame(events, |ui| show(ui, view, &|| Vec::new(), None));
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, view, &|| Vec::new(), None),
    );
}

/// The team's entry as the session holds it.
fn team_entry(view: &SaveView, team_id: u32) -> TeamEntry {
    view.session
        .as_ref()
        .unwrap()
        .file()
        .team(team_id)
        .unwrap()
        .clone()
}

/// The rostered `(slot, player_id)` pairs of `team_id`, in slot order.
fn rostered(view: &SaveView, team_id: u32) -> Vec<(usize, u32)> {
    team_entry(view, team_id)
        .roster
        .iter()
        .enumerate()
        .filter(|(_, slot)| slot.player_id != 0)
        .map(|(slot, roster_slot)| (slot, roster_slot.player_id))
        .collect()
}

/// The team bar's name field: the TextInput right of the topmost "Team"
/// label (the tab strip's own weak "Team" scope label sits lower).
fn bar_name_field(nodes: &[accesskit::Node]) -> &accesskit::Node {
    let label = nodes
        .iter()
        .filter(|node| node.role() == Role::Label && node.value() == Some("Team"))
        .min_by(|a, b| {
            let y = |node: &&accesskit::Node| node.bounds().map(|b| b.y0).unwrap_or(f64::MAX);
            y(a).partial_cmp(&y(b)).expect("ordered bounds")
        })
        .expect("the bar's Team label");
    right_of(nodes, label, Role::TextInput)
}

/// The Team menu button (the toolbar's, not the tab strip's "Team" tab):
/// the topmost button with the label.
fn team_menu_button(nodes: &[accesskit::Node]) -> &accesskit::Node {
    nodes
        .iter()
        .filter(|node| node.role() == Role::Button && node.label() == Some("Team"))
        .min_by(|a, b| {
            let y = |node: &&accesskit::Node| node.bounds().map(|r| r.y0).unwrap_or(f64::MAX);
            y(a).partial_cmp(&y(b)).expect("ordered bounds")
        })
        .expect("the Team menu button")
}

/// The Team menu's popup open for one frame: a click on the Team button
/// (skipped when a previous call left it open), then the frame's nodes.
fn open_team_menu(window: &mut Headless, view: &mut SaveView) -> Vec<accesskit::Node> {
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    if nodes
        .iter()
        .any(|node| node.role() == Role::Button && node.label() == Some("Set edited flag"))
    {
        return nodes;
    }
    click(window, team_menu_button(&nodes), view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    nodes
}

// SE-GUI-10
#[test]
fn the_team_fields_edit_and_save_and_the_colour_is_the_nearest_the_save_holds() {
    let path = fixture(PesVersion::Pes21, "team_tab_edit");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::TeamSettings;
    let mut window = Headless::sized(1100.0, 700.0);
    assert_eq!(
        team_entry(&view, 701).name,
        "/3/",
        "premise: team 701's name"
    );
    assert_eq!(
        team_entry(&view, 701).manager_id,
        Some(701),
        "premise: the manager ID"
    );

    // The Name field: the team bar's text input holding the team name
    // (the header's own Name field holds the player's).
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let name_field = nodes
        .iter()
        .find(|node| node.role() == Role::TextInput && node.value() == Some("/3/"))
        .expect("the team's Name field");
    click(&mut window, name_field, &mut view);
    type_text(&mut window, &mut view, "MY TEAM");
    assert_eq!(team_entry(&view, 701).name, "MY TEAM");
    assert!(
        team_entry(&view, 701).edit_flags.name,
        "a name edit sets the name flag"
    );

    // The Manager ID control: a typed value lands.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let manager = control_right_of(&nodes, "Manager ID");
    click(&mut window, manager, &mut view);
    window.frame(vec![egui::Event::Text("777".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(team_entry(&view, 701).manager_id, Some(777));

    // Colour 1: the picker's hex field takes #FF8000; the save keeps 6
    // bits a channel, so the stored colour is the nearest the save holds.
    // Both colour fields share the grid row, so Colour 1 is the leftmost.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let field = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::ColorWell
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("color field"))
        })
        .min_by(|a, b| {
            let x = |node: &&accesskit::Node| node.bounds().expect("bounds").x0;
            x(a).partial_cmp(&x(b)).expect("ordered bounds")
        })
        .expect("the Colour 1 field");
    click(&mut window, field, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let hex = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && node.value().is_some_and(|value| value.starts_with('#'))
        })
        .expect("the picker's hex field");
    click(&mut window, hex, &mut view);
    type_text(&mut window, &mut view, "#FF8000");
    let color = team_entry(&view, 701)
        .colors
        .expect("PES 21 stores colours")[0];
    assert_eq!(
        (color.red, color.green, color.blue),
        (63, 32, 0),
        "#FF8000 stores as the nearest 6-bit colour"
    );
    // The hex beside the field reads the shown colour back: #FF8200.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        nodes
            .iter()
            .any(|node| node.role() == Role::Label && node.value() == Some("#FF8200")),
        "the team bar shows #FF8200"
    );

    // Save; the reopened file holds the three values.
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let reopened = EditFile::load(&path).unwrap();
    let team = reopened.team(701).expect("team 701");
    assert_eq!(team.name, "MY TEAM");
    assert_eq!(team.manager_id, Some(777));
    let color = team.colors.expect("colours")[0];
    assert_eq!((color.red, color.green, color.blue), (63, 32, 0));
}

#[test]
fn the_bound_team_id_shows_binding_over_64_and_an_edit_keeps_the_low_bits() {
    let path = fixture(PesVersion::Pes17, "team_kit_slots");
    // Give slot 1 a binding with low bits set: the edit must keep them.
    // The strip flag clear, so a kit-slot edit's setting it shows.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes17)).unwrap();
    let slots = file
        .team_mut(701)
        .unwrap()
        .kit_slots
        .as_mut()
        .expect("PES 17 kit slots");
    slots[0].binding = 701 * 64 + 5;
    let binding = slots[0].binding;
    file.team_mut(701).unwrap().edit_flags.strip = Some(false);
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::TeamSettings;
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // Slot 1's row: the weak "Slot 1" label's row holds Number and Bound
    // team ID, in x order.
    let slot_label = nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some("Slot 1"))
        .expect("the Slot 1 row");
    let row_y = slot_label.bounds().expect("bounds").y0;
    let mut controls: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::SpinButton
                && node.bounds().is_some_and(|b| (b.y0 - row_y).abs() < 4.0)
        })
        .collect();
    controls.sort_by(|a, b| {
        a.bounds()
            .expect("bounds")
            .x0
            .partial_cmp(&b.bounds().expect("bounds").x0)
            .expect("ordered bounds")
    });
    assert_eq!(controls.len(), 2, "Number and Bound team ID on the row");
    let bound = controls[1];
    assert_eq!(
        bound.value(),
        Some((binding / 64).to_string()).as_deref(),
        "Bound team ID shows binding / 64"
    );

    // A Number edit sets the team's strip flag, as the stadium edit sets
    // its own (PES 17 is the version with kit slots, so the flag is Some).
    click(&mut window, controls[0], &mut view);
    window.frame(vec![egui::Event::Text("9".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(
        team_entry(&view, 701).kit_slots.expect("kit slots")[0].number,
        9
    );
    assert_eq!(
        team_entry(&view, 701).edit_flags.strip,
        Some(true),
        "a kit-slot edit sets the strip flag"
    );
    // The one undo restores the edit whole, the flag back to clear (a
    // frame first, so the field has given up the keyboard and Ctrl+Z is
    // the session's undo, not the text's own).
    window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        team_entry(&view, 701).edit_flags.strip,
        Some(false),
        "the undo restores the flag too"
    );

    // An edit writes `id * 64 + binding % 64`: the low 5 stays.
    click(&mut window, bound, &mut view);
    window.frame(vec![egui::Event::Text("702".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    let slots = team_entry(&view, 701).kit_slots.expect("kit slots");
    assert_eq!(
        slots[0].binding,
        702 * 64 + binding % 64,
        "the low bits are kept as stored"
    );
    assert_eq!(
        team_entry(&view, 701).edit_flags.strip,
        Some(true),
        "the bound edit sets the strip flag too"
    );

    // The shown id's top is the binding field's own top / 64 (PES 17's
    // binding is 24 bits, so 262143): typing 262144 stops there.
    click(&mut window, bound, &mut view);
    window.frame(vec![egui::Event::Text("262144".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    let slots = team_entry(&view, 701).kit_slots.expect("kit slots");
    assert_eq!(
        slots[0].binding,
        262143 * 64 + binding % 64,
        "past the top the id is 262143, the low bits still kept"
    );
}

// The bar's name field gets its 240 points from `add_sized`: the field
// keeps its own width rather than the wrapped row's offer.
#[test]
fn the_team_name_field_gets_its_full_width_on_the_first_frame() {
    let path = fixture(PesVersion::Pes21, "team_name_width");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::TeamSettings;
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let name = nodes
        .iter()
        .find(|node| node.role() == Role::TextInput && node.value() == Some("/3/"))
        .expect("the team's Name field");
    let bounds = name.bounds().expect("bounds");
    assert!(
        bounds.x1 - bounds.x0 >= 230.0,
        "the Name field draws near 240 wide, got {bounds:?}"
    );
}

// `names_row`'s `changed() && team_text_fits` guard is what keeps drawing
// from being an edit: frames of the tab with no input leave the flag and
// the session as they were.
#[test]
fn drawing_the_team_tab_edits_nothing() {
    let path = fixture(PesVersion::Pes21, "team_tab_no_input");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::TeamSettings;
    assert!(
        !team_entry(&view, 701).edit_flags.name,
        "premise: the name flag is clear"
    );
    let mut window = Headless::sized(1100.0, 700.0);
    for _ in 0..3 {
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    }
    let session = view.session.as_ref().unwrap();
    assert!(!session.is_dirty(), "three frames left the session clean");
    assert!(!session.can_undo(), "and pushed no undo entry");
    assert!(
        !team_entry(&view, 701).edit_flags.name,
        "the name flag stayed clear"
    );
}

// The same guard's other arm: a keystroke `team_text_fits` refuses is not
// taken — the field shows the stored text again next frame (the info
// strip's Name field's rule).
#[test]
fn a_team_name_keystroke_past_the_fields_limit_is_not_taken() {
    let path = fixture(PesVersion::Pes21, "team_name_overlong");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::TeamSettings;
    let mut window = Headless::sized(1100.0, 700.0);
    // The version's own field length in bytes, read from the schema —
    // never a typed constant.
    let limit = usize::try_from(
        schema_for(PesVersion::Pes21)
            .team
            .texts
            .iter()
            .find(|spec| spec.text == TeamText::Name)
            .expect("every version stores the team name")
            .len,
    )
    .expect("a text field's length fits usize");
    let full = "A".repeat(limit);

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let name_field = nodes
        .iter()
        .find(|node| node.role() == Role::TextInput && node.value() == Some("/3/"))
        .expect("the team's Name field");
    click(&mut window, name_field, &mut view);
    type_text(&mut window, &mut view, &full);
    assert_eq!(
        team_entry(&view, 701).name,
        full,
        "a name at the limit lands"
    );

    // One more letter past it: refused, the stored name stands.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let name_field = nodes
        .iter()
        .find(|node| node.role() == Role::TextInput && node.value() == Some(full.as_str()))
        .expect("the team's Name field");
    click(&mut window, name_field, &mut view);
    window.frame(
        vec![
            key(egui::Key::End, egui::Modifiers::NONE, true),
            egui::Event::Text("B".to_owned()),
        ],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(
        team_entry(&view, 701).name,
        full,
        "the past-limit keystroke is not taken"
    );
}

// The dialog's "(→ N)" preview reads the last *filled* slot's resulting
// ID: slot k (1-based) gets start + k - 1, so the preview is the start
// plus the filled slot's 0-based index (an empty last slot is not it).
#[test]
fn the_boglo_dialogs_preview_names_the_last_filled_slots_id() {
    let path = fixture(PesVersion::Pes21, "boglo_preview");
    let mut view = SaveView::new();
    view.open(&path);
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    // The last filled slot's index, read from the roster, never typed in.
    let last = rostered(&view, 701)
        .last()
        .expect("team 701 has filled slots")
        .0;

    let nodes = open_team_menu(&mut window, &mut view);
    click(
        &mut window,
        labeled(
            &nodes,
            Role::Button,
            "Set boots and gloves IDs for everyone\u{2026}",
        ),
        &mut view,
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        labeled(&nodes, Role::RadioButton, "Incremental from"),
        &mut view,
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let incremental = labeled(&nodes, Role::RadioButton, "Incremental from");
    let start = right_of(&nodes, incremental, Role::SpinButton);
    click(&mut window, start, &mut view);
    window.frame(vec![egui::Event::Text("100".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let preview = format!("(\u{2192} {})", 100 + last);
    assert!(
        nodes
            .iter()
            .any(|node| node.role() == Role::Label && node.value() == Some(preview.as_str())),
        "the preview names {preview}"
    );
}

// plans/save_editor.md "Editing UI" (the version-gated bullet, on the Team
// tab): fields the version does not store draw disabled in the same place
// with the gate tooltip naming the first version that has them.
#[test]
fn on_pes16_the_colours_ids_and_kit_slots_draw_disabled_with_their_versions() {
    let path = fixture(PesVersion::Pes16, "team_tab_gated");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::TeamSettings;
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // The two colour fields, disabled with "PES 17+".
    let mut colours: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::ColorWell
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("color field"))
        })
        .collect();
    colours.sort_by(|a, b| {
        a.bounds()
            .expect("bounds")
            .x0
            .partial_cmp(&b.bounds().expect("bounds").x0)
            .expect("ordered bounds")
    });
    assert_eq!(colours.len(), 2, "two colour fields");
    for field in &colours {
        assert!(field.is_disabled(), "the colour field is disabled");
        assert_eq!(
            tooltip_after_hover(&mut window, &mut view, centre(field)),
            Some("PES 17+".to_string()),
            "the colour field's tooltip"
        );
    }
    // Manager ID and Stadium ID, each its disabled stand-in with "PES 19+".
    for label in ["Manager ID", "Stadium ID"] {
        let dash = disabled_dash(&nodes, label);
        assert_eq!(
            tooltip_after_hover(&mut window, &mut view, centre(dash)),
            Some("PES 19+".to_string()),
            "{label}'s tooltip"
        );
    }
    // The kit slots' no-records line names PES 17.
    assert!(
        label_value(&nodes, "(no kit slot records on this version)"),
        "the no-records line"
    );
}

/// Whether the nodes draw a Label with `value` as its text.
fn label_value(nodes: &[accesskit::Node], value: &str) -> bool {
    nodes
        .iter()
        .any(|node| node.role() == Role::Label && node.value() == Some(value))
}

#[test]
fn the_short_name_upper_cases_caps_at_three_and_sets_the_flags() {
    for (version, test) in [
        (PesVersion::Pes15, "short_name_15"),
        (PesVersion::Pes21, "short_name_21"),
    ] {
        let path = fixture(version, test);
        let mut view = SaveView::new();
        view.open(&path);
        view.tab = Tab::TeamSettings;
        assert_eq!(
            team_entry(&view, 701).short_name,
            "3",
            "premise: team 701's short name"
        );
        let mut window = Headless::sized(1100.0, 700.0);
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        let field = nodes
            .iter()
            .find(|node| node.role() == Role::TextInput && node.value() == Some("3"))
            .expect("the Short name field");
        click(&mut window, field, &mut view);
        type_text(&mut window, &mut view, "abc");
        let team = team_entry(&view, 701);
        assert_eq!(team.short_name, "ABC", "upper-cased as typed");
        assert!(team.edit_flags.name, "the name flag is set");

        // A fourth letter is not taken: the field shows the stored text again.
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        let field = nodes
            .iter()
            .find(|node| node.role() == Role::TextInput && node.value() == Some("ABC"))
            .expect("the Short name field");
        click(&mut window, field, &mut view);
        type_text(&mut window, &mut view, "abcd");
        assert_eq!(team_entry(&view, 701).short_name, "ABC");
        assert_eq!(
            team_entry(&view, 701).edit_flags.short_name,
            (version == PesVersion::Pes15).then_some(true),
            "the short-name flag on {version}"
        );
    }
}

// The selected player's team's name, short name and colours in
// a bar above the tab strip, drawn whichever tab is open.
// SE-GUI-10
#[test]
fn the_team_bar_draws_on_every_tab() {
    let (mut view, mut window) = open_734_with("team_bar_tabs", &[], 0);
    for tab in [
        Tab::Abilities,
        Tab::Appearance,
        Tab::Tactics,
        Tab::TeamSettings,
    ] {
        view.tab = tab;
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        assert_eq!(
            bar_name_field(&nodes).value(),
            Some("/lit/"),
            "the bar's name field on {tab:?}"
        );
    }

    // A short-name edit from another tab lands in the team's entry as one
    // undoable edit.
    let before = team_entry(&view, 734).short_name;
    assert_ne!(before, "LTZ", "premise: a real change");
    view.tab = Tab::Tactics;
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let short = right_of_label(&nodes, "Short", Role::TextInput)
        .first()
        .copied()
        .expect("the bar's Short field");
    click(&mut window, short, &mut view);
    type_text(&mut window, &mut view, "ltz");
    assert_eq!(
        team_entry(&view, 734).short_name,
        "LTZ",
        "upper-cased as typed"
    );
    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undoable edit");
}

// The Team settings tab's contents (plans "Editing UI", the Team settings
// line): the manager and stadium IDs and the kit slot assignments; the
// name and colour fields are the bar's, which is not the tab's.
#[test]
fn the_team_settings_tab_holds_the_ids_and_kit_slots_alone() {
    let (mut view, mut window) = open_734_with("team_settings_contents", &[], 0);
    view.tab = Tab::TeamSettings;
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    control_right_of(&nodes, "Manager ID");
    control_right_of(&nodes, "Stadium ID");
    assert!(label_value(&nodes, "Slot 1"), "the kit slot rows (PES 17)");

    // Below the tab strip is the tab's content: no name field, no colour
    // field — the only ones drawn are the bar's, above it.
    let strip_y = nodes
        .iter()
        .find(|node| {
            node.role() == Role::Button
                && node.label() == Some("Abilities & Skills")
                && node.toggled().is_some()
        })
        .and_then(accesskit::Node::bounds)
        .map(|bounds| bounds.y0)
        .expect("the tab strip");
    let below = |node: &&accesskit::Node| {
        node.bounds()
            .is_some_and(|bounds| bounds.y0 > strip_y + 1.0)
    };
    // Premises: the bar's fields draw — the name field holding /lit/ and
    // the two colour wells, all above the strip.
    let names: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| node.role() == Role::TextInput && node.value() == Some("/lit/"))
        .collect();
    let bar_name = names
        .first()
        .copied()
        .expect("premise: the bar's name field is drawn");
    assert!(!below(&bar_name), "premise: the name field is the bar's");
    for node in names {
        assert!(!below(&node), "the team name field stays in the bar");
    }
    let colours: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| node.role() == Role::ColorWell)
        .collect();
    assert!(colours.len() >= 2, "premise: the bar's two colours draw");
    assert!(
        colours.iter().all(|node| !below(node)),
        "the colour fields stay in the bar"
    );
}

// SE-GUI-23
#[test]
fn the_team_menu_operations_edit_every_filled_slot_as_one_undoable_edit() {
    let path = fixture(PesVersion::Pes21, "team_menu_ops");
    // A goalkeeper among the CB placeholders, so the position names cover
    // two table entries; the visual flags set, so Clear visual flags is a
    // real change rather than a no-op batch.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().positions.registered = 0;
    let ids: Vec<u32> = file
        .team(701)
        .unwrap()
        .roster
        .iter()
        .map(|slot| slot.player_id)
        .filter(|&id| id != 0)
        .collect();
    for id in ids {
        let flags = &mut file.player_mut(id).unwrap().edit_flags;
        flags.face = true;
        flags.hair = true;
        flags.physique = true;
        flags.strip = true;
    }
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    // A rostered player of another team, to prove the operations stay on
    // the selected player's team.
    let other_id = {
        let file = view.session.as_ref().unwrap().file();
        file.teams()
            .iter()
            .filter(|team| team.id != 701)
            .flat_map(|team| team.roster.iter())
            .map(|slot| slot.player_id)
            .find(|&id| id != 0)
            .expect("a player of another team")
    };
    let other_before = player_entry(&view, other_id);
    assert!(!player_entry(&view, 70101).edit_flags.player, "premise");

    // Set edited flag: every filled slot's player flag is set.
    let nodes = open_team_menu(&mut window, &mut view);
    click(
        &mut window,
        labeled(&nodes, Role::Button, "Set edited flag"),
        &mut view,
    );
    let slots = rostered(&view, 701);
    assert!(slots.len() > 2, "premise: several filled slots");
    for &(_, id) in &slots {
        assert!(player_entry(&view, id).edit_flags.player, "{id} flagged");
    }
    assert_eq!(player_entry(&view, other_id), other_before, "untouched");

    // Set boots and gloves IDs for everyone: Incremental from 100, gloves
    // same as the boot IDs — slot k gets 100 + k - 1 on both (the check's
    // bound is the record's 14-bit field, not settings.toml's 0 to 100).
    let start_value = 100u32;
    let nodes = open_team_menu(&mut window, &mut view);
    click(
        &mut window,
        labeled(
            &nodes,
            Role::Button,
            "Set boots and gloves IDs for everyone\u{2026}",
        ),
        &mut view,
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let incremental = labeled(&nodes, Role::RadioButton, "Incremental from");
    click(&mut window, incremental, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let incremental = labeled(&nodes, Role::RadioButton, "Incremental from");
    let start = right_of(&nodes, incremental, Role::SpinButton);
    click(&mut window, start, &mut view);
    window.frame(vec![egui::Event::Text(start_value.to_string())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        labeled(&nodes, Role::RadioButton, "Same as the boot IDs"),
        &mut view,
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(&mut window, labeled(&nodes, Role::Button, "OK"), &mut view);
    for &(slot, id) in &slots {
        let player = player_entry(&view, id);
        let expected = start_value + u32::try_from(slot).expect("a roster has at most 40 slots");
        assert_eq!(
            player.appearance.boots_id,
            expected,
            "slot {}'s boots",
            slot + 1
        );
        assert_eq!(
            player.appearance.gloves_id,
            expected,
            "slot {}'s gloves",
            slot + 1
        );
    }

    // Set player names to positions: the registered position's full name.
    let nodes = open_team_menu(&mut window, &mut view);
    click(
        &mut window,
        labeled(&nodes, Role::Button, "Set player names to positions"),
        &mut view,
    );
    for &(slot, id) in &slots {
        let player = player_entry(&view, id);
        assert_eq!(
            player.name,
            POSITION_NAMES[usize::from(player.positions.registered)],
            "slot {}'s name",
            slot + 1
        );
    }
    assert_eq!(player_entry(&view, 70101).name, "Goalkeeper");
    assert_eq!(player_entry(&view, 70102).name, "Center Back");

    // Clear visual flags by Ctrl+F: the four flags off on every slot.
    window.frame(vec![key(egui::Key::F, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    for &(_, id) in &slots {
        let flags = &player_entry(&view, id).edit_flags;
        assert!(
            !(flags.face || flags.hair || flags.physique || flags.strip),
            "{id}'s visual flags cleared"
        );
    }

    // Four operations, four undo entries; the undos put every player back.
    assert_eq!(undo_entries(&mut window, &mut view), 4, "one entry each");
    assert_eq!(player_entry(&view, 70101).name, "PLACEHOLDER");
    assert!(!player_entry(&view, 70101).edit_flags.player);
    // Redo them all, save, reopen: the file holds the operation's values.
    for _ in 0..4 {
        window.frame(vec![key(egui::Key::Y, egui::Modifiers::CTRL, true)], |ui| {
            show(ui, &mut view, &|| Vec::new(), None)
        });
    }
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let reopened = EditFile::load(&path).unwrap();
    for &(slot, id) in &slots {
        let player = reopened.player(id).expect("the player");
        let expected = start_value + u32::try_from(slot).expect("a roster has at most 40 slots");
        assert_eq!(player.appearance.boots_id, expected);
        assert_eq!(player.appearance.gloves_id, expected);
        assert_eq!(
            player.name,
            POSITION_NAMES[usize::from(player.positions.registered)]
        );
        assert!(player.edit_flags.player);
        let flags = &player.edit_flags;
        assert!(!(flags.face || flags.hair || flags.physique || flags.strip));
    }
    assert_eq!(
        reopened.player(other_id).unwrap(),
        &other_before,
        "a player of another team is unchanged"
    );
}

// SE-GUI-22 (the Team menu's part): under the lock the aesthetics items
// are disabled with the ownership line as hover, Ctrl+F is a no-op, and
// the gameplay items still run.
#[test]
fn the_locked_team_menu_disables_the_aesthetics_items_with_the_ownership_line() {
    let path = fixture(PesVersion::Pes21, "team_menu_locked");
    // Two players' visual flags set, so a guardless Ctrl+F would make a
    // visible change (the fixture's are clear, which let the guard go).
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    for id in [70101u32, 70102] {
        let flags = &mut file.player_mut(id).unwrap().edit_flags;
        flags.face = true;
        flags.hair = true;
        flags.physique = true;
        flags.strip = true;
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let nodes = open_team_menu(&mut window, &mut view);

    for label in [
        "Set boots and gloves IDs for everyone\u{2026}",
        "Clear visual flags",
    ] {
        let item = nodes
            .iter()
            .find(|node| {
                node.role() == Role::Button
                    && node
                        .label()
                        .is_some_and(|node_label| node_label.starts_with(label))
            })
            .unwrap_or_else(|| panic!("no menu item {label:?}"));
        assert!(item.is_disabled(), "{label} is disabled under the lock");
        assert_eq!(
            tooltip_containing(&mut window, &mut view, centre(item), "Owned by"),
            Some("Owned by the team's export: edit its settings.toml and recompile.".to_string()),
            "{label}'s hover is the ownership line"
        );
    }
    for label in ["Set edited flag", "Set player names to positions"] {
        assert!(
            !labeled(&nodes, Role::Button, label).is_disabled(),
            "{label} stays enabled"
        );
    }

    // Ctrl+F under the lock: nothing changes, the flags still set.
    window.frame(vec![key(egui::Key::F, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(
        !view.session.as_ref().unwrap().is_dirty(),
        "Ctrl+F changed nothing"
    );
    for id in [70101u32, 70102] {
        let flags = &player_entry(&view, id).edit_flags;
        assert!(
            flags.face && flags.hair && flags.physique && flags.strip,
            "{id}'s visual flags survived the locked Ctrl+F"
        );
    }

    // Set edited flag still runs.
    let nodes = open_team_menu(&mut window, &mut view);
    click(
        &mut window,
        labeled(&nodes, Role::Button, "Set edited flag"),
        &mut view,
    );
    for &(_, id) in &rostered(&view, 701) {
        assert!(player_entry(&view, id).edit_flags.player, "{id} flagged");
    }
}

// The menu item itself (the other SE-GUI-23 test runs the clear through
// Ctrl+F). Every filled slot's four visual flags set first, then cleared,
// one undo restoring every one of them, redo clearing them again, and the
// saved file holding them clear.
// SE-GUI-23
#[test]
fn the_clear_visual_flags_item_clears_every_slot_with_undo_redo_and_save() {
    let path = fixture(PesVersion::Pes21, "team_menu_clear_flags");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    let filled: Vec<u32> = file
        .team(701)
        .unwrap()
        .roster
        .iter()
        .map(|slot| slot.player_id)
        .filter(|&id| id != 0)
        .collect();
    for &id in &filled {
        let flags = &mut file.player_mut(id).unwrap().edit_flags;
        flags.face = true;
        flags.hair = true;
        flags.physique = true;
        flags.strip = true;
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    let cleared = |view: &SaveView, id: u32| {
        let flags = &player_entry(view, id).edit_flags;
        !(flags.face || flags.hair || flags.physique || flags.strip)
    };
    let all_set = |view: &SaveView, id: u32| {
        let flags = &player_entry(view, id).edit_flags;
        flags.face && flags.hair && flags.physique && flags.strip
    };

    let nodes = open_team_menu(&mut window, &mut view);
    let item = nodes
        .iter()
        .find(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Clear visual flags"))
        })
        .expect("the Clear visual flags item");
    assert!(!item.is_disabled(), "unlocked, the item is enabled");
    click(&mut window, item, &mut view);
    let slots = rostered(&view, 701);
    for &(_, id) in &slots {
        assert!(cleared(&view, id), "{id}'s visual flags cleared");
    }

    // One undo entry restores them; redo clears them again.
    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");
    for &id in &filled {
        assert!(all_set(&view, id), "{id}'s four flags restored");
    }
    window.frame(vec![key(egui::Key::Y, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    for &(_, id) in &slots {
        assert!(cleared(&view, id), "{id}'s flags cleared again");
    }

    // Save; the reopened file holds them cleared.
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let reopened = EditFile::load(&path).unwrap();
    for &(_, id) in &slots {
        let flags = &reopened.player(id).unwrap().edit_flags;
        assert!(
            !(flags.face || flags.hair || flags.physique || flags.strip),
            "{id}'s flags cleared in the saved file"
        );
    }
}
