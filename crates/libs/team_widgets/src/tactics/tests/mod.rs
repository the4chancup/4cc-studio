//! The `tactics_settings` group's tests: the version gating, the
//! per-preset writes, the no-change cases and the per-control field
//! mapping — plus the `Headless` helpers `instructions`' and
//! `assignments`' test modules use (`crate::tactics::tests`).

use egui::accesskit::{self, Role};

use super::*;
use crate::test_support::{
    Headless, centre, control_right_of, controls_right_of, entry, entry_id, fixture_rows,
    fixture_tactics, pointer_at, roster_rows,
};

impl Headless {
    /// One frame of `tactics_settings` over `tactics`, `view` and
    /// `roster`: the frame's AccessKit nodes and whether the frame
    /// changed `tactics`.
    pub(crate) fn tactics(
        &mut self,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
        events: Vec<egui::Event>,
    ) -> (Vec<accesskit::Node>, bool) {
        self.tactics_at(None, version, tactics, view, roster, events)
    }

    /// `tactics` with `time` as the frame's timestamp (tooltips count
    /// their delay on it).
    pub(crate) fn tactics_at(
        &mut self,
        time: Option<f64>,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
        events: Vec<egui::Event>,
    ) -> (Vec<accesskit::Node>, bool) {
        let mut changed = None;
        let (nodes, _) = self.frame_at(time, events, |ui| {
            preset_selector(ui, view);
            changed = Some(tactics_settings(
                ui, "tactics", version, tactics, view, roster,
            ));
        });
        (nodes, changed.expect("the widget ran"))
    }

    /// A click at `pos` over the widget (press frame, then the release
    /// frame `clicked()` lands on).
    pub(crate) fn click_tactics(
        &mut self,
        pos: egui::Pos2,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> (Vec<accesskit::Node>, bool) {
        self.tactics(version, tactics, view, roster, pointer_at(pos, true));
        self.tactics(version, tactics, view, roster, pointer_at(pos, false))
    }

    /// `tactics` keeping the nodes' AccessKit ids (for
    /// `AccessKitActionRequest` events).
    pub(crate) fn tactics_ids(
        &mut self,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
        events: Vec<egui::Event>,
    ) -> (Vec<(accesskit::NodeId, accesskit::Node)>, bool) {
        let mut changed = None;
        let (nodes, _) = self.frame_at_ids(None, events, |ui| {
            preset_selector(ui, view);
            changed = Some(tactics_settings(
                ui, "tactics", version, tactics, view, roster,
            ));
        });
        (nodes, changed.expect("the widget ran"))
    }

    /// An AccessKit `Click` action on `id`: a click delivered as the
    /// accessibility event, for widgets a pointer click does not reach
    /// (a combo's popup items).
    pub(crate) fn action_click_tactics(
        &mut self,
        id: accesskit::NodeId,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> (Vec<accesskit::Node>, bool) {
        self.tactics(
            version,
            tactics,
            view,
            roster,
            vec![egui::Event::AccessKitActionRequest(
                accesskit::ActionRequest {
                    action: accesskit::Action::Click,
                    target_tree: accesskit::TreeId::ROOT,
                    target_node: id,
                    data: None,
                },
            )],
        )
    }

    /// The text of the tooltip drawn while the pointer rests at `pos`
    /// containing `needle` (bounded: a missing tooltip fails rather
    /// than hangs; needs the context's `tooltip_delay` at 0). The
    /// pointer first moves away for a frame: egui refuses a tooltip
    /// while the last click is more recent than the last movement.
    pub(crate) fn tactics_tooltip(
        &mut self,
        pos: egui::Pos2,
        needle: &str,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> Option<String> {
        self.tactics(
            version,
            tactics,
            view,
            roster,
            vec![egui::Event::PointerMoved(egui::Pos2::ZERO)],
        );
        self.tactics(
            version,
            tactics,
            view,
            roster,
            vec![egui::Event::PointerMoved(pos)],
        );
        for _ in 0..90 {
            let (nodes, _) = self.tactics(version, tactics, view, roster, vec![]);
            if let Some(value) = nodes.iter().find_map(|node| {
                node.value()
                    .filter(|value| value.contains(needle))
                    .map(str::to_owned)
            }) {
                return Some(value);
            }
        }
        None
    }
}

/// A click on `node`: press on one frame, release on the next.
pub(crate) fn click(
    window: &mut Headless,
    node: &accesskit::Node,
    version: PesVersion,
    tactics: &mut TeamTactics,
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
) -> (Vec<accesskit::Node>, bool) {
    window.click_tactics(centre(node), version, tactics, view, roster)
}

// plans/save_editor.md "Editing UI" (the version-gated bullet) on this
// tab: a field the version lacks stays in its place, disabled, its
// hover naming the first version that has it.
#[test]
fn fields_the_version_lacks_draw_disabled_with_their_versions() {
    let mut window = Headless::sized(800.0, 900.0);
    window
        .ctx
        .global_style_mut(|style| style.interaction.tooltip_delay = 0.0);
    // PES 16: the four instruction rows are drawn as on 17+, disabled
    // with "PES 17+", each combo showing the empty pick's text; and
    // Attack/defence levels is disabled with "PES 17+".
    let mut tactics = fixture_tactics(PesVersion::Pes16);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes16);
    let roster = roster_rows(&rows);
    let (nodes, _) = window.tactics(PesVersion::Pes16, &mut tactics, &mut view, &roster, vec![]);
    for label in ["Attack 1", "Attack 2", "Defence 1", "Defence 2"] {
        let combos = controls_right_of(&nodes, label, Role::ComboBox);
        assert_eq!(combos.len(), 2, "{label}'s instruction and target");
        for combo in &combos {
            assert!(combo.is_disabled(), "{label}'s combos are disabled");
            assert_eq!(
                combo.value(),
                Some("Unassigned"),
                "{label} shows the empty pick"
            );
        }
    }
    let first = controls_right_of(&nodes, "Attack 1", Role::ComboBox)[0];
    assert_eq!(
        window.tactics_tooltip(
            centre(first),
            "PES 17+",
            PesVersion::Pes16,
            &mut tactics,
            &mut view,
            &roster,
        ),
        Some("PES 17+".to_string()),
        "the instruction rows' hover"
    );
    let levels = entry(&nodes, Role::CheckBox, "Attack/defence levels");
    assert!(levels.is_disabled(), "the flag is disabled on PES 16");
    assert_eq!(
        window.tactics_tooltip(
            centre(levels),
            "PES 17+",
            PesVersion::Pes16,
            &mut tactics,
            &mut view,
            &roster,
        ),
        Some("PES 17+".to_string()),
        "Attack/defence levels' hover"
    );

    // PES 15: Fluid formation is disabled with "PES 16+".
    let mut tactics = fixture_tactics(PesVersion::Pes15);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes15);
    let roster = roster_rows(&rows);
    let (nodes, _) = window.tactics(PesVersion::Pes15, &mut tactics, &mut view, &roster, vec![]);
    let fluid = entry(&nodes, Role::CheckBox, "Fluid formation");
    assert!(fluid.is_disabled(), "the flag is disabled on PES 15");
    assert_eq!(
        window.tactics_tooltip(
            centre(fluid),
            "PES 16+",
            PesVersion::Pes15,
            &mut tactics,
            &mut view,
            &roster,
        ),
        Some("PES 16+".to_string()),
        "Fluid formation's hover"
    );
}

// Every control edits `presets[view.preset]` or the team-wide fields,
// nothing else: with preset 2 on screen, a slider move and a style
// click write the second preset alone.
#[test]
fn the_controls_edit_the_preset_on_screen() {
    let mut window = Headless::sized(800.0, 900.0);
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    let mut view = TacticsView {
        preset: 1,
        ..TacticsView::default()
    };
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);
    let presets_before = tactics.presets.clone();

    // The preset selector: pick 2 (writes `view`, not the record).
    let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
    let two = entry(&nodes, Role::Button, "2");
    let (_, changed) = click(&mut window, two, version, &mut tactics, &mut view, &roster);
    assert_eq!(view.preset, 1, "preset 2 stays on screen");
    assert!(!changed, "the preset selector writes no record");
    assert_eq!(tactics.presets, presets_before, "nor touches one");

    // Move Support range's slider one stop up.
    let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
    let slider = control_right_of(&nodes, "Support range", Role::Slider);
    let bounds = slider.bounds().expect("slider bounds");
    let from = egui::pos2(((bounds.x0 + bounds.x1) / 2.0) as f32, centre(slider).y);
    let to = egui::pos2(bounds.x1 as f32 - 4.0, from.y);
    let mut changed = false;
    for events in [
        pointer_at(from, true),
        vec![egui::Event::PointerMoved(to)],
        pointer_at(to, false),
    ] {
        let (_, frame_changed) = window.tactics(version, &mut tactics, &mut view, &roster, events);
        changed |= frame_changed;
    }
    assert!(changed, "the drag changed the record");
    assert_ne!(
        tactics.presets[1].sliders, presets_before[1].sliders,
        "preset 2's sliders moved"
    );
    assert_eq!(tactics.presets[0], presets_before[0], "preset 1 untouched");
    assert_eq!(tactics.presets[2], presets_before[2], "preset 3 untouched");

    // Click a style pair's on choice.
    let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
    let possession = entry(&nodes, Role::Button, "Possession");
    let (_, changed) = click(
        &mut window,
        possession,
        version,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert!(changed, "the click changed the record");
    assert!(
        tactics.presets[1].style.attacking_style,
        "preset 2's attacking style is Possession"
    );
    assert_eq!(tactics.presets[0], presets_before[0], "preset 1 untouched");
    assert_eq!(tactics.presets[2], presets_before[2], "preset 3 untouched");
}

// plans/save_editor.md "Tactics editor" (the widget's contract): an
// idle frame, a hover, opening a combo and closing it, or re-picking
// the current value reports no change and leaves the record equal.
#[test]
fn no_gesture_no_change() {
    let mut window = Headless::sized(800.0, 900.0);
    for version in [PesVersion::Pes15, PesVersion::Pes17, PesVersion::Pes21] {
        let mut tactics = fixture_tactics(version);
        let before = tactics.clone();
        let mut view = TacticsView::default();
        let rows = fixture_rows(version);
        let roster = roster_rows(&rows);
        let (nodes, changed) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        assert!(!changed, "{version:?}: an idle frame reports no change");
        assert_eq!(tactics, before, "{version:?}: and edits nothing");

        // Hover over every interactive control: each frame reports no
        // change and edits nothing.
        let controls: Vec<egui::Pos2> = nodes
            .iter()
            .filter(|node| {
                matches!(
                    node.role(),
                    Role::Button
                        | Role::CheckBox
                        | Role::ComboBox
                        | Role::Slider
                        | Role::SpinButton
                ) && !node.is_disabled()
            })
            .map(centre)
            .collect();
        for pos in controls {
            let (_, changed) = window.tactics(
                version,
                &mut tactics,
                &mut view,
                &roster,
                vec![egui::Event::PointerMoved(pos)],
            );
            assert!(!changed, "{version:?}: a hover reports no change");
            assert_eq!(tactics, before, "{version:?}: a hover edits nothing");
        }

        if version == PesVersion::Pes15 {
            continue;
        }
        // Open the Attack 1 instruction combo and close it again.
        let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);
        let (_, changed) = click(
            &mut window,
            combo,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(!changed, "{version:?}: opening the combo writes nothing");
        assert_eq!(tactics, before, "{version:?}: and edits nothing");

        // Re-pick the current value inside it (the fixture's Attack 1
        // is Off, "Unassigned"): no change, nothing written.
        let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &roster, vec![]);
        let same = entry_id(&ids, Role::Button, "Unassigned");
        let (_, changed) =
            window.action_click_tactics(same, version, &mut tactics, &mut view, &roster);
        assert!(!changed, "{version:?}: re-picking the value writes nothing");
        assert_eq!(tactics, before, "{version:?}: and edits nothing");

        // An out-of-range stored slider value shows as stored and is
        // not rewritten until the member moves it.
        let mut out_of_range = before.clone();
        out_of_range.presets[0].sliders.support_range = 0;
        let expected = out_of_range.clone();
        let (nodes, changed) =
            window.tactics(version, &mut out_of_range, &mut view, &roster, vec![]);
        assert!(
            !changed,
            "{version:?}: an out-of-range value is not rewritten"
        );
        assert_eq!(out_of_range, expected, "{version:?}: nor clamped on draw");
        let slider = control_right_of(&nodes, "Support range", Role::Slider);
        assert_eq!(
            slider.numeric_value(),
            Some(0.0),
            "{version:?}: the slider shows the stored value"
        );

        // A click on the open combo toggles it closed again.
        let (_, changed) = click(
            &mut window,
            combo,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(!changed, "{version:?}: closing the combo writes nothing");
        assert_eq!(tactics, before, "{version:?}: and edits nothing");
    }
}

// A control's AccessKit id: the `role` on `label`'s row to its right
// (a slider's or spinner's node has no label of its own).
fn id_right_of(
    ids: &[(accesskit::NodeId, accesskit::Node)],
    label: &str,
    role: Role,
) -> accesskit::NodeId {
    let label_bounds = ids
        .iter()
        .find(|(_, node)| node.role() == Role::Label && node.value() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("no label {label:?}"));
    ids.iter()
        .find(|(_, node)| {
            node.role() == role
                && node.bounds().is_some_and(|b| {
                    (b.y0 - label_bounds.y0).abs() < 4.0 && b.x0 >= label_bounds.x1 - 1.0
                })
        })
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no {role:?} right of {label:?}"))
}

// The style pair `label`'s `choice` button's id (the choices' texts
// repeat across pairs — "Wide" twice — so the row pins which).
fn button_right_of(
    ids: &[(accesskit::NodeId, accesskit::Node)],
    label: &str,
    choice: &str,
) -> accesskit::NodeId {
    let label_bounds = ids
        .iter()
        .find(|(_, node)| node.role() == Role::Label && node.value() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("no label {label:?}"));
    ids.iter()
        .find(|(_, node)| {
            node.role() == Role::Button
                && node.label() == Some(choice)
                && node.bounds().is_some_and(|b| {
                    (b.y0 - label_bounds.y0).abs() < 4.0 && b.x0 >= label_bounds.x1 - 1.0
                })
        })
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no {choice:?} right of {label:?}"))
}

/// How a case interacts: a `SetValue` on the slider's or the
/// spinner's node, a `Click` on a style pair's choice or a checkbox.
#[derive(Clone, Copy)]
enum Control {
    /// Label and the value to set (the range's top).
    Slider(&'static str, f64),
    /// The pair's label and the choice's text.
    Choice(&'static str, &'static str),
    /// The checkbox's label (the click toggles it on).
    Check(&'static str),
    /// The DragValue's label and the value to set.
    Spin(&'static str, f64),
}

// Every Formation settings and Automatic settings control writes its
// own field: one interaction on a fresh fixture, then the record is
// the fixture with that field alone set.
#[test]
fn each_settings_control_writes_its_own_field() {
    let mut window = Headless::sized(1400.0, 1100.0);
    let version = PesVersion::Pes21;
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);
    // One fixture parse for every case (each wants the fresh record).
    let base = fixture_tactics(version);
    /// One case: what it is about, how to interact, the fixture write
    /// before it and the expected write after.
    struct Case {
        /// The case's name (the assertion text).
        name: &'static str,
        /// The interaction.
        control: Control,
        /// Puts the field opposite the pick (a fresh fixture may
        /// already hold the target).
        setup: fn(&mut TeamTactics),
        /// Sets the named field, on a fixture clone.
        expect: fn(&mut TeamTactics),
    }
    let cases: &[Case] = &[
        Case {
            name: "Support range",
            control: Control::Slider("Support range", 10.0),
            setup: |t| t.presets[0].sliders.support_range = 1,
            expect: |t| t.presets[0].sliders.support_range = 10,
        },
        Case {
            name: "Defensive line",
            control: Control::Slider("Defensive line", 10.0),
            setup: |t| t.presets[0].sliders.defensive_line = 1,
            expect: |t| t.presets[0].sliders.defensive_line = 10,
        },
        Case {
            name: "Compactness",
            control: Control::Slider("Compactness", 10.0),
            setup: |t| t.presets[0].sliders.compactness = 1,
            expect: |t| t.presets[0].sliders.compactness = 10,
        },
        Case {
            name: "Numbers in attack",
            control: Control::Slider("Numbers in attack", 3.0),
            setup: |t| t.presets[0].sliders.numbers_in_attack = 1,
            expect: |t| t.presets[0].sliders.numbers_in_attack = 3,
        },
        Case {
            name: "Numbers in defence",
            control: Control::Slider("Numbers in defence", 3.0),
            setup: |t| t.presets[0].sliders.numbers_in_defence = 1,
            expect: |t| t.presets[0].sliders.numbers_in_defence = 3,
        },
        Case {
            name: "Attacking style on",
            control: Control::Choice("Attacking style", "Possession"),
            setup: |t| t.presets[0].style.attacking_style = false,
            expect: |t| t.presets[0].style.attacking_style = true,
        },
        Case {
            name: "Attacking style off",
            control: Control::Choice("Attacking style", "Counter Attack"),
            setup: |t| t.presets[0].style.attacking_style = true,
            expect: |t| t.presets[0].style.attacking_style = false,
        },
        Case {
            name: "Attacking zone on",
            control: Control::Choice("Attacking zone", "Wide"),
            setup: |t| t.presets[0].style.attacking_zone = false,
            expect: |t| t.presets[0].style.attacking_zone = true,
        },
        Case {
            name: "Attacking zone off",
            control: Control::Choice("Attacking zone", "Centre"),
            setup: |t| t.presets[0].style.attacking_zone = true,
            expect: |t| t.presets[0].style.attacking_zone = false,
        },
        Case {
            name: "Buildup on",
            control: Control::Choice("Buildup", "Short Pass"),
            setup: |t| t.presets[0].style.buildup = false,
            expect: |t| t.presets[0].style.buildup = true,
        },
        Case {
            name: "Buildup off",
            control: Control::Choice("Buildup", "Long Pass"),
            setup: |t| t.presets[0].style.buildup = true,
            expect: |t| t.presets[0].style.buildup = false,
        },
        Case {
            name: "Positioning on",
            control: Control::Choice("Positioning", "Flexible"),
            setup: |t| t.presets[0].style.positioning = false,
            expect: |t| t.presets[0].style.positioning = true,
        },
        Case {
            name: "Positioning off",
            control: Control::Choice("Positioning", "Maintain"),
            setup: |t| t.presets[0].style.positioning = true,
            expect: |t| t.presets[0].style.positioning = false,
        },
        Case {
            name: "Defensive style on",
            control: Control::Choice("Defensive style", "All Out Defence"),
            setup: |t| t.presets[0].style.defensive_style = false,
            expect: |t| t.presets[0].style.defensive_style = true,
        },
        Case {
            name: "Defensive style off",
            control: Control::Choice("Defensive style", "Frontline Pressure"),
            setup: |t| t.presets[0].style.defensive_style = true,
            expect: |t| t.presets[0].style.defensive_style = false,
        },
        Case {
            name: "Containment area on",
            control: Control::Choice("Containment area", "Wide"),
            setup: |t| t.presets[0].style.containment_area = false,
            expect: |t| t.presets[0].style.containment_area = true,
        },
        Case {
            name: "Containment area off",
            control: Control::Choice("Containment area", "Middle"),
            setup: |t| t.presets[0].style.containment_area = true,
            expect: |t| t.presets[0].style.containment_area = false,
        },
        Case {
            name: "Pressure on",
            control: Control::Choice("Pressure", "Conservative"),
            setup: |t| t.presets[0].style.pressure = false,
            expect: |t| t.presets[0].style.pressure = true,
        },
        Case {
            name: "Pressure off",
            control: Control::Choice("Pressure", "Aggressive"),
            setup: |t| t.presets[0].style.pressure = true,
            expect: |t| t.presets[0].style.pressure = false,
        },
        Case {
            name: "Fluid formation",
            control: Control::Check("Fluid formation"),
            setup: |t| t.presets[0].style.fluid = Some(false),
            expect: |t| t.presets[0].style.fluid = Some(true),
        },
        Case {
            name: "Substitution",
            control: Control::Spin("Substitution", 3.0),
            setup: |t| t.auto.substitution = Some(0),
            expect: |t| t.auto.substitution = Some(3),
        },
        Case {
            name: "Offside trap",
            control: Control::Check("Offside trap"),
            setup: |t| t.auto.offside_trap = Some(false),
            expect: |t| t.auto.offside_trap = Some(true),
        },
        Case {
            name: "Preset change",
            control: Control::Check("Preset change"),
            setup: |t| t.auto.preset_change = Some(false),
            expect: |t| t.auto.preset_change = Some(true),
        },
        Case {
            name: "Attack/defence levels",
            control: Control::Check("Attack/defence levels"),
            setup: |t| t.auto.attack_defence_levels = Some(false),
            expect: |t| t.auto.attack_defence_levels = Some(true),
        },
    ];
    for case in cases {
        let mut tactics = base.clone();
        (case.setup)(&mut tactics);
        let mut expected = base.clone();
        (case.expect)(&mut expected);
        let mut view = TacticsView::default();
        let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &roster, vec![]);
        let (id, action, data) = match case.control {
            Control::Slider(label, value) => (
                id_right_of(&ids, label, Role::Slider),
                accesskit::Action::SetValue,
                Some(accesskit::ActionData::NumericValue(value)),
            ),
            Control::Spin(label, value) => (
                id_right_of(&ids, label, Role::SpinButton),
                accesskit::Action::SetValue,
                Some(accesskit::ActionData::NumericValue(value)),
            ),
            Control::Choice(label, choice) => (
                button_right_of(&ids, label, choice),
                accesskit::Action::Click,
                None,
            ),
            Control::Check(label) => (
                entry_id(&ids, Role::CheckBox, label),
                accesskit::Action::Click,
                None,
            ),
        };
        let (_, changed) = window.tactics(
            version,
            &mut tactics,
            &mut view,
            &roster,
            vec![egui::Event::AccessKitActionRequest(
                accesskit::ActionRequest {
                    action,
                    target_tree: accesskit::TreeId::ROOT,
                    target_node: id,
                    data,
                },
            )],
        );
        let name = case.name;
        assert!(changed, "{name}: the interaction changed the record");
        assert_eq!(
            tactics, expected,
            "{name}: the one field written, the rest untouched"
        );
    }
}

// The wrap's measured cell widths are real: (a) no cell's controls cross
// a label's left edge — an underestimate would pack the next cell over
// this one; (b) at a window wide enough for the five slider cells in one
// row the five share a row's y, and the seven style pairs sit on as few
// rows as their measured widths allow — an overestimate wraps early.
#[test]
fn the_wrap_cells_pack_without_overlapping_or_wrapping_early() {
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);
    let slider_labels = [
        "Support range",
        "Defensive line",
        "Compactness",
        "Numbers in attack",
        "Numbers in defence",
    ];
    let pair_labels = [
        "Attacking style",
        "Attacking zone",
        "Buildup",
        "Positioning",
        "Defensive style",
        "Containment area",
        "Pressure",
    ];
    let label_bounds = |nodes: &[accesskit::Node], label: &str| {
        nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some(label))
            .and_then(|node| node.bounds())
            .unwrap_or_else(|| panic!("no label {label:?}"))
    };

    // Measure the cells at a wide window: the wrap is one flowing line —
    // a cell's drawn width is its label's left to its last control's
    // right (bounded by the next cell's label on the same row).
    let mut measure = Headless::sized(1600.0, 900.0);
    let (nodes, _) = measure.tactics(version, &mut tactics, &mut view, &roster, vec![]);
    let mut cells: Vec<accesskit::Rect> = slider_labels
        .iter()
        .chain(pair_labels.iter())
        .map(|label| label_bounds(&nodes, label))
        .collect();
    cells.sort_by(|a, b| {
        (a.y0, a.x0)
            .partial_cmp(&(b.y0, b.x0))
            .expect("ordered bounds")
    });
    let cell_widths: Vec<f64> = cells
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let next_on_row = cells
                .get(index + 1)
                .and_then(|next| ((next.y0 - label.y0).abs() < 4.0).then_some(next.x0));
            nodes
                .iter()
                .filter(|node| {
                    matches!(
                        node.role(),
                        Role::Slider
                            | Role::SpinButton
                            | Role::Button
                            | Role::CheckBox
                            | Role::ComboBox
                    ) && node.bounds().is_some_and(|bounds| {
                        (bounds.y0 - label.y0).abs() < 4.0
                            && bounds.x0 >= label.x0
                            && next_on_row.is_none_or(|next| bounds.x0 < next - 1.0)
                    })
                })
                .filter_map(|node| node.bounds())
                .map(|bounds| bounds.x1)
                .fold(label.x1, f64::max)
                - label.x0
        })
        .collect();

    // The five slider cells' summed widths plus margins — 1,284 px here
    // (247 + 246 + 242 + 269 + 281) — the window for the packing checks.
    let natural: f64 = cell_widths[..5].iter().sum();
    let width = (natural + 40.0).ceil() as f32;

    let mut window = Headless::sized(width, 900.0);
    let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);

    // (a) the allocated cell covers its drawn contents: the formula's
    // width is at least the drawn width for every cell (an underestimate
    // shows as drawn content past the row's right edge, inside the
    // cell's own span).
    let mut expected = Vec::new();
    window.frame(vec![], |ui| {
        let spacing = ui.spacing().item_spacing.x;
        for spec in &SLIDERS {
            expected.push(
                text_width(ui, spec.label)
                    + ui.spacing().slider_width
                    + ui.spacing().interact_size.x
                    + 2.0 * spacing,
            );
        }
        for toggle in &STYLE_TOGGLES {
            expected.push(
                text_width(ui, toggle.label)
                    + choice_width(ui, toggle.off)
                    + choice_width(ui, toggle.on)
                    + 2.0 * spacing,
            );
        }
    });
    for (index, cell) in cell_widths.iter().enumerate() {
        let name = slider_labels
            .iter()
            .chain(pair_labels.iter())
            .nth(index)
            .expect("the cell's name");
        let estimated = expected[index];
        assert!(
            *cell <= f64::from(estimated) + 1.0,
            "{name}: allocated {estimated} under its drawn {cell}"
        );
    }

    // (b) the five sliders share one row's y.
    let rows: std::collections::BTreeSet<i64> = slider_labels
        .iter()
        .map(|label| label_bounds(&nodes, label).y0 as i64)
        .collect();
    assert_eq!(rows.len(), 1, "the five slider cells share one row");

    // And the seven pairs take as few rows as their measured widths
    // allow: a greedy pack of the measured cells (the inter-cell gap is
    // inside each span already) — their ~190-320 px cells land 5 + 2 on
    // the ~1,320 px above.
    let mut greedy_rows = 1;
    let mut used = 0.0;
    for cell in &cell_widths[5..] {
        if used > 0.0 && used + cell > f64::from(width) {
            greedy_rows += 1;
            used = *cell;
        } else {
            used += cell;
        }
    }
    let drawn_rows: std::collections::BTreeSet<i64> = pair_labels
        .iter()
        .map(|label| label_bounds(&nodes, label).y0 as i64)
        .collect();
    assert_eq!(
        drawn_rows.len(),
        greedy_rows,
        "the pairs take the fewest rows their widths allow \
         (cells {cell_widths:?}, window {width})"
    );
}

// The Automatic settings row wraps like the formation cells' (the
// draft's `.row`): at a pane narrower than the row the last checkbox
// lands on a new line below the Substitution pair, inside the pane; the
// armed note's button lands inside too. (Inside a host's vertical-only
// scroll area the region grows to the content's width, so the wrap is
// exercised by the pane itself, not the window's clip.)
#[test]
fn the_automatic_settings_and_the_note_wrap_at_a_narrow_pane() {
    let mut window = Headless::sized(400.0, 900.0);
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);

    let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
    let substitution = control_right_of(&nodes, "Substitution", Role::SpinButton);
    let levels = entry(&nodes, Role::CheckBox, "Attack/defence levels");
    assert!(
        levels.bounds().expect("checkbox bounds").x1 <= 400.0,
        "Attack/defence levels inside the pane"
    );
    assert!(
        levels.bounds().expect("checkbox bounds").y0
            > substitution.bounds().expect("field bounds").y0,
        "it wrapped to a new line below the Substitution pair"
    );

    // Arm a role: the note's button lands inside the pane.
    view.assign = Some(Assign::Role(5));
    let (nodes, _) = window.frame(vec![], |ui| {
        assign_note(ui, &mut view);
        tactics_settings(ui, "tactics", version, &mut tactics, &mut view, &roster);
    });
    let cancel = entry(&nodes, Role::Button, "Cancel");
    assert!(
        cancel.bounds().expect("button bounds").x1 <= 400.0,
        "the note's button inside the pane"
    );
}
