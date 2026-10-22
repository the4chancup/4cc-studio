use pes_savefile::model::KitSlot;

use super::*;

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

/// The node of `role` with `label`, or panic naming both.
fn labeled<'a>(nodes: &'a [accesskit::Node], role: Role, label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("no {role:?} {label:?}"))
}

/// Whether the nodes draw a Label with `value` as its text.
fn label_value(nodes: &[accesskit::Node], value: &str) -> bool {
    nodes
        .iter()
        .any(|node| node.role() == Role::Label && node.value() == Some(value))
}

/// The Savefile menu's popup open for one frame: a click on the Savefile
/// button (skipped when a previous call left it open), then the frame's
/// nodes.
fn open_savefile_menu(window: &mut Headless, view: &mut SaveView) -> Vec<accesskit::Node> {
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    if nodes.iter().any(|node| {
        node.role() == Role::Button && node.label() == Some("Output rosters to TSV\u{2026}")
    }) {
        return nodes;
    }
    let button = nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some("Savefile"))
        .expect("the Savefile menu button");
    click(window, button, view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    nodes
}

// SE-GUI-27
#[test]
fn fix_database_repairs_the_save_behind_a_confirmation_and_one_undo() {
    let path = fixture(PesVersion::Pes17, "savefile_fix_database");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes17)).unwrap();
    // The three base-copy cases: the flag off with another player's id
    // (the id clears), the flag on with his own id (the flag clears), the
    // flag on with another player's id (a real base copy, untouched).
    {
        let player = file.player_mut(70101).unwrap();
        player.edit_flags.base_copy = Some(false);
        player.appearance.base_copy_id = 70102;
    }
    {
        let player = file.player_mut(70102).unwrap();
        player.edit_flags.base_copy = Some(true);
        player.appearance.base_copy_id = 70102;
    }
    {
        let player = file.player_mut(70103).unwrap();
        player.edit_flags.base_copy = Some(true);
        player.appearance.base_copy_id = 70104;
    }
    for id in [70101u32, 70102] {
        let flags = &mut file.player_mut(id).unwrap().edit_flags;
        flags.face = true;
        flags.hair = true;
        flags.physique = true;
        flags.strip = true;
    }
    file.team_mut(701).unwrap().edit_flags.strip = Some(true);
    file.team_mut(701)
        .unwrap()
        .kit_slots
        .as_mut()
        .expect("PES 17 kit slots")[0] = KitSlot {
        number: 5,
        binding: 12345,
    };
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);

    // The menu item opens the confirmation; Cancel leaves the session clean.
    let nodes = open_savefile_menu(&mut window, &mut view);
    click(
        &mut window,
        labeled(&nodes, Role::Button, "Fix database\u{2026}"),
        &mut view,
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        label_value(
            &nodes,
            "Repair base copies, clear every player's visual settings flags and the teams' \
             strip flag, and reset the kit slots to their defaults?"
        ),
        "the confirmation's question"
    );
    click(
        &mut window,
        labeled(&nodes, Role::Button, "Cancel"),
        &mut view,
    );
    assert!(
        !view.session.as_ref().unwrap().is_dirty(),
        "Cancel changed nothing"
    );
    assert!(
        !view.session.as_ref().unwrap().can_undo(),
        "Cancel pushed no undo entry"
    );

    // Fix database applies the whole repair.
    let nodes = open_savefile_menu(&mut window, &mut view);
    click(
        &mut window,
        labeled(&nodes, Role::Button, "Fix database\u{2026}"),
        &mut view,
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        labeled(&nodes, Role::Button, "Fix database"),
        &mut view,
    );
    assert_eq!(
        player_entry(&view, 70101).appearance.base_copy_id,
        0,
        "the flag-off stray copy id clears"
    );
    assert_eq!(
        player_entry(&view, 70102).edit_flags.base_copy,
        Some(false),
        "the baseless flag clears"
    );
    let third = player_entry(&view, 70103);
    assert_eq!(
        third.edit_flags.base_copy,
        Some(true),
        "a real base copy keeps its flag"
    );
    assert_eq!(third.appearance.base_copy_id, 70104, "and its copy id");
    for id in [70101u32, 70102] {
        let flags = &player_entry(&view, id).edit_flags;
        assert!(
            !(flags.face || flags.hair || flags.physique || flags.strip),
            "{id}'s visual flags cleared"
        );
    }
    let team = team_entry(&view, 701);
    assert_eq!(team.edit_flags.strip, Some(false), "the strip flag cleared");
    let slots = team.kit_slots.expect("PES 17 kit slots");
    assert_eq!(
        slots[0],
        KitSlot {
            number: 0,
            binding: 701 * 64
        }
    );
    assert_eq!(
        slots[1],
        KitSlot {
            number: 1,
            binding: 701 * 64
        }
    );
    assert_eq!(
        slots[2],
        KitSlot {
            number: 0x80,
            binding: 701 * 64
        }
    );
    for slot in &slots[3..] {
        assert_eq!(
            *slot,
            KitSlot {
                number: 0,
                binding: 0xFFFFC0
            }
        );
    }

    // One undo step restores every one of them.
    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");
    assert_eq!(player_entry(&view, 70101).appearance.base_copy_id, 70102);
    assert_eq!(player_entry(&view, 70102).edit_flags.base_copy, Some(true));
    for id in [70101u32, 70102] {
        let flags = &player_entry(&view, id).edit_flags;
        assert!(
            flags.face && flags.hair && flags.physique && flags.strip,
            "{id}'s flags restored"
        );
    }
    let team = team_entry(&view, 701);
    assert_eq!(team.edit_flags.strip, Some(true), "the strip flag restored");
    assert_eq!(
        team.kit_slots.expect("kit slots")[0],
        KitSlot {
            number: 5,
            binding: 12345
        },
        "the kit slot restored"
    );

    // Redo, save, reopen: the file holds the repair.
    window.frame(vec![key(egui::Key::Y, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let reopened = EditFile::load(&path).unwrap();
    assert_eq!(reopened.player(70101).unwrap().appearance.base_copy_id, 0);
    assert_eq!(
        reopened.player(70102).unwrap().edit_flags.base_copy,
        Some(false)
    );
    assert_eq!(
        reopened.player(70103).unwrap().edit_flags.base_copy,
        Some(true)
    );
    assert_eq!(
        reopened.player(70103).unwrap().appearance.base_copy_id,
        70104
    );
    assert_eq!(
        reopened.team(701).unwrap().edit_flags.strip,
        Some(false),
        "the saved strip flag"
    );
    assert_eq!(
        reopened.team(701).unwrap().kit_slots.expect("kit slots")[0],
        KitSlot {
            number: 0,
            binding: 701 * 64
        },
        "the saved kit slot"
    );
}

// SE-GUI-22 (the Savefile menu's part): under the lock Fix database is
// disabled with the ownership line as its hover, while the read-only TSV
// item stays enabled.
#[test]
fn the_locked_savefile_menu_disables_fix_database_only() {
    let path = fixture(PesVersion::Pes21, "savefile_menu_locked");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let nodes = open_savefile_menu(&mut window, &mut view);

    let item = labeled(&nodes, Role::Button, "Fix database\u{2026}");
    assert!(
        item.is_disabled(),
        "Fix database is disabled under the lock"
    );
    assert_eq!(
        tooltip_containing(&mut window, &mut view, centre(item), "Owned by"),
        Some("Owned by the team's export: edit its settings.toml and recompile.".to_string()),
        "its hover is the ownership line"
    );
    assert!(
        !labeled(&nodes, Role::Button, "Output rosters to TSV\u{2026}").is_disabled(),
        "the TSV item stays enabled: it writes nothing of the save's"
    );
}
