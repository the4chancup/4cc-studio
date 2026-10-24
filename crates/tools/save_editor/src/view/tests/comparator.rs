//! The comparator panel's tests (SE-GUI-13; `plans/save_editor.md`
//! "Comparator (gameplay + aesthetics)"): the pick and its refusals, the
//! rows in the tabs' terms, the scope toggles, the only-in heads, the
//! click's selection and the `Comparing…` mark.

use std::path::PathBuf;

use pes_savefile::model::player::STRIPPED_APPEARANCE_ID;

use super::*;

/// The salt the picked saves the tests write encrypt with (the CLI tests'
/// own).
const TEST_SALT: [u8; 320] = [7; 320];

/// The PES 17 fixture with `edit` applied, written under `dir` as `name`:
/// the picked save the dialog stub returns.
fn picked_save(dir: &Path, name: &str, edit: impl FnOnce(&mut EditFile)) -> PathBuf {
    let mut file =
        EditFile::from_bytes(&fixture_save(PesVersion::Pes17)).expect("the fixture loads");
    edit(&mut file);
    let path = dir.join(name);
    std::fs::write(&path, file.to_bytes(&TEST_SALT).expect("the save writes")).unwrap();
    path
}

/// The comparator opened from the toolbar's Panels menu: the
/// "Comparator…" item clicked, the next frame's node pairs returned.
fn open_comparator(
    window: &mut Headless,
    view: &mut SaveView,
) -> Vec<(accesskit::NodeId, accesskit::Node)> {
    let nodes = draw(window, view, None);
    click(window, node_labeled(&nodes, Role::Button, "Panels"), view);
    let nodes = draw(window, view, None);
    click(
        window,
        node_labeled(&nodes, Role::Button, "Comparator\u{2026}"),
        view,
    );
    let (_, pairs, _) = window.frame_ids(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    pairs
}

/// `path` stubbed as the Pick dialog's choice; the "Pick…" button clicked.
fn pick(window: &mut Headless, view: &mut SaveView, path: PathBuf) {
    view.open_save_dialog = Box::new(move || Some(path.clone()));
    let nodes = draw(window, view, None);
    click(
        window,
        node_labeled(&nodes, Role::Button, "Pick\u{2026}"),
        view,
    );
}

/// Frames until the comparison lands: `pending` empty and the result keyed
/// to the session's current revision, the picked session's and the scope
/// (bounded, so a worker that never replies fails rather than hangs).
/// Returns the landing frame's node pairs.
fn compared(
    window: &mut Headless,
    view: &mut SaveView,
) -> Vec<(accesskit::NodeId, accesskit::Node)> {
    frames_until(
        window,
        view,
        None,
        |view| {
            let session = view.session.as_ref().expect("an open save");
            let picked = view
                .comparator
                .picked
                .as_ref()
                .map(|(_, picked)| picked.revision());
            view.comparator.comparison.pending.is_none()
                && view
                    .comparator
                    .comparison
                    .result
                    .as_ref()
                    .is_some_and(|(key, _)| {
                        key.revision == session.revision()
                            && Some(key.picked) == picked
                            && key.scope == view.comparator.scope
                    })
        },
        "the comparison never returned",
    )
}

/// The comparator window's nodes out of `pairs` (`frame_ids`' output).
fn comparator_nodes(pairs: &[(accesskit::NodeId, accesskit::Node)]) -> Vec<&accesskit::Node> {
    nodes_in_window(pairs, "Comparator")
        .into_iter()
        .map(|(_, node)| node)
        .collect()
}

/// The `(NodeId, node)` pair of the Button labeled `label` inside the
/// comparator window (the player list's buttons share the screen); each
/// caller takes the half it needs.
fn comparator_button<'a>(
    pairs: &'a [(accesskit::NodeId, accesskit::Node)],
    label: &str,
) -> (accesskit::NodeId, &'a accesskit::Node) {
    nodes_in_window(pairs, "Comparator")
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .unwrap_or_else(|| panic!("no window button {label:?}"))
}

/// A PES 21 fixture written under `dir`, for a pick of another version.
fn picked_pes21(dir: &Path) -> PathBuf {
    let path = dir.join("picked21");
    std::fs::write(&path, fixture_save(PesVersion::Pes21)).unwrap();
    path
}

/// Team 734's fixture save open with 73401's height edited to 183, the
/// unedited fixture picked, the comparator open and its result landed:
/// the pair the row tests build on.
fn compared_73401(
    test: &str,
) -> (
    SaveView,
    Headless,
    Vec<(accesskit::NodeId, accesskit::Node)>,
) {
    let (mut view, mut window) = open_734_with(test, &[73401], 183);
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    let picked = picked_save(&dir, "picked", |_| {});
    view.comparator.open = true;
    pick(&mut window, &mut view, picked);
    let pairs = compared(&mut window, &mut view);
    (view, window, pairs)
}

/// Wheels over the comparator's list until a node holding `text` (a
/// Label's value or a Button's label) is drawn — a frame draws only the
/// rows in view, so a row deep in the list is scrolled to — and returns
/// the pairs of a frame where it held still (the wheel's offset target
/// keeps animating past the found frame). Bounded: a row that never
/// shows fails rather than hangs.
fn scroll_to(
    window: &mut Headless,
    view: &mut SaveView,
    text: &str,
) -> Vec<(accesskit::NodeId, accesskit::Node)> {
    // The window's centre, where the list is.
    let at = egui::pos2(400.0, 400.0);
    let mut bounds = None;
    for _ in 0..600 {
        let (_, pairs, _) = window.frame_ids(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
        let nodes = comparator_nodes(&pairs);
        let node = nodes
            .iter()
            .find(|node| node.value() == Some(text) || node.label() == Some(text));
        if let Some(node) = node {
            let next = node.bounds();
            if next.is_some() && next == bounds {
                return pairs;
            }
            bounds = next;
            continue;
        }
        bounds = None;
        // Not in view: one notch down; the next settle frame reads where
        // its animation lands.
        window.frame_ids(
            vec![
                egui::Event::PointerMoved(at),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -300.0),
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Wheel,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            |ui| show(ui, view, &|| Vec::new(), None),
        );
    }
    panic!("the list never showed {text:?}")
}

// SE-GUI-13
#[test]
fn the_panel_lists_the_differences_per_team_and_a_rows_click_selects() {
    let (mut view, mut window) = open_734_with("comparator_rows", &[73401], 183);
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    let picked = picked_save(&dir, "picked", |_| {});
    let pairs = open_comparator(&mut window, &mut view);
    assert!(
        pairs
            .iter()
            .any(|(_, node)| node.label() == Some("Comparator")),
        "the window's title"
    );
    pick(&mut window, &mut view, picked);
    compared(&mut window, &mut view);
    let pairs = scroll_to(&mut window, &mut view, "Player 73401  49 Muh Spooks");
    let nodes = comparator_nodes(&pairs);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Team 734 /lit/")),
        "734's heading"
    );
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::Button && node.label() == Some("Player 73401  49 Muh Spooks")
        }),
        "73401's head"
    );
    let (_, row) = comparator_button(&pairs, "Height 185 \u{2192} 183");

    // The selector on another team: the click selects the player and
    // moves the selector to his, as the AATF panel's click does.
    view.team = TeamChoice::Team(718);
    view.selected = Some((718, 0));
    click(&mut window, row, &mut view);
    assert_eq!(view.selected, Some((734, 0)), "73401's team and slot");
    assert_eq!(view.team, TeamChoice::Team(734), "the selector moved");
}

// SE-GUI-13
#[test]
fn a_motion_row_reads_as_the_tab_shows_it() {
    let (mut view, mut window) = open_734_with("comparator_motion", &[], 183);
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    // The open save's 73401 free-kick motion stored 14 (the picked's
    // stored 0): the tab shows one above the stored.
    let mut entry = player_entry(&view, 73401);
    assert_eq!(entry.motion.free_kick, 0, "premise: stored 0");
    entry.motion.free_kick = 14;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let picked = picked_save(&dir, "picked", |_| {});
    view.comparator.open = true;
    pick(&mut window, &mut view, picked);
    compared(&mut window, &mut view);
    let pairs = scroll_to(&mut window, &mut view, "Free kick 1 \u{2192} 15");
    comparator_button(&pairs, "Free kick 1 \u{2192} 15");
}

// SE-GUI-13
#[test]
fn the_scope_toggles_narrow_the_rows() {
    let (mut view, mut window) = open_734_with("comparator_scopes", &[73401], 183);
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    // A gameplay row (height, through `open_734_with`) and an aesthetics
    // one (73401's boots id, 0 in the fixture).
    let mut entry = player_entry(&view, 73401);
    assert_eq!(entry.appearance.boots_id, 0, "premise: no boots");
    entry.appearance.boots_id = 3601;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let picked = picked_save(&dir, "picked", |_| {});
    view.comparator.open = true;
    pick(&mut window, &mut view, picked);
    compared(&mut window, &mut view);
    let pairs = scroll_to(&mut window, &mut view, "Boots ID 0 \u{2192} 3601");
    comparator_button(&pairs, "Boots ID 0 \u{2192} 3601");
    comparator_button(&pairs, "Height 185 \u{2192} 183");

    let has = |pairs: &[(accesskit::NodeId, accesskit::Node)], label: &str| {
        comparator_nodes(pairs)
            .iter()
            .any(|node| node.role() == Role::Button && node.label() == Some(label))
    };
    // Gameplay: the height alone; Aesthetics: the boots alone. `compared`
    // waits for the new scope's result before the scroll, or the held
    // previous result's rows would read as the new scope's.
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Gameplay"),
        &mut view,
    );
    compared(&mut window, &mut view);
    let pairs = scroll_to(&mut window, &mut view, "Height 185 \u{2192} 183");
    assert!(has(&pairs, "Height 185 \u{2192} 183"), "the gameplay row");
    assert!(
        !has(&pairs, "Boots ID 0 \u{2192} 3601"),
        "the aesthetics row narrowed out"
    );
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Aesthetics"),
        &mut view,
    );
    compared(&mut window, &mut view);
    let pairs = scroll_to(&mut window, &mut view, "Boots ID 0 \u{2192} 3601");
    assert!(
        has(&pairs, "Boots ID 0 \u{2192} 3601"),
        "the aesthetics row"
    );
    assert!(
        !has(&pairs, "Height 185 \u{2192} 183"),
        "the gameplay row narrowed out"
    );
}

/// An edit after the first result marks the held list `Comparing…` and
/// starts no run until the gesture ends — the mark read on a held-pointer
/// frame, where no reply can land (`an_edit_rechecks_and_the_old_result_marks_checking`'s
/// premise).
#[test]
fn an_edit_marks_the_held_list_comparing() {
    let (mut view, mut window, _) = compared_73401("comparator_mark");
    scroll_to(&mut window, &mut view, "Height 185 \u{2192} 183");

    // The pointer held on the toolbar's empty end so the re-run waits for
    // the release: a started run could reply before the mark is read.
    let at = egui::pos2(1090.0, 12.0);
    window.frame(pointer_at(at, true), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    // 73401 back to his fixture height: the saves are identical again.
    let mut entry = player_entry(&view, 73401);
    entry.basic.height = 185;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        view.comparator.comparison.pending.is_none(),
        "premise: no run started while the pointer is held"
    );
    let nodes = comparator_nodes(&pairs);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Comparing\u{2026}")),
        "the edit marks the held result"
    );
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::Button && node.label() == Some("Height 185 \u{2192} 183")
        }),
        "the stale row stays while comparing"
    );
    window.frame(pointer_at(at, false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(
        view.comparator.comparison.pending.is_some(),
        "the run starts on the release frame"
    );
    let pairs = compared(&mut window, &mut view);
    let nodes = comparator_nodes(&pairs);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("No differences.")),
        "identical saves read empty"
    );
    assert!(
        !nodes.iter().any(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Height"))
        }),
        "the old row is gone"
    );
}

// SE-GUI-13
#[test]
fn a_save_of_another_version_is_refused_naming_both() {
    let (mut view, mut window) = open_734_with("comparator_version", &[], 183);
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    let picked = picked_pes21(&dir);
    view.comparator.open = true;
    pick(&mut window, &mut view, picked);
    let lines = window_lines(&mut window, &mut view, None, "Comparator");
    assert!(
        lines.iter().any(|line| line
            == "The picked save is a PES 2021 save and the open save a PES 2017 one: both must be of one version"),
        "the refusal's literal text: {lines:?}"
    );
    assert!(
        lines.iter().any(|line| line == "(none)"),
        "the pick dropped: {lines:?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("Player ") || line.starts_with("Team ")),
        "no rows: {lines:?}"
    );
    assert!(
        view.comparator.comparison.result.is_none() && view.comparator.comparison.pending.is_none(),
        "no comparison ran"
    );
}

// SE-GUI-13
#[test]
fn the_players_only_in_one_save_list_under_their_headings() {
    let (mut view, mut window) = open_734_with("comparator_only_in", &[], 183);
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    // As `tests/cli/diff.rs`'s `diff_pair`: one id renumbered out, so
    // 9_999_999 is in the picked save alone and 73410 in the open alone.
    let picked = picked_save(&dir, "picked", |file| {
        file.player_mut(73410).expect("73410 is a player").id = 9_999_999;
    });
    let slot = view
        .session
        .as_ref()
        .unwrap()
        .file()
        .team(734)
        .expect("team 734")
        .roster
        .iter()
        .position(|slot| slot.player_id == 73410)
        .map(|slot| u8::try_from(slot).expect("a roster slot"))
        .expect("premise: 73410 rostered");
    view.comparator.open = true;
    pick(&mut window, &mut view, picked);
    compared(&mut window, &mut view);
    let pairs = scroll_to(&mut window, &mut view, "Only in the open save");
    let nodes = comparator_nodes(&pairs);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Only in the picked save")),
        "the picked-only heading"
    );
    // His record keeps the name; no roster holds him in the picked save.
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Player 9999999 DFW")),
        "the picked-only head"
    );
    assert!(
        !nodes.iter().any(|node| {
            node.role() == Role::Button
                && node.label().is_some_and(|label| label.contains("9999999"))
        }),
        "the picked-only head is no click target"
    );
    // The open-only head clicks as a row does (an AccessKit Click: the
    // scroll's animation keeps its bounds moving past the found frame, so
    // a pointer position can already be stale).
    view.selected = Some((734, 0));
    action_click(
        &mut window,
        &mut view,
        comparator_button(&pairs, "Player 73410  46 DFW").0,
    );
    assert_eq!(view.selected, Some((734, slot)), "73410 selected");
}

// SE-GUI-13
#[test]
fn the_rows_group_per_team_in_the_open_saves_order() {
    let dir = crate::test_support::scratch("comparator_groups");
    // 73410's place on team 734's roster given to 74301 (so 74301 is on
    // 734 and on 743, and 73410 on no roster), and a height each on
    // 73401, 73410, 74301 and 74302. The report's order is 73401, 73410,
    // 74301, 74302 (the fixture's record order).
    let open_path = picked_save(&dir, "open", |file| {
        for id in [73401u32, 73410, 74301, 74302] {
            file.player_mut(id).expect("player").basic.height = 183;
        }
        file.team_mut(734).expect("team 734").roster[9].player_id = 74301;
    });
    let picked = picked_save(&dir, "picked", |_| {});
    let mut view = SaveView::new();
    view.open(&open_path);
    view.comparator.open = true;
    let mut window = Headless::sized(1100.0, 800.0);
    pick(&mut window, &mut view, picked);
    compared(&mut window, &mut view);
    let lines = window_lines(&mut window, &mut view, None, "Comparator");
    let at = lines
        .iter()
        .position(|line| line == "Team 734 /lit/")
        .expect("734's heading");
    assert_eq!(
        &lines[at..at + 11],
        [
            "Team 734 /lit/",
            "Player 73401  49 Muh Spooks",
            "Height 185 \u{2192} 183",
            // 74301 is on 734 and on 743: his first team is 734, and his
            // head wears his number on 734's slot.
            "Player 74301  46 NOTHING EVER HAPPENS",
            "Height 189 \u{2192} 183",
            "Team 743 /pol/",
            "Player 74302  50 STOP WATCHING PORN",
            "Height 185 \u{2192} 183",
            "No team",
            "Player 73410 DFW",
            "Height 180 \u{2192} 183",
        ],
        "one group per team in the open save's order: {lines:?}"
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("Team "))
            .count(),
        2,
        "each team headed once"
    );

    // A click on 74301's row (the Height button below his head) selects
    // him on 734; 73410's head and row are not buttons.
    let pairs = scroll_to(&mut window, &mut view, "Player 73410 DFW");
    let nodes = comparator_nodes(&pairs);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Player 73410 DFW")),
        "the no-team head is a label"
    );
    assert_eq!(
        nodes
            .iter()
            .filter(|node| {
                node.role() == Role::Button
                    && node
                        .label()
                        .is_some_and(|label| label.starts_with("Height "))
            })
            .count(),
        3,
        "the three rostered rows are buttons"
    );
    assert_eq!(
        nodes
            .iter()
            .filter(|node| node.value() == Some("Height 180 \u{2192} 183"))
            .count(),
        1,
        "73410's row is not clickable"
    );
    view.team = TeamChoice::Team(718);
    view.selected = Some((718, 0));
    // 74301 alone stands at 189 in the fixture: his row's label is his.
    action_click(
        &mut window,
        &mut view,
        comparator_button(&pairs, "Height 189 \u{2192} 183").0,
    );
    assert_eq!(view.selected, Some((734, 9)), "74301 on 734's slot");
    assert_eq!(view.team, TeamChoice::Team(734), "the selector moved");
}

/// A stripped player's mark beside his head: the picked save stripped,
/// the open save stripped, then both (the SE-DIF-04 fixture's build:
/// every player's appearance id at -1). The fixture's first record,
/// 70101 of team 701, is at the top of the list — in view without
/// scrolling. In the open-stripped case the head is his only line and
/// its click selects him (SE-GUI-13).
#[test]
fn a_stripped_players_mark_reads_beside_his_head() {
    let strip = |file: &mut EditFile, on: bool| {
        if on {
            for player in file.players_mut() {
                player.appearance_id = Some(STRIPPED_APPEARANCE_ID);
            }
        }
    };
    for (open_stripped, picked_stripped, mark) in [
        (false, true, "stripped in the picked save"),
        (true, false, "stripped in the open save"),
        (true, true, "stripped in both"),
    ] {
        let dir = crate::test_support::scratch(if open_stripped && picked_stripped {
            "comparator_mark_both"
        } else if open_stripped {
            "comparator_mark_open"
        } else {
            "comparator_mark_picked"
        });
        let open_path = picked_save(&dir, "open", |file| strip(file, open_stripped));
        let picked = picked_save(&dir, "picked", |file| strip(file, picked_stripped));
        let mut view = SaveView::new();
        view.open(&open_path);
        view.comparator.open = true;
        let mut window = Headless::sized(1100.0, 800.0);
        pick(&mut window, &mut view, picked);
        compared(&mut window, &mut view);
        let expected = format!("Player 70101  1 PLACEHOLDER, {mark}");
        let pairs = scroll_to(&mut window, &mut view, &expected);
        assert!(
            comparator_nodes(&pairs).iter().any(|node| {
                node.role() == Role::Button && node.label() == Some(expected.as_str())
            }),
            "the stripped mark beside the head: {expected}"
        );

        if open_stripped && !picked_stripped {
            // The head is a stripped-only player's only line: its click
            // selects him (the row's rule is the open save's rosters).
            // An AccessKit Click: the scroll's animation can keep a row's
            // bounds moving past the frame it was found on.
            view.team = TeamChoice::Team(718);
            view.selected = Some((718, 0));
            action_click(
                &mut window,
                &mut view,
                comparator_button(&pairs, &expected).0,
            );
            assert_eq!(view.selected, Some((701, 0)), "70101's team and slot");
            assert_eq!(view.team, TeamChoice::Team(701), "the selector moved");
        }
    }
}

// SE-GUI-13
#[test]
fn a_refused_pick_clears_the_held_result() {
    let (mut view, mut window, _) = compared_73401("comparator_refusal_clears");
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    let picked = picked_pes21(&dir);
    pick(&mut window, &mut view, picked);
    let lines = window_lines(&mut window, &mut view, None, "Comparator");
    assert!(
        lines.iter().any(|line| line
            == "The picked save is a PES 2021 save and the open save a PES 2017 one: both must be of one version"),
        "the refusal's literal: {lines:?}"
    );
    assert!(
        !lines.iter().any(|line| line == "Team 734 /lit/"),
        "the landed result's rows are gone: {lines:?}"
    );
    assert!(
        view.comparator.comparison.result.is_none() && view.comparator.comparison.pending.is_none(),
        "the comparison reset"
    );
}

// SE-GUI-13
#[test]
fn a_repick_of_the_same_path_recompares() {
    let (mut view, mut window, _) = compared_73401("comparator_repick");
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    // The picked file rewritten on disk under the same path (the member
    // changed the reference in the game): a second edit lands.
    let picked = picked_save(&dir, "picked", |file| {
        file.player_mut(73404)
            .expect("73404 is a player")
            .basic
            .height = 180;
    });
    pick(&mut window, &mut view, picked);
    compared(&mut window, &mut view);
    let pairs = scroll_to(&mut window, &mut view, "Height 180 \u{2192} 185");
    comparator_button(&pairs, "Height 180 \u{2192} 185");
    comparator_button(&pairs, "Height 185 \u{2192} 183");
}

// SE-GUI-13
#[test]
fn another_save_keeps_the_pick_and_recompares() {
    let (mut view, mut window, _) = compared_73401("comparator_reopen");
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    // A second save of the same version, carrying another edit: the pick
    // stays and the result is the new save's (a member compares several
    // saves against one reference).
    let second = picked_save(&dir, "second", |file| {
        file.player_mut(73402)
            .expect("73402 is a player")
            .basic
            .height = 180;
    });
    view.open(&second);
    compared(&mut window, &mut view);
    let lines = window_lines(&mut window, &mut view, None, "Comparator");
    let picked_path = dir.join("picked").display().to_string();
    assert!(
        lines.iter().any(|line| line == &picked_path),
        "the picked path still reads: {lines:?}"
    );
    let pairs = scroll_to(&mut window, &mut view, "Height 185 \u{2192} 180");
    comparator_button(&pairs, "Height 185 \u{2192} 180");
    assert!(
        !comparator_nodes(&pairs)
            .iter()
            .any(|node| node.label() == Some("Height 185 \u{2192} 183")),
        "the first save's row is gone"
    );

    // A save of another version: the pick is refused naming both, the
    // pick dropped.
    let pes21 = picked_pes21(&dir);
    view.open(&pes21);
    let lines = window_lines(&mut window, &mut view, None, "Comparator");
    assert!(
        lines.iter().any(|line| line == "(none)"),
        "the pick dropped: {lines:?}"
    );
    assert!(
        lines.iter().any(|line| line
            == "The picked save is a PES 2017 save and the open save a PES 2021 one: both must be of one version"),
        "the refusal's literal: {lines:?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("Player ") || line.starts_with("Team ")),
        "no rows: {lines:?}"
    );
}
