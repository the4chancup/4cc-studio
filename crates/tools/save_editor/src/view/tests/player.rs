//! The Player menu's stat operations and the FPC reads and writes, end to
//! end through the headless window (`plans/save_editor.md` "Acceptance":
//! SE-GUI-24's four operations, SE-GUI-11's hide/show, the SE-GUI-22 lock's
//! FPC items, and the PES 15/16 appearance record's round-trip).

use pes_savefile::interchange::team_toml::labels::POSITIONS;
use pes_savefile::model::names::display_name;
use pes_savefile::model::{PlayerEntry, TeamEntry};

use super::*;

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

/// The node of `role` with `label`, or panic naming both.
fn labeled<'a>(nodes: &'a [accesskit::Node], role: Role, label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("no {role:?} {label:?}"))
}

/// The Button whose label starts with `label` (a menu item's label carries
/// its shortcut text after the name).
fn menu_item<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| {
            node.role() == Role::Button && node.label().is_some_and(|item| item.starts_with(label))
        })
        .unwrap_or_else(|| panic!("no menu item {label:?}"))
}

/// The toolbar menu button with `label` (the topmost button with it).
fn menu_button<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .filter(|node| node.role() == Role::Button && node.label() == Some(label))
        .min_by(|a, b| {
            let y = |node: &&accesskit::Node| node.bounds().map(|r| r.y0).unwrap_or(f64::MAX);
            y(a).partial_cmp(&y(b)).expect("ordered bounds")
        })
        .unwrap_or_else(|| panic!("the {label:?} menu button"))
}

/// `menu`'s popup open for one frame: a click on its button (skipped when
/// a previous call left `item` visible), then the frame's nodes.
fn open_menu(
    window: &mut Headless,
    view: &mut SaveView,
    menu: &str,
    item: &str,
) -> Vec<accesskit::Node> {
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    if nodes.iter().any(|node| {
        node.role() == Role::Button && node.label().is_some_and(|l| l.starts_with(item))
    }) {
        return nodes;
    }
    click(window, menu_button(&nodes, menu), view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    nodes
}

/// Replaces the focused text control's contents with `text`: End moves the
/// caret past the text, the backspaces clear it, then the characters land.
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

/// Opens the number dialog of `item` and types `value` into it, then OK.
fn run_stat_dialog(
    window: &mut Headless,
    view: &mut SaveView,
    item: &str,
    line: &str,
    value: &str,
) {
    let nodes = open_menu(window, view, "Player", "Set all stats");
    click(window, menu_item(&nodes, item), view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    let field = right_of_label(&nodes, line, Role::SpinButton)
        .first()
        .copied()
        .unwrap_or_else(|| panic!("the {item:?} dialog's value field"));
    click(window, field, view);
    type_text(window, view, value);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    click(window, labeled(&nodes, Role::Button, "OK"), view);
}

/// The line `stat_num` draws for `name`, as the dialog's field anchors on it.
fn set_line(name: &str) -> String {
    format!("Every ability stat of {name} to the value")
}

/// The Bump dialog's line for `name`.
fn bump_line(name: &str) -> String {
    format!("Add to every ability stat of {name}, clamped to 40 to 99")
}

/// Every ability of `player` is `value` (the ones the version has).
fn assert_abilities(player: &PlayerEntry, value: u8) {
    for (index, ability) in player.stats.abilities().iter().enumerate() {
        if let Some(ability) = ability {
            assert_eq!(*ability, value, "ability {index}");
        }
    }
}

// The Set and Bump rows: Set all stats writes every ability the
// version has and nothing else (no edit flag, weak foot, form and injury
// untouched); Bump adds N clamped to 40 to 99, the dialog remembered and N
// read whole (a -10 is a bump of -10, not -1); each one undoable step.
// SE-GUI-24
#[test]
fn set_and_bump_stats_write_every_ability_as_one_undoable_edit() {
    let path = fixture(PesVersion::Pes21, "player_set_bump");
    // 70102's first ability off the fixture's, for the negative bump.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70102).unwrap().stats.attacking_prowess = 60;
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let before = player_entry(&view, 70101);
    assert_ne!(
        before.stats.attacking_prowess, 75,
        "premise: not already 75"
    );

    // Set all stats to 75 through the dialog (the field is prefilled with
    // his Attacking Prowess, the first ability).
    let name = display_name(&before.name);
    run_stat_dialog(
        &mut window,
        &mut view,
        "Set all stats",
        &set_line(&name),
        "75",
    );
    let set = player_entry(&view, 70101);
    assert_abilities(&set, 75);
    assert_eq!(set.stats.weak_foot_usage, before.stats.weak_foot_usage);
    assert_eq!(
        set.stats.weak_foot_accuracy,
        before.stats.weak_foot_accuracy
    );
    assert_eq!(set.stats.form, before.stats.form, "form is no ability");
    assert_eq!(
        set.stats.injury_resistance, before.stats.injury_resistance,
        "injury is no ability"
    );
    assert_eq!(set.edit_flags, before.edit_flags, "no edit flag is set");

    // Set to 90, then bump +10 by Ctrl+B: every ability clamps to 99 (the
    // dialog's N runs -10 to 10, so the overshoot is what clamps).
    run_stat_dialog(
        &mut window,
        &mut view,
        "Set all stats",
        &set_line(&name),
        "90",
    );
    window.frame(vec![key(egui::Key::B, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let field = right_of_label(&nodes, &bump_line(&name), Role::SpinButton)
        .first()
        .copied()
        .expect("the Bump dialog's value field");
    click(&mut window, field, &mut view);
    type_text(&mut window, &mut view, "10");
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(&mut window, labeled(&nodes, Role::Button, "OK"), &mut view);
    assert_abilities(&player_entry(&view, 70101), 99);

    // Set to 45, bump -10 through the menu: every ability clamps to 40.
    run_stat_dialog(
        &mut window,
        &mut view,
        "Set all stats",
        &set_line(&name),
        "45",
    );
    run_stat_dialog(
        &mut window,
        &mut view,
        "Bump all stats",
        &bump_line(&name),
        "-10",
    );
    assert_abilities(&player_entry(&view, 70101), 40);
    let bumped = player_entry(&view, 70101);
    assert_eq!(bumped.stats.form, before.stats.form);
    assert_eq!(
        bumped.stats.injury_resistance,
        before.stats.injury_resistance
    );
    assert_eq!(bumped.edit_flags, before.edit_flags);

    // Five operations, five undo entries; the undos restore the player.
    assert_eq!(undo_entries(&mut window, &mut view), 5, "one entry each");
    assert_eq!(player_entry(&view, 70101), before, "the undos restore all");

    // Redo them all, save, reopen: the file holds the 40s.
    for _ in 0..5 {
        window.frame(vec![key(egui::Key::Y, egui::Modifiers::CTRL, true)], |ui| {
            show(ui, &mut view, &|| Vec::new(), None)
        });
    }
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_abilities(EditFile::load(&path).unwrap().player(70101).unwrap(), 40);

    // N is read whole: a -10 on 70102's 60 gives 50, not the 59 a
    // two-character read would leave.
    view.selected = Some((701, 1));
    let second_line = bump_line(&display_name(&player_entry(&view, 70102).name));
    run_stat_dialog(
        &mut window,
        &mut view,
        "Bump all stats",
        &second_line,
        "-10",
    );
    assert_eq!(
        player_entry(&view, 70102).stats.attacking_prowess,
        50,
        "-10 bumps by -10, not by -1"
    );
}

// The Set dialog's own clamp: the field's 40 to 99 range settles a typed
// 100 at 99 and a typed 30 at 40 (SE-GUI-24's "Set and Bump clamped to
// 40 to 99").
// SE-GUI-24
#[test]
fn the_set_dialog_clamps_the_typed_value_to_40_and_99() {
    let path = fixture(PesVersion::Pes21, "player_set_clamp");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let name = display_name(&player_entry(&view, 70101).name);
    for (typed, clamped) in [("100", 99u8), ("30", 40u8)] {
        run_stat_dialog(
            &mut window,
            &mut view,
            "Set all stats",
            &set_line(&name),
            typed,
        );
        assert_abilities(&player_entry(&view, 70101), clamped);
    }
}

// The Bump dialog's field prefills with the last N used in the window
// (`view.bump`): after a bump by 7 the reopened dialog's field shows 7.
// SE-GUI-24
#[test]
fn the_bump_dialog_prefills_the_last_n_used() {
    let path = fixture(PesVersion::Pes21, "player_bump_remember");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let name = display_name(&player_entry(&view, 70101).name);

    run_stat_dialog(
        &mut window,
        &mut view,
        "Bump all stats",
        &bump_line(&name),
        "7",
    );

    // Reopen the dialog: the field carries the last N, before anything
    // is typed.
    let nodes = open_menu(&mut window, &mut view, "Player", "Set all stats");
    click(&mut window, menu_item(&nodes, "Bump all stats"), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let field = right_of_label(&nodes, &bump_line(&name), Role::SpinButton)
        .first()
        .copied()
        .expect("the Bump dialog's value field");
    assert_eq!(field.value(), Some("7"), "the last bump is prefilled");
}

/// `entry` with the copy/swap field set taken from `source`: height,
/// weight, the whole stats, positions and skills, the free-kick motion
/// alone, and the eight gameplay edit flags (`plans/save_editor.md`
/// "Feature inventory", the Copy row's field set).
fn with_stats_of(entry: &PlayerEntry, source: &PlayerEntry) -> PlayerEntry {
    let mut entry = entry.clone();
    entry.basic.height = source.basic.height;
    entry.basic.weight = source.basic.weight;
    entry.stats = source.stats.clone();
    entry.positions = source.positions.clone();
    entry.skills = source.skills.clone();
    entry.motion.free_kick = source.motion.free_kick;
    entry.edit_flags.player = source.edit_flags.player;
    entry.edit_flags.basic_settings = source.edit_flags.basic_settings;
    entry.edit_flags.registered_position = source.edit_flags.registered_position;
    entry.edit_flags.playable_positions = source.edit_flags.playable_positions;
    entry.edit_flags.abilities = source.edit_flags.abilities;
    entry.edit_flags.skills = source.edit_flags.skills;
    entry.edit_flags.playing_style = source.edit_flags.playing_style;
    entry.edit_flags.com_styles = source.edit_flags.com_styles;
    entry
}

/// Gives 70101 and 70102 distinct values in and out of the field set, so a
/// copy's kept half is not vacuous (`version`'s fixture written to `path`).
fn mark_copy_players(path: &std::path::Path, version: PesVersion) {
    let mut file = EditFile::from_bytes(&fixture_save(version)).unwrap();
    {
        let dest = file.player_mut(70101).unwrap();
        dest.name = "DEST".to_owned();
        dest.shirt_name = "DSHIRT".to_owned();
        dest.basic.nationality = 7;
        dest.basic.age = 25;
        dest.basic.height = 170;
        dest.basic.weight = 61;
        dest.motion.penalty_kick = 2;
        dest.motion.goal_celebration_1 = 30;
        // The kept flags all held on the destination, clear on the source:
        // a wrong copy shows.
        dest.edit_flags.motion = true;
        dest.edit_flags.face = true;
        dest.edit_flags.hair = true;
        dest.edit_flags.physique = true;
        dest.edit_flags.strip = true;
        dest.edit_flags.base_copy = dest.edit_flags.base_copy.map(|_| true);
        dest.appearance.boots_id = 33;
        dest.appearance.inners = 1;
    }
    {
        let source = file.player_mut(70102).unwrap();
        source.name = "SOURCE".to_owned();
        source.shirt_name = "SSHIRT".to_owned();
        source.basic.nationality = 99;
        source.basic.age = 40;
        source.basic.height = 191;
        source.basic.weight = 90;
        source.stats.speed = 88;
        source.stats.finishing = 77;
        source.stats.form = 6;
        source.positions.registered = 3;
        source.positions.playing_style = 5;
        source.positions.stronger_foot = 1;
        source.positions.playable[0] = 2;
        source.skills.skills[3] = true;
        source.skills.com_styles[2] = true;
        source.motion.free_kick = 9;
        source.motion.penalty_kick = 3;
        source.motion.goal_celebration_1 = 50;
        let flags = &mut source.edit_flags;
        flags.player = true;
        flags.basic_settings = true;
        flags.registered_position = true;
        flags.playable_positions = true;
        flags.abilities = true;
        flags.skills = true;
        flags.playing_style = true;
        flags.com_styles = true;
        // The kept flags explicitly clear, opposite of the destination's.
        flags.motion = false;
        flags.face = false;
        flags.hair = false;
        flags.physique = false;
        flags.strip = false;
        flags.base_copy = flags.base_copy.map(|_| false);
        source.appearance.boots_id = 44;
        source.appearance.inners = 2;
    }
    file.save(path).unwrap();
}

/// `slot`'s label in the Copy/Swap dialog's player combo: the shirt
/// number, registered position and display name, as the player list shows
/// them.
fn slot_label(entry: &PlayerEntry, number: u16) -> String {
    let position = POSITIONS
        .get(usize::from(entry.positions.registered))
        .copied()
        .unwrap_or("?");
    format!("{number} {position} {}", display_name(&entry.name))
}

/// Picks `label` in the Copy/Swap dialog's player combo and clicks OK.
fn run_copy_swap(window: &mut Headless, view: &mut SaveView, item: &str, label: &str) {
    let nodes = open_menu(window, view, "Player", "Set all stats");
    click(window, menu_item(&nodes, item), view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    let combo = right_of_label(&nodes, "Player", Role::ComboBox)
        .first()
        .copied()
        .unwrap_or_else(|| panic!("the {item:?} dialog's player combo"));
    click(window, combo, view);
    let (_, nodes, _) = window.frame_ids(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    let (option, _) = nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("no {label:?} in the player combo"));
    action_click(window, view, *option);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    click(window, labeled(&nodes, Role::Button, "OK"), view);
}

// The Copy row: the picked player's field set lands on the
// selected one and nothing outside it moves — names, nationality, age,
// every motion but the free kick's, the motion and appearance flags and
// the whole appearance stay — as one undoable edit, saved and reopened.
// SE-GUI-24
#[test]
fn copy_stats_writes_the_field_set_and_nothing_else() {
    let path = fixture(PesVersion::Pes21, "player_copy");
    mark_copy_players(&path, PesVersion::Pes21);
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let dest_before = player_entry(&view, 70101);
    let source_before = player_entry(&view, 70102);
    // Premises: the fields a copy must and must not carry all differ.
    assert_ne!(dest_before.stats, source_before.stats);
    assert_ne!(dest_before.positions, source_before.positions);
    assert_ne!(dest_before.name, source_before.name);
    assert_ne!(dest_before.motion, source_before.motion);
    assert_ne!(dest_before.appearance, source_before.appearance);

    // The dialog's team combo opens on the selected player's team (701),
    // its player combo on the first filled slot: 70102 is the option named
    // by his shirt number, position and name.
    let number = team_entry(&view, 701).roster[1].number;
    run_copy_swap(
        &mut window,
        &mut view,
        "Copy stats from",
        &slot_label(&source_before, number),
    );

    assert_eq!(
        player_entry(&view, 70101),
        with_stats_of(&dest_before, &source_before),
        "the field set copied, the rest 70101's own"
    );
    assert_eq!(
        player_entry(&view, 70102),
        source_before,
        "the source is untouched"
    );
    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");
    assert_eq!(player_entry(&view, 70101), dest_before, "restored");

    // Redo, save, reopen: the file holds the copy.
    window.frame(vec![key(egui::Key::Y, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        EditFile::load(&path).unwrap().player(70101).unwrap(),
        &with_stats_of(&dest_before, &source_before),
        "the reopened file holds the copy"
    );
}

// The Swap row: the same field set traded both ways between the
// selected and the picked player — here on PES 16, whose appearance block
// is a separate record and stays out of the trade as the plan's row says.
// SE-GUI-24
#[test]
fn swap_stats_trades_the_field_set_on_pes16() {
    let path = fixture(PesVersion::Pes16, "player_swap16");
    mark_copy_players(&path, PesVersion::Pes16);
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let first_before = player_entry(&view, 70101);
    let second_before = player_entry(&view, 70102);
    assert_ne!(first_before.appearance, second_before.appearance);

    let number = team_entry(&view, 701).roster[1].number;
    run_copy_swap(
        &mut window,
        &mut view,
        "Swap stats with",
        &slot_label(&second_before, number),
    );

    assert_eq!(
        player_entry(&view, 70101),
        with_stats_of(&first_before, &second_before),
        "70101 takes 70102's set, keeps the rest"
    );
    assert_eq!(
        player_entry(&view, 70102),
        with_stats_of(&second_before, &first_before),
        "70102 takes 70101's set, keeps the rest"
    );
    assert_eq!(
        player_entry(&view, 70101).appearance,
        first_before.appearance,
        "the appearance record stays"
    );
    assert_eq!(
        player_entry(&view, 70102).appearance,
        second_before.appearance,
        "the appearance record stays"
    );

    // One undo trades them back; redo trades again; saved file holds it.
    assert_eq!(undo_entries(&mut window, &mut view), 1, "one undo step");
    assert_eq!(player_entry(&view, 70101), first_before);
    assert_eq!(player_entry(&view, 70102), second_before);
    window.frame(vec![key(egui::Key::Y, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let reopened = EditFile::load(&path).unwrap();
    assert_eq!(
        reopened.player(70101).unwrap(),
        &with_stats_of(&first_before, &second_before)
    );
    assert_eq!(
        reopened.player(70102).unwrap(),
        &with_stats_of(&second_before, &first_before)
    );
}

// The four menu dialogs' headings are their titles (`StatNum::title`,
// `CopySwap::title`): the heading text is what a member reads the dialog
// by.
#[test]
fn the_menu_dialogs_draw_their_headings() {
    let path = fixture(PesVersion::Pes21, "player_dialog_headings");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    for (item, title) in [
        ("Set all stats", "Set all stats"),
        ("Bump all stats", "Bump all stats"),
        ("Copy stats from", "Copy stats"),
        ("Swap stats with", "Swap stats"),
    ] {
        let nodes = open_menu(&mut window, &mut view, "Player", "Set all stats");
        click(&mut window, menu_item(&nodes, item), &mut view);
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        assert!(
            label_drawn(&nodes, title),
            "the {item:?} dialog draws its heading {title:?}"
        );
        click(
            &mut window,
            labeled(&nodes, Role::Button, "Cancel"),
            &mut view,
        );
    }
}

// The Copy/Swap dialog's pick: it opens on the selected player's team's
// first filled roster slot, and picking another team re-picks that
// team's first filled slot — the combo's shown text names the picked
// player by his number, position and name.
#[test]
fn the_copy_swap_dialog_picks_the_teams_first_filled_slot() {
    let path = fixture(PesVersion::Pes21, "player_copy_swap_pick");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    // The selected player is the first listed, so his team's first
    // filled slot is the pick. Another team of the fixture, for the
    // switch: read both from the save, never typed in.
    let (slot, first_id) = rostered(&view, 701)
        .first()
        .copied()
        .expect("team 701 has a filled slot");
    let first = player_entry(&view, first_id);
    let number = team_entry(&view, 701).roster[slot].number;
    let second_id = view
        .session
        .as_ref()
        .unwrap()
        .file()
        .teams()
        .iter()
        .find(|team| team.id != 701 && team.roster.iter().any(|slot| slot.player_id != 0))
        .expect("another team with a filled slot")
        .id;
    let second_name = team_entry(&view, second_id).name.clone();
    let (second_slot, second_player_id) = rostered(&view, second_id)
        .first()
        .copied()
        .expect("the other team has a filled slot");
    let second = player_entry(&view, second_player_id);
    let second_number = team_entry(&view, second_id).roster[second_slot].number;

    let nodes = open_menu(&mut window, &mut view, "Player", "Set all stats");
    click(&mut window, menu_item(&nodes, "Copy stats from"), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        player_combo(&nodes).value().map(str::to_owned),
        Some(slot_label(&first, number)),
        "the pick is the team's first filled slot, shown by his label"
    );

    // Pick the other team in the Team combo: the pick re-seats on its
    // first filled slot.
    let team_combo = right_of_label(&nodes, "Team", Role::ComboBox)
        .first()
        .copied()
        .expect("the team combo");
    click(&mut window, team_combo, &mut view);
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (option, _) = nodes
        .iter()
        .find(|(_, node)| node.label() == Some(second_name.as_str()))
        .unwrap_or_else(|| panic!("no {second_name:?} in the team combo"));
    action_click(&mut window, &mut view, *option);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        player_combo(&nodes).value().map(str::to_owned),
        Some(slot_label(&second, second_number)),
        "the pick is the picked team's first filled slot"
    );
}

/// The Copy/Swap dialog's player combo node.
fn player_combo(nodes: &[accesskit::Node]) -> &accesskit::Node {
    right_of_label(nodes, "Player", Role::ComboBox)
        .first()
        .copied()
        .expect("the player combo")
}

// The Copy/Swap dialog leaves on OK as on Cancel (`view.copy_swap` is
// `None` after both), the `&&` mutant's keep-it-open case.
#[test]
fn the_copy_swap_dialog_closes_on_ok_and_cancel() {
    let path = fixture(PesVersion::Pes21, "player_copy_swap_close");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);

    for button in ["OK", "Cancel"] {
        let nodes = open_menu(&mut window, &mut view, "Player", "Set all stats");
        click(&mut window, menu_item(&nodes, "Copy stats from"), &mut view);
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        click(
            &mut window,
            labeled(&nodes, Role::Button, button),
            &mut view,
        );
        assert!(view.copy_swap.is_none(), "{button} closed the dialog");
    }
}

/// Whether the nodes draw a Label node with `value` as its text.
fn label_drawn(nodes: &[accesskit::Node], value: &str) -> bool {
    nodes
        .iter()
        .any(|node| node.role() == Role::Label && node.value() == Some(value))
}

/// The Button labeled `label` on the Appearance tab's FPC group.
fn fpc_button<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    labeled(nodes, Role::Button, label)
}

/// The Appearance tab unlocked and drawn for one frame's nodes.
fn appearance_frame(window: &mut Headless, view: &mut SaveView) -> Vec<accesskit::Node> {
    view.tab = Tab::Appearance;
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    nodes
}

/// 70101's strip fields the presets write, as the session holds him.
fn strip_of(view: &SaveView, id: u32) -> (u8, bool, u8, u32, u32) {
    let player = player_entry(view, id);
    (
        player.appearance.sleeves,
        player.appearance.untucked,
        player.appearance.socks,
        player.appearance.boots_id,
        player.appearance.gloves_id,
    )
}

// Hide body gives the hide preset and the tab reads "FPC:
// hidden body"; Ctrl+I gives the show preset and "FPC: body shown". A
// model's own boots ID is kept and reads as shown. Set FPC invisibility
// hides every filled slot of the team in one undo step, and a strip
// setting that breaks hiding lists its finding on the tab — reported,
// never cleared.
// SE-GUI-11
#[test]
fn hide_and_show_body_apply_the_presets_keeping_own_ids() {
    let path = fixture(PesVersion::Pes21, "fpc_hide_show");
    // 70101 dressed with inners on (the finding's case), 70102 with his
    // model's own boots ID.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    {
        let player = file.player_mut(70101).unwrap();
        player.appearance.boots_id = 0;
        player.appearance.gloves_id = 0;
        player.appearance.sleeves = 1;
        player.appearance.untucked = true;
        player.appearance.socks = 0;
        player.appearance.inners = 1;
    }
    {
        let player = file.player_mut(70102).unwrap();
        player.appearance.boots_id = 627;
        player.appearance.gloves_id = 0;
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    assert_eq!(
        strip_of(&view, 70101),
        (1, true, 0, 0, 0),
        "premise: dressed, boots and gloves 0"
    );

    // Hide body: long sleeves, tucked, short socks, the nonexistent IDs,
    // the inners left on as the finding reports rather than clears.
    let nodes = appearance_frame(&mut window, &mut view);
    assert!(
        label_drawn(&nodes, "FPC: body shown"),
        "dressed reads shown"
    );
    click(&mut window, fpc_button(&nodes, "Hide body"), &mut view);
    assert_eq!(
        strip_of(&view, 70101),
        (2, false, 2, 55, 11),
        "the hide preset landed"
    );
    assert_eq!(player_entry(&view, 70101).appearance.inners, 1, "kept");
    let nodes = appearance_frame(&mut window, &mut view);
    assert!(
        label_drawn(&nodes, "FPC: hidden body"),
        "hidden reads hidden"
    );
    assert!(
        label_drawn(
            &nodes,
            "Error: The player wears inners under a hidden body, which breaks the hiding."
        ),
        "the inners finding's text on the tab"
    );

    // Ctrl+I on a hidden player shows him: the show preset, "FPC: body shown".
    window.frame(vec![key(egui::Key::I, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        strip_of(&view, 70101),
        (1, true, 0, 0, 0),
        "the show preset"
    );
    let nodes = appearance_frame(&mut window, &mut view);
    assert!(label_drawn(&nodes, "FPC: body shown"));

    // Two operations, two undo entries, the player restored.
    assert_eq!(undo_entries(&mut window, &mut view), 2, "one entry each");

    // 70102's own boots ID 627: Hide keeps it, and he reads "FPC: body
    // shown" — his model hides him, the save cannot tell.
    view.selected = Some((701, 1));
    let nodes = appearance_frame(&mut window, &mut view);
    click(&mut window, fpc_button(&nodes, "Hide body"), &mut view);
    assert_eq!(
        strip_of(&view, 70102),
        (2, false, 2, 627, 11),
        "the strip hidden, the own boots ID kept"
    );
    let nodes = appearance_frame(&mut window, &mut view);
    assert!(
        label_drawn(&nodes, "FPC: body shown"),
        "a model's own boots ID reads as shown"
    );
    click(&mut window, fpc_button(&nodes, "Show body"), &mut view);
    assert_eq!(
        strip_of(&view, 70102),
        (1, true, 0, 627, 0),
        "Show keeps the own ID too"
    );
    assert_eq!(undo_entries(&mut window, &mut view), 2, "one entry each");

    // The Team menu's Set FPC invisibility: the hide preset on every
    // filled slot of team 701, one undo step.
    let before: Vec<(usize, u32, u32, u32)> = rostered(&view, 701)
        .iter()
        .map(|&(slot, id)| {
            let player = player_entry(&view, id);
            (
                slot,
                id,
                player.appearance.boots_id,
                player.appearance.gloves_id,
            )
        })
        .collect();
    let nodes = open_menu(&mut window, &mut view, "Team", "Set edited flag");
    click(
        &mut window,
        menu_item(&nodes, "Set FPC invisibility"),
        &mut view,
    );
    for &(slot, id, boots, gloves) in &before {
        let strip = strip_of(&view, id);
        assert_eq!(strip.0, 2, "slot {}'s sleeves", slot + 1);
        assert!(!strip.1, "slot {}'s tuck", slot + 1);
        assert_eq!(strip.2, 2, "slot {}'s socks", slot + 1);
        // A record's own ID stays, 0 or the hide ID takes the preset's.
        let expected_boots = if boots == 0 || boots == 55 { 55 } else { boots };
        assert_eq!(strip.3, expected_boots, "slot {}'s boots", slot + 1);
        let expected_gloves = if gloves == 0 || gloves == 11 {
            11
        } else {
            gloves
        };
        assert_eq!(strip.4, expected_gloves, "slot {}'s gloves", slot + 1);
    }
    assert_eq!(
        player_entry(&view, 70102).appearance.boots_id,
        627,
        "his own ID kept in the team run"
    );

    // Remove FPC invisibility is the same op the other way, run on the
    // hidden team: the show preset on every filled slot, the own IDs
    // still kept; one undo restores the hidden state.
    let nodes = open_menu(&mut window, &mut view, "Team", "Set edited flag");
    click(
        &mut window,
        menu_item(&nodes, "Remove FPC invisibility"),
        &mut view,
    );
    for &(slot, id, boots, gloves) in &before {
        let strip = strip_of(&view, id);
        assert_eq!(strip.0, 1, "slot {}'s sleeves", slot + 1);
        assert!(strip.1, "slot {}'s tuck", slot + 1);
        assert_eq!(strip.2, 0, "slot {}'s socks", slot + 1);
        assert_eq!(strip.3, boots, "slot {}'s boots", slot + 1);
        assert_eq!(strip.4, gloves, "slot {}'s gloves", slot + 1);
    }
    // One undo (the Remove entry alone) hides them all again; the next is
    // the Set entry and restores the team's own state.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    for &(slot, id, boots, gloves) in &before {
        let strip = strip_of(&view, id);
        assert_eq!(strip.0, 2, "slot {}'s sleeves hidden again", slot + 1);
        assert_eq!(strip.2, 2, "slot {}'s socks hidden again", slot + 1);
        let expected_boots = if boots == 0 { 55 } else { boots };
        assert_eq!(strip.3, expected_boots, "slot {}'s boots", slot + 1);
        let expected_gloves = if gloves == 0 { 11 } else { gloves };
        assert_eq!(strip.4, expected_gloves, "slot {}'s gloves", slot + 1);
    }
    assert_eq!(undo_entries(&mut window, &mut view), 1, "the Set entry");
    for &(_, id, boots, gloves) in &before {
        let strip = strip_of(&view, id);
        assert_eq!(strip.3, boots, "restored");
        assert_eq!(strip.4, gloves, "restored");
    }
}

// Ctrl+I toggles both ways (the SE-GUI-11 test covers it hidden to
// shown): on a shown player it hides, again it shows, and the Player
// menu's Toggle FPC settings, enabled under the unlock, hides again.
// SE-GUI-11
#[test]
fn ctrl_i_toggles_the_fpc_visibility_both_ways_and_the_menu_item_too() {
    let path = fixture(PesVersion::Pes21, "fpc_ctrl_i");
    // 70101 dressed "body shown": boots and gloves 0.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    {
        let player = file.player_mut(70101).unwrap();
        player.appearance.boots_id = 0;
        player.appearance.gloves_id = 0;
        player.appearance.sleeves = 1;
        player.appearance.untucked = true;
        player.appearance.socks = 0;
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);
    let nodes = appearance_frame(&mut window, &mut view);
    assert!(label_drawn(&nodes, "FPC: body shown"), "premise: shown");

    // Ctrl+I on a shown player hides him: the hide preset lands.
    window.frame(vec![key(egui::Key::I, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let nodes = appearance_frame(&mut window, &mut view);
    assert!(label_drawn(&nodes, "FPC: hidden body"), "Ctrl+I hid him");
    assert_eq!(strip_of(&view, 70101), (2, false, 2, 55, 11));

    // Ctrl+I again shows him: the show preset lands.
    window.frame(vec![key(egui::Key::I, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let nodes = appearance_frame(&mut window, &mut view);
    assert!(label_drawn(&nodes, "FPC: body shown"), "Ctrl+I showed him");
    assert_eq!(strip_of(&view, 70101), (1, true, 0, 0, 0));

    // The Player menu's Toggle FPC settings, enabled under the unlock,
    // hides him again.
    let nodes = open_menu(&mut window, &mut view, "Player", "Set all stats");
    let item = menu_item(&nodes, "Toggle FPC settings");
    assert!(
        !item.is_disabled(),
        "the toggle is enabled under the unlock"
    );
    click(&mut window, item, &mut view);
    let nodes = appearance_frame(&mut window, &mut view);
    assert!(
        label_drawn(&nodes, "FPC: hidden body"),
        "the menu toggle hid him"
    );
    assert_eq!(strip_of(&view, 70101), (2, false, 2, 55, 11));
}

// The PES 15/16 half of the same scenario: the appearance block is a
// separate record there; Hide body round-trips through Save and reopen.
// SE-GUI-11
#[test]
fn hide_body_round_trips_on_pes16() {
    let path = fixture(PesVersion::Pes16, "fpc_hide_16");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes16)).unwrap();
    {
        let player = file.player_mut(70101).unwrap();
        player.appearance.boots_id = 0;
        player.appearance.gloves_id = 0;
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.edit_aesthetics = true;
    let mut window = Headless::sized(1100.0, 700.0);

    let nodes = appearance_frame(&mut window, &mut view);
    click(&mut window, fpc_button(&nodes, "Hide body"), &mut view);
    assert_eq!(strip_of(&view, 70101), (2, false, 2, 55, 11));
    let nodes = appearance_frame(&mut window, &mut view);
    assert!(label_drawn(&nodes, "FPC: hidden body"));

    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let player = EditFile::load(&path)
        .unwrap()
        .player(70101)
        .unwrap()
        .clone();
    assert_eq!(
        (
            player.appearance.sleeves,
            player.appearance.untucked,
            player.appearance.socks,
            player.appearance.boots_id,
            player.appearance.gloves_id
        ),
        (2, false, 2, 55, 11),
        "the hide preset saved and reopened"
    );
}

// The FPC items' half of the lock: under it the tab's Hide/Show body,
// the Player menu's Toggle FPC settings and the Team menu's FPC items are
// disabled with the ownership line as hover; Ctrl+I is a no-op while Set
// all stats and Copy stats stay enabled.
// SE-GUI-22
#[test]
fn the_lock_disables_the_fpc_items_with_the_ownership_line() {
    let path = fixture(PesVersion::Pes21, "fpc_locked");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().appearance.boots_id = 0;
    file.player_mut(70101).unwrap().appearance.gloves_id = 0;
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let before = player_entry(&view, 70101);

    // The tab's two buttons disabled, each hover the ownership line.
    let nodes = appearance_frame(&mut window, &mut view);
    for label in ["Hide body", "Show body"] {
        let button = fpc_button(&nodes, label);
        assert!(button.is_disabled(), "{label} is disabled under the lock");
        assert_eq!(
            tooltip_containing(&mut window, &mut view, centre(button), "Owned by"),
            Some("Owned by the team's export: edit its settings.toml and recompile.".to_string()),
            "{label}'s hover is the ownership line"
        );
    }

    // The Player menu's Toggle FPC settings disabled the same way, the
    // gameplay items enabled.
    let nodes = open_menu(&mut window, &mut view, "Player", "Set all stats");
    let toggle = menu_item(&nodes, "Toggle FPC settings");
    assert!(
        toggle.is_disabled(),
        "the toggle is disabled under the lock"
    );
    assert_eq!(
        tooltip_containing(&mut window, &mut view, centre(toggle), "Owned by"),
        Some("Owned by the team's export: edit its settings.toml and recompile.".to_string()),
        "the toggle's hover is the ownership line"
    );
    for label in [
        "Set all stats",
        "Copy stats from",
        "Swap stats with",
        "Bump all stats",
    ] {
        assert!(
            !menu_item(&nodes, label).is_disabled(),
            "{label} stays enabled"
        );
    }

    // The Team menu's two FPC items disabled with the same hover.
    let nodes = open_menu(&mut window, &mut view, "Team", "Set edited flag");
    for label in ["Set FPC invisibility", "Remove FPC invisibility"] {
        let item = menu_item(&nodes, label);
        assert!(item.is_disabled(), "{label} is disabled under the lock");
        assert_eq!(
            tooltip_containing(&mut window, &mut view, centre(item), "Owned by"),
            Some("Owned by the team's export: edit its settings.toml and recompile.".to_string()),
            "{label}'s hover is the ownership line"
        );
    }

    // Ctrl+I under the lock changes nothing.
    window.frame(vec![key(egui::Key::I, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(player_entry(&view, 70101), before, "Ctrl+I changed nothing");
    assert!(!view.session.as_ref().unwrap().is_dirty());
}
