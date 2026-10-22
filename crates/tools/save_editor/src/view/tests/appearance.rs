use pes_savefile::model::PlayerMotion;
use pes_savefile::model::player::STRIPPED_APPEARANCE_ID;
use pes_savefile::schema::ingame_face::IngameFaceField;

use super::*;

/// The node of the Appearance tab's "Edit aesthetics anyway" switch.
fn edit_anyway(nodes: &[accesskit::Node]) -> &accesskit::Node {
    nodes
        .iter()
        .find(|node| {
            node.role() == Role::CheckBox && node.label() == Some("Edit aesthetics anyway")
        })
        .expect("the Edit aesthetics anyway switch")
}

/// Whether the tab draws a Label node with `value` as its text.
fn label_drawn(nodes: &[accesskit::Node], value: &str) -> bool {
    nodes
        .iter()
        .any(|node| node.role() == Role::Label && node.value() == Some(value))
}

/// The locked tab drawn on `version`: a fresh `SaveView` on the Appearance
/// tab of `version`'s fixture, one frame's nodes in, window and view back.
fn locked_tab(version: PesVersion, test: &str) -> (SaveView, Headless, Vec<accesskit::Node>) {
    let path = fixture(version, test);
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    (view, window, nodes)
}

// SE-GUI-07
#[test]
fn the_tab_is_locked_on_a_fox_save_until_the_switch_unlocks_it() {
    let (mut view, mut window, nodes) = locked_tab(PesVersion::Pes21, "appearance_lock_fox");

    // Locked: the Fox ownership line, every control of the groups disabled,
    // the switch the one control that is not.
    assert!(
        label_drawn(
            &nodes,
            "Owned by the team's export: edit its settings.toml and recompile."
        ),
        "the Fox ownership line"
    );
    // The self-labeled controls (checkboxes, the switch, a swatch, the
    // button) and the label/pair ones (a Label node beside its control).
    for (label, role) in [
        ("Face", Role::CheckBox),
        ("Motion", Role::CheckBox),
        ("Player gloves", Role::CheckBox),
        ("Edit aesthetics anyway", Role::CheckBox),
        ("Skin Light", Role::ColorWell),
        ("Randomize", Role::Button),
    ] {
        let control = nodes
            .iter()
            .find(|node| node.role() == role && node.label() == Some(label))
            .unwrap_or_else(|| panic!("the {label} control"));
        assert_eq!(
            control.is_disabled(),
            label != "Edit aesthetics anyway",
            "{label}"
        );
    }
    for (label, role) in [
        ("Chest", Role::SpinButton),
        ("Corner kick", Role::SpinButton),
        ("Goal celebration 1", Role::SpinButton),
        ("Boots ID", Role::SpinButton),
        ("Copy from ID", Role::SpinButton),
        ("Sleeves", Role::ComboBox),
    ] {
        let control = right_of_label(&nodes, label, role)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("the {label} control"));
        assert!(control.is_disabled(), "{label}");
    }

    // On: the note says a stripped player's look comes from the tables
    // while his motions are still read, and the controls draw enabled.
    click(&mut window, edit_anyway(&nodes), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        label_drawn(
            &nodes,
            "On a stripped save the game still reads the motions; the look comes from the database tables."
        ),
        "the Fox note under the switch"
    );
    let chest = right_of_label(&nodes, "Chest", Role::SpinButton)
        .first()
        .copied()
        .expect("the Chest control");
    assert!(!chest.is_disabled(), "unlocked, Chest is enabled");

    // A physique edit lands in the session's player as one undo step
    // (shown -7 to 7, stored +7: typed 5 stores 12).
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .appearance
            .chest,
        7,
        "premise: Chest stores 7 (shown 0)"
    );
    click(&mut window, chest, &mut view);
    window.frame(vec![egui::Event::Text("5".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .appearance
            .chest,
        12,
        "typed 5 stores 12"
    );
    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");

    // A fresh view (the tool's next start) is locked again; the same view
    // opening another save keeps the switch on.
    let path = fixture(PesVersion::Pes21, "appearance_lock_fox_fresh");
    let mut fresh = SaveView::new();
    fresh.open(&path);
    fresh.tab = Tab::Appearance;
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut fresh, &|| Vec::new(), None));
    assert!(
        right_of_label(&nodes, "Chest", Role::SpinButton)[0].is_disabled(),
        "the next start is locked again"
    );

    let other = fixture(PesVersion::Pes21, "appearance_lock_fox_other");
    view.open(&other);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        !right_of_label(&nodes, "Chest", Role::SpinButton)[0].is_disabled(),
        "opening another save keeps it unlocked"
    );
}

// SE-GUI-07
#[test]
fn the_tab_is_locked_on_a_pre_fox_save_with_the_patch_step() {
    let (mut view, mut window, nodes) = locked_tab(PesVersion::Pes17, "appearance_lock_prefox");

    // Locked: the pre-Fox ownership line carries the patch step and the
    // Apply aesthetics patch link (disabled controls' hover is the same
    // sentence).
    assert!(
        label_drawn(
            &nodes,
            "Owned by the team's export: edit its settings.toml and recompile, then apply the patch."
        ),
        "the pre-Fox ownership line"
    );
    assert!(
        nodes
            .iter()
            .any(|node| node.label() == Some("Apply aesthetics patch")),
        "the patch link"
    );
    assert!(
        right_of_label(&nodes, "Chest", Role::SpinButton)[0].is_disabled(),
        "locked on PES 17 too"
    );
    let ownership_hover = right_of_label(&nodes, "Chest", Role::SpinButton)[0];
    assert_eq!(
        tooltip_containing(&mut window, &mut view, centre(ownership_hover), "Owned by"),
        Some(
            "Owned by the team's export: edit its settings.toml and recompile, then apply the patch."
                .to_string()
        ),
        "the disabled controls' hover is the ownership line"
    );

    // On: the note warns a later patch overwrites hand edits.
    click(&mut window, edit_anyway(&nodes), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        label_drawn(
            &nodes,
            "A later patch overwrites whatever is edited by hand."
        ),
        "the pre-Fox note under the switch"
    );
    assert!(
        !right_of_label(&nodes, "Chest", Role::SpinButton)[0].is_disabled(),
        "unlocked on PES 17 too"
    );
    // PES 17 still has the base-copy flag, enabled with the rest.
    let base_copy = nodes
        .iter()
        .find(|node| node.role() == Role::CheckBox && node.label() == Some("Base copy"))
        .expect("the Base copy flag on PES 17");
    assert!(!base_copy.is_disabled(), "PES 17 holds the flag");
}

// SE-GUI-07 (its THEN's last clause)
#[test]
fn a_stripped_players_tab_says_the_look_is_the_tables_and_the_motions_are_read() {
    let path = fixture(PesVersion::Pes21, "appearance_stripped_line");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().appearance_id = Some(STRIPPED_APPEARANCE_ID);
    file.save(&path).unwrap();
    const LINE: &str =
        "Stripped: the game takes his look from the database tables; the motions are still read.";

    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    let mut window = Headless::sized(1100.0, 700.0);
    // 70101 is the selected player; every other player's id is set while
    // his is -1, so the Strip again window opened too: close it first.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let close = nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some("Close"))
        .expect("the Strip again window's Close");
    click(&mut window, close, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(label_drawn(&nodes, LINE), "70101's tab marks him stripped");

    // Another player's tab does not draw the line.
    view.selected = Some((701, 1));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        !label_drawn(&nodes, LINE),
        "70102's tab draws no stripped line"
    );
}

// plans/save_editor.md "Editing UI", the motion paragraph (DECISIONS.md
// 2026-10-22): Randomize draws every motion but the dribbling one from 1
// to the version's shown top, one undoable edit. The pinned source lands
// each at its top, so the assertions name the stored tops exactly.
#[test]
fn randomize_draws_within_pes21s_measured_tops_one_undo_entry() {
    let path = fixture(PesVersion::Pes21, "appearance_random_21");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    view.randomize_draw = Box::new(|top| top);
    let mut window = Headless::sized(1100.0, 700.0);
    let motion_of = |view: &SaveView| -> PlayerMotion {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .motion
            .clone()
    };
    let before = motion_of(&view);
    assert_eq!(before.dribbling, Some(0), "premise: the dribbling motion");
    assert_ne!(before.goal_celebration_1, 162, "premise: gc1 starts else");

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let button = nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some("Randomize"))
        .expect("the Randomize button");
    assert!(!button.is_disabled(), "unlocked the button is enabled");
    click(&mut window, button, &mut view);

    // The pinned draw returns the shown top, so every motion lands at its
    // measured stored top; the dribbling motion is untouched.
    let motion = motion_of(&view);
    assert_eq!(motion.hunching_dribbling, 4);
    assert_eq!(motion.hunching_running, 4);
    assert_eq!(motion.arm_movement_dribbling, 9);
    assert_eq!(motion.arm_movement_running, 9);
    assert_eq!(motion.corner_kick, 9);
    assert_eq!(motion.free_kick, 19);
    assert_eq!(motion.penalty_kick, 6);
    assert_eq!(motion.goal_celebration_1, 162);
    assert_eq!(motion.goal_celebration_2, 162);
    assert_eq!(motion.dribbling, Some(0), "the dribbling motion is left");

    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");
    assert_eq!(motion_of(&view), before, "the one undo restores them all");
}

// The same rule on a version with no dribbling motion at all (PES 15's
// measured tops are the small ones).
#[test]
fn randomize_draws_within_pes15s_measured_tops_one_undo_entry() {
    let path = fixture(PesVersion::Pes15, "appearance_random_15");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    view.randomize_draw = Box::new(|top| top);
    let mut window = Headless::sized(1100.0, 700.0);
    let motion_of = |view: &SaveView| -> PlayerMotion {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .motion
            .clone()
    };
    let before = motion_of(&view);
    assert_eq!(before.dribbling, None, "PES 15 has no dribbling motion");

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    // Its stand-in control is disabled with the version tooltip.
    let dribbling = right_of_label(&nodes, "Dribbling motion", Role::SpinButton)
        .into_iter()
        .find(|node| node.is_disabled())
        .expect("the disabled dribbling motion");
    assert_eq!(
        tooltip_after_hover(&mut window, &mut view, centre(dribbling)),
        Some("PES 20+".to_string()),
        "the dribbling motion's tooltip"
    );
    let button = nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some("Randomize"))
        .expect("the Randomize button");
    click(&mut window, button, &mut view);

    let motion = motion_of(&view);
    assert_eq!(motion.hunching_dribbling, 2);
    assert_eq!(motion.hunching_running, 2);
    assert_eq!(motion.arm_movement_dribbling, 4);
    assert_eq!(motion.arm_movement_running, 4);
    assert_eq!(motion.corner_kick, 5);
    assert_eq!(motion.free_kick, 15);
    assert_eq!(motion.penalty_kick, 3);
    assert_eq!(motion.goal_celebration_1, 112);
    assert_eq!(motion.goal_celebration_2, 112);
    assert_eq!(motion.dribbling, None, "still none on PES 15");

    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");
    assert_eq!(motion_of(&view), before, "the one undo restores them all");
}

// plans/save_editor.md "Editing UI", the Abilities rule applied to the
// tab: a value the save holds outside its range draws as stored and
// changes only when edited (the egui clamping trap stays off).
#[test]
fn a_stored_celebration_past_its_top_draws_as_stored_without_marking_the_player() {
    let path = fixture(PesVersion::Pes21, "appearance_celebration_200");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().motion.goal_celebration_1 = 200;
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    for _ in 0..5 {
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    }
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let celebration = right_of_label(&nodes, "Goal celebration 1", Role::SpinButton)
        .first()
        .copied()
        .expect("the celebration control");
    assert_eq!(
        celebration.value(),
        Some("200"),
        "the control draws the stored 200"
    );
    let session = view.session.as_ref().unwrap();
    assert!(!session.is_dirty(), "drawing left no unsaved changes");
    assert!(!session.can_undo(), "drawing pushed no undo entry");
    assert_eq!(
        session
            .file()
            .player(70101)
            .unwrap()
            .motion
            .goal_celebration_1,
        200,
        "the value stands as stored"
    );
}

// SE-GUI-20
#[test]
fn a_fox_save_with_some_ids_set_lists_them_with_strip_again() {
    let path = fixture(PesVersion::Pes21, "strip_again_window");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    for player in file.players_mut() {
        player.appearance_id = Some(STRIPPED_APPEARANCE_ID);
    }
    // Team 702's first three rostered players are the ones left set: their
    // rows' team names are 702's own, which is not the first team the save
    // lists (so a wrong-team lookup shows).
    let listed_ids: Vec<u32> = file
        .team(702)
        .unwrap()
        .roster
        .iter()
        .map(|slot| slot.player_id)
        .filter(|&id| id != 0)
        .take(3)
        .collect();
    for &id in &listed_ids {
        file.player_mut(id).unwrap().appearance_id = Some(id);
    }
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // The window lists exactly the three restored players, by id: the
    // five-digit id labels under the heading (the info strip draws the
    // selected player's id too, elsewhere on screen).
    let heading = nodes
        .iter()
        .find(|node| node.value() == Some("Appearance ids set on a stripped save"))
        .expect("the Strip again window");
    let heading_y = heading.bounds().expect("bounds").y0;
    let mut listed: Vec<&str> = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::Label
                && node.bounds().is_some_and(|bounds| bounds.y0 > heading_y)
                && node.value().is_some_and(|value| {
                    value.len() == 5 && value.bytes().all(|byte| byte.is_ascii_digit())
                })
        })
        .filter_map(|node| node.value())
        .collect();
    listed.sort();
    let expected_ids: Vec<String> = listed_ids.iter().map(u32::to_string).collect();
    assert_eq!(listed, expected_ids, "exactly the three");
    // Each row names the team whose roster holds its player: team 702's
    // "/a/", three times.
    let team_labels = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::Label
                && node.bounds().is_some_and(|bounds| bounds.y0 > heading_y)
                && node.value() == Some("/a/")
        })
        .count();
    assert_eq!(team_labels, 3, "each row's team name is its player's team");
    assert!(
        label_drawn(
            &nodes,
            "3 player(s) have an appearance id set while the rest are stripped:"
        ),
        "the window's count line"
    );

    // Strip again: all three back to -1, one undo restoring them.
    let strip = nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some("Strip again"))
        .expect("the Strip again button");
    click(&mut window, strip, &mut view);
    for &id in &listed_ids {
        assert_eq!(
            view.session
                .as_ref()
                .unwrap()
                .file()
                .player(id)
                .unwrap()
                .appearance_id,
            Some(STRIPPED_APPEARANCE_ID),
            "{id} is stripped"
        );
    }
    assert!(view.session.as_ref().unwrap().is_dirty());
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        !nodes.iter().any(|node| node.label() == Some("Strip again")),
        "the window closed"
    );

    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");
    for &id in &listed_ids {
        assert_eq!(
            view.session
                .as_ref()
                .unwrap()
                .file()
                .player(id)
                .unwrap()
                .appearance_id,
            Some(id),
            "{id} is restored"
        );
    }
}

// The same rule's low end: a draw of 1 stores 0 in a 1-based motion and 1
// in a celebration (a celebration is never OFF), the dribbling motion
// unchanged.
#[test]
fn randomize_low_end_stores_zero_for_one_based_and_one_for_celebrations() {
    let path = fixture(PesVersion::Pes21, "appearance_random_low");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    view.randomize_draw = Box::new(|_| 1);
    let mut window = Headless::sized(1100.0, 700.0);
    let motion_of = |view: &SaveView| -> PlayerMotion {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .motion
            .clone()
    };

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let button = nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some("Randomize"))
        .expect("the Randomize button");
    click(&mut window, button, &mut view);

    let motion = motion_of(&view);
    for (label, value) in [
        ("hunching dribbling", motion.hunching_dribbling),
        ("hunching running", motion.hunching_running),
        ("arm dribbling", motion.arm_movement_dribbling),
        ("arm running", motion.arm_movement_running),
        ("corner", motion.corner_kick),
        ("free", motion.free_kick),
        ("penalty", motion.penalty_kick),
    ] {
        assert_eq!(value, 0, "{label} stores 0 for a drawn 1");
    }
    assert_eq!(motion.goal_celebration_1, 1, "a celebration stores 1");
    assert_eq!(motion.goal_celebration_2, 1, "a celebration stores 1");
    assert_eq!(motion.dribbling, Some(0), "the dribbling motion is left");
}

// plans/save_editor.md "Editing UI", the physique bullet's scale: a `Signed`
// key shows `stored - limit` and stores `shown + limit` (limit 7). Stored
// 12 draws 5; typing 3 stores 10.
#[test]
fn a_physique_field_shows_stored_minus_limit_and_stores_shown_plus_it() {
    let path = fixture(PesVersion::Pes21, "physique_scale");
    // A stored value off the neutral 7, so `- limit` and `/ limit` differ.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().appearance.neck_length = 12;
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    let neck_length = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .appearance
            .neck_length
    };
    assert_eq!(neck_length(&view), 12, "premise: stored 12");
    let mut window = Headless::sized(1100.0, 700.0);

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let neck = right_of_label(&nodes, "Neck length", Role::SpinButton)
        .first()
        .copied()
        .expect("the Neck length control");
    assert_eq!(neck.value(), Some("5"), "stored 12 shows 5");
    click(&mut window, neck, &mut view);
    window.frame(vec![egui::Event::Text("3".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(neck_length(&view), 10, "typed 3 stores 3 + 7");
}

// plans/save_editor.md "Editing UI", the palette bullet: a swatch click
// writes its index as one undoable edit.
#[test]
fn a_swatch_click_stores_its_index() {
    let path = fixture(PesVersion::Pes21, "appearance_swatch");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    let skin_of = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .appearance
            .ingame_face
            .get(IngameFaceField::SkinColor)
            .unwrap()
    };
    assert_eq!(skin_of(&view), 1, "premise: skin is light (1)");

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let swatch = nodes
        .iter()
        .find(|node| node.role() == Role::ColorWell && node.label() == Some("Skin Brown"))
        .expect("the brown swatch");
    assert!(!swatch.is_disabled(), "unlocked the swatch is enabled");
    click(&mut window, swatch, &mut view);

    assert_eq!(skin_of(&view), 5, "brown stores 5");
    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");
    assert_eq!(skin_of(&view), 1, "the undo restores it");
}

// The motion fields' scales (`motion_number`): a 1-based motion shows
// `stored + 1` and its range ends at the version's shown top (a typed value
// past it clamps there), storing `shown - 1`; a celebration shows as stored.
#[test]
fn a_one_based_motion_shows_one_above_stored_within_the_measured_top() {
    let path = fixture(PesVersion::Pes21, "motion_one_based");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    let motion_of = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .motion
            .clone()
    };
    let stored = motion_of(&view).free_kick;

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let free = right_of_label(&nodes, "Free kick", Role::SpinButton)
        .first()
        .copied()
        .expect("the Free kick control");
    assert_eq!(
        free.value(),
        Some((stored + 1).to_string()).as_deref(),
        "a 1-based motion shows stored + 1"
    );
    let celebration = right_of_label(&nodes, "Goal celebration 1", Role::SpinButton)
        .first()
        .copied()
        .expect("the Goal celebration 1 control");
    assert_eq!(
        celebration.value(),
        Some(motion_of(&view).goal_celebration_1.to_string()).as_deref(),
        "a celebration shows as stored"
    );

    // Typing 7 stores 6; typing past the shown top clamps to it (PES 21's
    // free-kick top 19, shown 20) and stores 19.
    click(&mut window, free, &mut view);
    window.frame(vec![egui::Event::Text("7".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(motion_of(&view).free_kick, 6, "typed 7 stores 6");
    click(&mut window, free, &mut view);
    window.frame(vec![egui::Event::Text("99".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(
        motion_of(&view).free_kick,
        19,
        "past the shown top it stops at 20, stored 19"
    );
}

// A combo pick stores the option's index (`combo`'s `picked != stored`
// write): Sleeves' "Long" is index 2, one undo entry.
#[test]
fn a_combo_pick_stores_the_options_index_as_one_undo_entry() {
    let path = fixture(PesVersion::Pes21, "combo_sleeves");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    let sleeves = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .appearance
            .sleeves
    };
    assert_ne!(sleeves(&view), 2, "premise: not Long");
    let mut window = Headless::sized(1100.0, 700.0);

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let combo = right_of_label(&nodes, "Sleeves", Role::ComboBox)
        .first()
        .copied()
        .expect("the Sleeves combo");
    click(&mut window, combo, &mut view);
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (item, _) = nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Long"))
        .expect("the Long option");
    action_click(&mut window, &mut view, *item);

    assert_eq!(sleeves(&view), 2, "Long stores 2");
    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");
}

// The Player gloves checkbox draws the face run's PlayerGloves flag: 1
// ticked, 0 not (`face(..., PlayerGloves) != 0`).
#[test]
fn the_player_gloves_checkbox_draws_the_runs_flag() {
    let path = fixture(PesVersion::Pes21, "gloves_checkbox");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    for (id, gloves) in [(70101u32, 1u8), (70102, 0)] {
        file.player_mut(id)
            .unwrap()
            .appearance
            .ingame_face
            .set(IngameFaceField::PlayerGloves, gloves)
            .unwrap();
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    fn gloves_box(nodes: &[accesskit::Node]) -> &accesskit::Node {
        nodes
            .iter()
            .find(|node| node.role() == Role::CheckBox && node.label() == Some("Player gloves"))
            .expect("the Player gloves checkbox")
    }

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        gloves_box(&nodes).toggled(),
        Some(accesskit::Toggled::True),
        "70101's run flag 1 ticks it"
    );
    view.selected = Some((701, 1));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        gloves_box(&nodes).toggled(),
        Some(accesskit::Toggled::False),
        "70102's run flag 0 unticks it"
    );
}

/// The swatches of the Skin row (`Role::ColorWell`, labeled "Skin …"),
/// in x order.
fn skin_swatches(nodes: &[accesskit::Node]) -> Vec<&accesskit::Node> {
    let mut swatches: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::ColorWell
                && node.label().is_some_and(|label| label.starts_with("Skin "))
        })
        .collect();
    swatches.sort_by(|a, b| {
        a.bounds()
            .expect("bounds")
            .x0
            .partial_cmp(&b.bounds().expect("bounds").x0)
            .expect("ordered bounds")
    });
    swatches
}

/// A Label node with `value` on `row`'s y (within four points).
fn label_on_row<'a>(
    nodes: &'a [accesskit::Node],
    row: &accesskit::Node,
    value: &str,
) -> Option<&'a accesskit::Node> {
    let y = row.bounds().expect("the row's bounds").y0;
    nodes.iter().find(|node| {
        node.role() == Role::Label
            && node.value() == Some(value)
            && node.bounds().is_some_and(|b| (b.y0 - y).abs() < 4.0)
    })
}

// The Skin palette draws the palette's entries: on Fox the Custom entry
// is dropped (0 to 6), on PES 17 the whole palette (0 to 7); the held
// index's swatch reports selected; a stored value past the top draws its
// number after the swatches, one at the top none.
#[test]
fn the_skin_palette_draws_its_entries_held_swatch_and_a_past_top_number() {
    let path = fixture(PesVersion::Pes21, "palette_skin");
    // Held at 3 ("Skin Medium"), then past the top (7, a PES 21 save can
    // hold it) for the number case.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101)
        .unwrap()
        .appearance
        .ingame_face
        .set(IngameFaceField::SkinColor, 3)
        .unwrap();
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let swatches = skin_swatches(&nodes);
    assert_eq!(
        swatches.len(),
        7,
        "Fox draws entries 0 to 6, Custom dropped"
    );
    assert!(
        !swatches.iter().any(|sw| sw.label() == Some("Skin Custom")),
        "no Custom swatch on PES 21"
    );
    let held: Vec<&accesskit::Node> = swatches
        .iter()
        .copied()
        .filter(|sw| sw.toggled() == Some(accesskit::Toggled::True))
        .collect();
    assert_eq!(held.len(), 1, "exactly the held swatch is selected");
    assert_eq!(held[0].label(), Some("Skin Medium"), "skin 3 held");

    // A stored value past the top draws its number after the swatches.
    file.player_mut(70101)
        .unwrap()
        .appearance
        .ingame_face
        .set(IngameFaceField::SkinColor, 7)
        .unwrap();
    file.save(&path).unwrap();
    view.open(&path);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let swatches = skin_swatches(&nodes);
    assert!(
        label_on_row(&nodes, swatches[0], "7").is_some(),
        "the past-top value's number draws"
    );
    // One at the top draws none.
    file.player_mut(70101)
        .unwrap()
        .appearance
        .ingame_face
        .set(IngameFaceField::SkinColor, 6)
        .unwrap();
    file.save(&path).unwrap();
    view.open(&path);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let swatches = skin_swatches(&nodes);
    assert!(
        label_on_row(&nodes, swatches[0], "6").is_none(),
        "a value at the top draws no number"
    );

    // Pre-Fox draws the whole palette, Custom included.
    let path17 = fixture(PesVersion::Pes17, "palette_skin_17");
    let mut view17 = SaveView::new();
    view17.open(&path17);
    view17.tab = Tab::Appearance;
    view17.edit_aesthetics = true;
    let mut window17 = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) =
        window17.frame(Vec::new(), |ui| show(ui, &mut view17, &|| Vec::new(), None));
    let swatches = skin_swatches(&nodes);
    assert_eq!(swatches.len(), 8, "PES 17 draws the whole palette");
    assert!(
        swatches.iter().any(|sw| sw.label() == Some("Skin Custom")),
        "Custom on PES 17"
    );
}

// The strip and motion grids break a row after `pairs_per_row` pairs (190
// each, so four at this pane's width): the first four controls of each
// share one row, the fifth wraps below — the same rule
// `the_stats_rows_wrap_at_the_panes_width` pins for the stats.
#[test]
fn the_strip_and_motion_grids_wrap_after_the_fitted_pairs() {
    let path = fixture(PesVersion::Pes21, "grid_wraps");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let y = |label: &str| control_right_of(&nodes, label).bounds().expect("bounds").y0;

    let first = y("Boots ID");
    for label in ["Gloves ID", "Sleeves", "Inners"] {
        assert!((y(label) - first).abs() < 1.0, "{label} on the first row");
    }
    assert!(y("Socks") > first + 5.0, "Socks wraps below");

    let first = y("Hunching (dribbling)");
    for label in [
        "Hunching (running)",
        "Arm movement (dribbling)",
        "Arm movement (running)",
    ] {
        assert!((y(label) - first).abs() < 1.0, "{label} on the first row");
    }
    assert!(y("Corner kick") > first + 5.0, "Corner kick wraps below");
}

// The id controls take the record's own bound (`player_max_value`, the
// codec's 14-bit field — wider than `settings.toml`'s 0 to 100 stock
// range): a compile-allocated 627 types and stores.
#[test]
fn the_boots_id_control_takes_the_records_bound() {
    let path = fixture(PesVersion::Pes21, "boots_id_627");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let boots = right_of_label(&nodes, "Boots ID", Role::SpinButton)
        .first()
        .copied()
        .expect("the Boots ID control");
    click(&mut window, boots, &mut view);
    window.frame(vec![egui::Event::Text("627".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .appearance
            .boots_id,
        627,
        "a compile-allocated id stores"
    );
}

// plans/save_editor.md "Feature inventory", the "Is a copy?" row: ticking
// Base copy sets the seven gameplay edit flags in the same edit (a copy's
// sections count as edited); unticking clears `base_copy` alone. On PES
// 18, the last version with the flag.
#[test]
fn ticking_base_copy_sets_the_seven_gameplay_flags_one_undoable_edit() {
    let path = fixture(PesVersion::Pes18, "base_copy_flags");
    // The eight cleared, so each set bit is the tick's doing.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes18)).unwrap();
    {
        let flags = &mut file.player_mut(70101).unwrap().edit_flags;
        flags.base_copy = Some(false);
        flags.abilities = false;
        flags.com_styles = false;
        flags.motion = false;
        flags.playable_positions = false;
        flags.registered_position = false;
        flags.skills = false;
        flags.playing_style = false;
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Appearance;
    view.edit_aesthetics = true;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(1100.0, 700.0);
    let flags_of = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .edit_flags
            .clone()
    };
    let all_set = |view: &SaveView| {
        let flags = flags_of(view);
        flags.base_copy == Some(true)
            && flags.abilities
            && flags.com_styles
            && flags.motion
            && flags.playable_positions
            && flags.registered_position
            && flags.skills
            && flags.playing_style
    };
    assert!(!all_set(&view), "premise: the eight start cleared");
    let before = flags_of(&view);
    let base_copy_of = |view: &SaveView| flags_of(view).base_copy;
    let seven_set = |view: &SaveView| {
        let flags = flags_of(view);
        flags.abilities
            && flags.com_styles
            && flags.motion
            && flags.playable_positions
            && flags.registered_position
            && flags.skills
            && flags.playing_style
    };

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let checkbox = nodes
        .iter()
        .find(|node| node.role() == Role::CheckBox && node.label() == Some("Base copy"))
        .expect("the Base copy flag on PES 18");
    assert!(!checkbox.is_disabled(), "PES 18 holds the flag");
    click(&mut window, checkbox, &mut view);
    assert!(all_set(&view), "the tick sets base copy and the seven");

    // One undo step restores all eight.
    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");
    assert_eq!(flags_of(&view), before, "the undo restores all eight");

    // Ticked again, then unticked: `base_copy` alone clears.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let checkbox = nodes
        .iter()
        .find(|node| node.role() == Role::CheckBox && node.label() == Some("Base copy"))
        .expect("the Base copy flag");
    click(&mut window, checkbox, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let checkbox = nodes
        .iter()
        .find(|node| node.role() == Role::CheckBox && node.label() == Some("Base copy"))
        .expect("the Base copy flag");
    click(&mut window, checkbox, &mut view);
    assert_eq!(
        base_copy_of(&view),
        Some(false),
        "the untick clears base copy alone"
    );
    assert!(seven_set(&view), "the seven stay set");
}
