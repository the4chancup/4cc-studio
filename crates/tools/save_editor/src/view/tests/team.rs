use pes_savefile::model::TeamEntry;

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

/// The first node of `role` on the same row as the labeled node `anchor`
/// and right of it (a radio's companion control).
fn right_of<'a>(
    nodes: &'a [accesskit::Node],
    anchor: &accesskit::Node,
    role: Role,
) -> &'a accesskit::Node {
    let bounds = anchor.bounds().expect("the anchor's bounds");
    nodes
        .iter()
        .filter(|node| {
            node.role() == role
                && node
                    .bounds()
                    .is_some_and(|b| (b.y0 - bounds.y0).abs() < 4.0 && b.x0 >= bounds.x1 - 1.0)
        })
        .min_by(|a, b| {
            let x = |node: &&accesskit::Node| node.bounds().expect("bounds").x0;
            x(a).partial_cmp(&x(b)).expect("ordered bounds")
        })
        .expect("a control right of the anchor")
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

/// `player_id`'s entry as the session holds it.
fn player_entry(view: &SaveView, player_id: u32) -> PlayerEntry {
    view.session
        .as_ref()
        .unwrap()
        .file()
        .player(player_id)
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
    view.tab = Tab::Team;
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

    // The Name field: the tab's text input holding the team name (the info
    // strip's own Name field holds the player's).
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
            let y = |node: &&accesskit::Node| node.bounds().expect("bounds").y0;
            y(a).partial_cmp(&y(b)).expect("ordered bounds")
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
        "the tab shows #FF8200"
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
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes17)).unwrap();
    let slots = file
        .team_mut(701)
        .unwrap()
        .kit_slots
        .as_mut()
        .expect("PES 17 kit slots");
    slots[0].binding = 701 * 64 + 5;
    let binding = slots[0].binding;
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Team;
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
}

// plans/save_editor.md "Editing UI" (the version-gated bullet, on the Team
// tab): fields the version does not store draw disabled in the same place
// with the gate tooltip naming the first version that has them.
#[test]
fn on_pes16_the_colours_ids_and_kit_slots_draw_disabled_with_their_versions() {
    let path = fixture(PesVersion::Pes16, "team_tab_gated");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Team;
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
        view.tab = Tab::Team;
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

    // Ctrl+F under the lock: nothing changes.
    window.frame(vec![key(egui::Key::F, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(
        !view.session.as_ref().unwrap().is_dirty(),
        "Ctrl+F changed nothing"
    );

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
