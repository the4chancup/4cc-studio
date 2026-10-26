//! The `tactics_settings` group's tests: the version gating, the
//! per-preset writes, the no-change cases and the per-control field
//! mapping — plus the `Headless` helpers `instructions`' and
//! `assignments`' test modules use (`crate::tactics::tests`).

use egui::accesskit::{self, Role};

use pes_savefile::model::instruction::Instruction;

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
    /// hover must add a node holding `needle` to the frame the pointer
    /// left from: a label the view always draws with the same text (a
    /// closed combo's `value` is the selected text its tooltip repeats)
    /// cannot pass for the tooltip. The pointer first moves away for a
    /// frame: egui refuses a tooltip while the last click is more recent
    /// than the last movement.
    pub(crate) fn tactics_tooltip(
        &mut self,
        pos: egui::Pos2,
        needle: &str,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        roster: &[RosterRow<'_>],
    ) -> Option<String> {
        let holding = |nodes: &[accesskit::Node]| -> Vec<String> {
            nodes
                .iter()
                .filter_map(|node| node.value())
                .filter(|value| value.contains(needle))
                .map(str::to_owned)
                .collect()
        };
        let (resting, _) = self.tactics(
            version,
            tactics,
            view,
            roster,
            vec![egui::Event::PointerMoved(egui::Pos2::ZERO)],
        );
        let resting = holding(&resting);
        self.tactics(
            version,
            tactics,
            view,
            roster,
            vec![egui::Event::PointerMoved(pos)],
        );
        let count =
            |values: &[String], value: &String| values.iter().filter(|v| *v == value).count();
        for _ in 0..90 {
            let (nodes, _) = self.tactics(version, tactics, view, roster, vec![]);
            let values = holding(&nodes);
            if let Some(added) = values
                .iter()
                .find(|value| count(&values, value) > count(&resting, value))
            {
                return Some(added.clone());
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

/// The formation grid's pair labels, the draft's order (the five
/// sliders, then the seven style pairs).
const FORMATION_LABELS: [&str; 12] = [
    "Support range",
    "Defensive line",
    "Compactness",
    "Numbers in attack",
    "Numbers in defence",
    "Attacking style",
    "Attacking zone",
    "Buildup",
    "Positioning",
    "Defensive style",
    "Containment area",
    "Pressure",
];

/// The bounds of the Label node showing `label` (a grid's label cell).
fn formation_label_bounds(nodes: &[accesskit::Node], label: &str) -> accesskit::Rect {
    nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some(label))
        .and_then(|node| node.bounds())
        .unwrap_or_else(|| panic!("no label {label:?}"))
}

/// The x of the leftmost control on `label`'s grid row: the control
/// cell's left edge (the slider, or the pair's first choice).
fn control_x0(nodes: &[accesskit::Node], label: &str) -> f64 {
    let bounds = formation_label_bounds(nodes, label);
    nodes
        .iter()
        .filter(|node| {
            node.role() != Role::Label
                && node
                    .bounds()
                    .is_some_and(|b| (b.y0 - bounds.y0).abs() < 4.0 && b.x0 >= bounds.x1 - 1.0)
        })
        .filter_map(|node| node.bounds())
        .map(|b| b.x0)
        .fold(f64::MAX, f64::min)
}

// plans/save_editor.md "Tactics editor" (the formation settings' aligned
// grid of up to two pairs a row): at a pane the bound lets two pairs
// share a row (900 wide here), Support range and Defensive line sit on
// one row and the twelve labels take the two label columns; at a pane
// under it (500) they take one column and the controls one more.
#[test]
fn the_formation_grid_pairs_by_the_columns_width() {
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);

    let mut window = Headless::sized(900.0, 900.0);
    let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
    assert!(
        (formation_label_bounds(&nodes, "Support range").y0
            - formation_label_bounds(&nodes, "Defensive line").y0)
            .abs()
            < 1.0,
        "two pairs on the first row"
    );
    let columns: std::collections::BTreeSet<i64> = FORMATION_LABELS
        .iter()
        .map(|label| formation_label_bounds(&nodes, label).x0 as i64)
        .collect();
    assert_eq!(
        columns.len(),
        2,
        "the twelve labels sit in two label columns"
    );

    let mut narrow = Headless::sized(500.0, 900.0);
    let (nodes, _) = narrow.tactics(version, &mut tactics, &mut view, &roster, vec![]);
    let label_x0 = formation_label_bounds(&nodes, FORMATION_LABELS[0]).x0;
    let controls_x0 = control_x0(&nodes, FORMATION_LABELS[0]);
    for label in &FORMATION_LABELS[1..] {
        assert_eq!(
            formation_label_bounds(&nodes, label).x0,
            label_x0,
            "{label} shares the labels' column"
        );
        assert_eq!(
            control_x0(&nodes, label),
            controls_x0,
            "{label}'s control shares the controls' column"
        );
    }
}

// The two-pairs bound is computed from the measures, never read back
// from a drawn frame: at a pane that draws two pairs (900 wide) the
// drawn row's width sits under the estimate and only a few points below
// it (8.0 here, the one cell gap of slack) — an underestimate would let
// a pane at the threshold clip a row's end, a large overestimate wraps
// rows that fit.
#[test]
fn the_pairs_threshold_covers_the_drawn_row() {
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);
    let mut window = Headless::sized(900.0, 900.0);
    let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);

    // The grid's drawn width: from the labels' left edge to the widest
    // row's rightmost control (bounded by the grid's rows, the fluid
    // checkbox above and the instructions table below).
    let grid_left = FORMATION_LABELS
        .iter()
        .map(|label| formation_label_bounds(&nodes, label).x0)
        .fold(f64::MAX, f64::min);
    let grid_top = formation_label_bounds(&nodes, FORMATION_LABELS[0]).y0 - 2.0;
    let grid_bottom = FORMATION_LABELS
        .iter()
        .map(|label| formation_label_bounds(&nodes, label).y0)
        .fold(0.0, f64::max)
        + 24.0;
    let grid_right = nodes
        .iter()
        .filter(|node| {
            node.role() != Role::Label
                && node
                    .bounds()
                    .is_some_and(|b| b.y0 >= grid_top && b.y1 <= grid_bottom && b.x0 >= grid_left)
        })
        .filter_map(|node| node.bounds())
        .map(|b| b.x1)
        .fold(0.0, f64::max);
    let drawn = grid_right - grid_left;

    let mut estimate = 0.0f64;
    window.frame(vec![], |ui| estimate = f64::from(two_pairs_width(ui)));
    assert!(
        drawn <= estimate,
        "the bound covers the drawn row ({drawn} drawn vs {estimate} estimated)"
    );
    assert!(
        estimate - drawn <= 16.0,
        "by no more than a few points ({estimate} estimated vs {drawn} drawn)"
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

    // Arm join: at a pane narrower than the note plus its button, Done
    // wraps to a line below the note and still lands inside the pane.
    view.assign = Some(Assign::Join);
    let mut narrow = Headless::sized(360.0, 900.0);
    let (nodes, _) = narrow.frame(vec![], |ui| {
        assign_note(ui, &mut view);
        tactics_settings(ui, "tactics", version, &mut tactics, &mut view, &roster);
    });
    let note = nodes
        .iter()
        .find(|node| {
            node.role() == Role::Label
                && node.value()
                    == Some("Click pitch players to join or leave the attack (up to three).")
        })
        .expect("the join note");
    let done = entry(&nodes, Role::Button, "Done");
    let note_bounds = note.bounds().expect("note bounds");
    let done_bounds = done.bounds().expect("button bounds");
    assert!(
        done_bounds.y0 > note_bounds.y0 + 4.0,
        "Done starts below the note's first line (note {note_bounds:?}, Done {done_bounds:?})"
    );
    assert!(done_bounds.x1 <= 360.0, "Done ends inside the pane");
}

/// A roster of named slots for `tactics_term`: the fixture's own names
/// are all "PLACEHOLDER".
fn named_roster() -> Vec<(String, String, Option<u8>)> {
    (0..32)
        .map(|i| (format!("Player {:02}", i + 1), String::new(), None))
        .collect()
}

/// `tactics_term` words the preset fields with the tab's labels: the
/// sliders' numbers, the style pairs' texts, the fluid switch's
/// "off"/"on".
#[test]
fn the_preset_fields_terms() {
    let version = PesVersion::Pes21;
    let tactics = fixture_tactics(version);
    let rows = named_roster();
    let roster = roster_rows(&rows);
    let term =
        |field, old, new| tactics_term(version, field, old, new, &tactics, &tactics, &roster);
    assert_eq!(
        term(
            TacticsField::Preset {
                preset: 0,
                field: PresetField::SupportRange,
            },
            5,
            7
        ),
        (
            "Preset 1 · Support range".to_owned(),
            "5".to_owned(),
            "7".to_owned()
        )
    );
    assert_eq!(
        term(
            TacticsField::Preset {
                preset: 0,
                field: PresetField::AttackingStyle,
            },
            0,
            1
        ),
        (
            "Preset 1 · Attacking style".to_owned(),
            "Counter Attack".to_owned(),
            "Possession".to_owned()
        )
    );
    assert_eq!(
        term(
            TacticsField::Preset {
                preset: 2,
                field: PresetField::DefensiveStyle,
            },
            1,
            0
        ),
        (
            "Preset 3 · Defensive style".to_owned(),
            "All Out Defence".to_owned(),
            "Frontline Pressure".to_owned()
        )
    );
    assert_eq!(
        term(
            TacticsField::Preset {
                preset: 1,
                field: PresetField::FluidFormation,
            },
            0,
            1
        ),
        (
            "Preset 2 · Fluid formation".to_owned(),
            "off".to_owned(),
            "on".to_owned()
        )
    );
}

/// The set-piece takers' and the captain's terms: the chips' labels and
/// the holder's name, "Unassigned" for 255.
#[test]
fn the_set_pieces_terms() {
    let version = PesVersion::Pes21;
    let tactics = fixture_tactics(version);
    let rows = named_roster();
    let roster = roster_rows(&rows);
    let term =
        |field, old, new| tactics_term(version, field, old, new, &tactics, &tactics, &roster);
    assert_eq!(
        term(TacticsField::FreeKickTakerLong, 3, 9),
        (
            "Long free kick".to_owned(),
            "Player 04".to_owned(),
            "Player 10".to_owned()
        )
    );
    assert_eq!(
        term(TacticsField::Captain, 0, 5),
        (
            "Captain".to_owned(),
            "Player 01".to_owned(),
            "Player 06".to_owned()
        )
    );
    assert_eq!(
        term(TacticsField::PenaltyTaker, 255, 2),
        (
            "Penalty kick".to_owned(),
            "Unassigned".to_owned(),
            "Player 03".to_owned()
        )
    );
}

/// A formation slot's terms name the formation and the starter playing
/// it — through `starting_eleven`, not the slot's index — and the part:
/// the position's label, the coordinates' numbers.
#[test]
fn the_formation_slots_terms() {
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    // A non-identity eleven: wording slot 4's player as roster slot 4
    // would pass here too if `starting_eleven` were not read.
    tactics.starting_eleven[4] = 7;
    let rows = named_roster();
    let roster = roster_rows(&rows);
    let term =
        |field, old, new| tactics_term(version, field, old, new, &tactics, &tactics, &roster);
    assert_eq!(
        term(
            TacticsField::Formation {
                preset: 0,
                formation: 0,
                slot: 4,
                part: FormationPart::Position,
            },
            9,
            10
        ),
        (
            "Preset 1 · Kick-off · Player 08 · Position".to_owned(),
            "LWF".to_owned(),
            "RWF".to_owned()
        )
    );
    assert_eq!(
        term(
            TacticsField::Formation {
                preset: 1,
                formation: 2,
                slot: 4,
                part: FormationPart::X,
            },
            30,
            40
        ),
        (
            "Preset 2 · Out-of-possession · Player 08 · X".to_owned(),
            "30".to_owned(),
            "40".to_owned()
        )
    );
    assert_eq!(
        term(
            TacticsField::Formation {
                preset: 0,
                formation: 1,
                slot: 4,
                part: FormationPart::Y,
            },
            20,
            44
        ),
        (
            "Preset 1 · In-possession · Player 08 · Y".to_owned(),
            "20".to_owned(),
            "44".to_owned()
        )
    );
}

/// A man-marking row names its own slot's starter and shows the marked
/// opponent's slot, "None" for 255.
#[test]
fn the_man_marking_terms() {
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    tactics.starting_eleven[5] = 2;
    let rows = named_roster();
    let roster = roster_rows(&rows);
    let term =
        |field, old, new| tactics_term(version, field, old, new, &tactics, &tactics, &roster);
    assert_eq!(
        term(TacticsField::ManMarking { preset: 0, slot: 5 }, 255, 3),
        (
            "Preset 1 · Man marking · Player 03".to_owned(),
            "None".to_owned(),
            "3".to_owned()
        )
    );
}

/// The instruction rows: the row label (`Attack 1` to `Defence 2`), the
/// instruction's text as the version's table decodes it (its hex past
/// the table), the target's combo text.
#[test]
fn the_instructions_terms() {
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    // The target bytes word under the instruction in force: a
    // player-taking one (Counter Target, Defensive) names them, the
    // fixture's Offs would read Unassigned.
    let encode = |inst| instruction::encode(version, inst).expect("PES 21 stores it");
    tactics.presets[0]
        .attack_instructions
        .as_mut()
        .expect("PES 21 has instructions")[0]
        .instruction = encode(Instruction::CounterTarget);
    tactics.presets[0]
        .defence_instructions
        .as_mut()
        .expect("PES 21 has instructions")[0]
        .instruction = encode(Instruction::Defensive);
    let rows = named_roster();
    let roster = roster_rows(&rows);
    let term =
        |field, old, new| tactics_term(version, field, old, new, &tactics, &tactics, &roster);
    assert_eq!(
        term(
            TacticsField::Instruction {
                preset: 0,
                side: InstructionSide::Attack,
                index: 0,
                part: InstructionPart::Instruction,
            },
            6,
            12
        ),
        (
            "Preset 1 · Attack 1".to_owned(),
            "Tiki-Taka".to_owned(),
            "Deep Defensive Line".to_owned()
        )
    );
    assert_eq!(
        term(
            TacticsField::Instruction {
                preset: 0,
                side: InstructionSide::Defence,
                index: 1,
                part: InstructionPart::Instruction,
            },
            6,
            0xAB
        ),
        (
            "Preset 1 · Defence 2".to_owned(),
            "Tiki-Taka".to_owned(),
            "Unknown (0xAB)".to_owned()
        )
    );
    assert_eq!(
        term(
            TacticsField::Instruction {
                preset: 0,
                side: InstructionSide::Attack,
                index: 0,
                part: InstructionPart::PlayerId,
            },
            5,
            255
        ),
        (
            "Preset 1 · Attack 1 · Target".to_owned(),
            "6 Player 06".to_owned(),
            "Unassigned".to_owned()
        )
    );
    // A target that is no starter names the roster slot alone.
    assert_eq!(
        term(
            TacticsField::Instruction {
                preset: 0,
                side: InstructionSide::Defence,
                index: 0,
                part: InstructionPart::PlayerId,
            },
            20,
            5
        ),
        (
            "Preset 1 · Defence 1 · Target".to_owned(),
            "Player 21".to_owned(),
            "6 Player 06".to_owned()
        )
    );
}

/// The remaining terms: the automatic settings, the lists and the join
/// pick, the tactics record's team id.
#[test]
fn the_rest_of_the_terms() {
    let version = PesVersion::Pes21;
    let tactics = fixture_tactics(version);
    let rows = named_roster();
    let roster = roster_rows(&rows);
    let term =
        |field, old, new| tactics_term(version, field, old, new, &tactics, &tactics, &roster);
    assert_eq!(
        term(TacticsField::AutoSubstitution, 0, 3),
        ("Substitution".to_owned(), "0".to_owned(), "3".to_owned())
    );
    assert_eq!(
        term(TacticsField::AutoAttackDefenceLevels, 0, 1),
        (
            "Attack/defence levels".to_owned(),
            "off".to_owned(),
            "on".to_owned()
        )
    );
    assert_eq!(
        term(TacticsField::Starting(4), 4, 7),
        (
            "Starting eleven · 5".to_owned(),
            "Player 05".to_owned(),
            "Player 08".to_owned()
        )
    );
    assert_eq!(
        term(TacticsField::Bench(0), 11, 20),
        (
            "Bench · 1".to_owned(),
            "Player 12".to_owned(),
            "Player 21".to_owned()
        )
    );
    assert_eq!(
        term(TacticsField::PlayerToJoinAttack(0), 3, 255),
        (
            "Join attack · 1".to_owned(),
            "Player".to_owned(),
            "none".to_owned()
        )
    );
    assert_eq!(
        term(TacticsField::TeamId, 701, 702),
        ("Team ID".to_owned(), "701".to_owned(), "702".to_owned())
    );
}

/// Every `field:` key names the row its own entry labels: a `SliderSpec`,
/// `StyleToggle` or `RoleSpec` whose key drifted draws its row under
/// another entry's label.
#[test]
fn every_table_entry_keys_its_own_label() {
    let version = PesVersion::Pes21;
    let tactics = fixture_tactics(version);
    let rows = named_roster();
    let roster = roster_rows(&rows);
    let label = |field| tactics_term(version, field, 0, 0, &tactics, &tactics, &roster).0;
    for spec in &SLIDERS {
        assert_eq!(
            label(TacticsField::Preset {
                preset: 0,
                field: spec.field,
            }),
            format!("Preset 1 · {}", spec.label),
            "{:?}",
            spec.field
        );
    }
    for toggle in &STYLE_TOGGLES {
        assert_eq!(
            label(TacticsField::Preset {
                preset: 0,
                field: toggle.field,
            }),
            format!("Preset 1 · {}", toggle.label),
            "{:?}",
            toggle.field
        );
    }
    for role in &assignments::SET_PIECES {
        assert_eq!(label(role.field), role.label, "{:?}", role.field);
    }
}

/// The `PlayerId` row words its target byte under the instruction in
/// force on that side, as the tab's target combo does: "An opponent"
/// under Tight Marking, "Unassigned" under a target-less one, a player
/// under a player-taking one.
#[test]
fn the_target_terms_read_the_instruction_in_force_on_each_side() {
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    let encode = |inst| instruction::encode(version, inst).expect("PES 21 stores it");
    {
        let attack = tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions");
        attack[0].instruction = encode(Instruction::TightMarking);
        attack[0].player = 4;
        attack[1].instruction = encode(Instruction::TikiTaka);
        attack[1].player = 0;
        let defence = tactics.presets[0]
            .defence_instructions
            .as_mut()
            .expect("PES 21 has instructions");
        defence[0].instruction = encode(Instruction::CounterTarget);
        defence[0].player = tactics.starting_eleven[5];
    }
    // The tab's own text for the same tactics, drawn.
    let mut window = Headless::sized(800.0, 900.0);
    let mut view = TacticsView::default();
    let rows = named_roster();
    let roster = roster_rows(&rows);
    let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
    let combo_text = |label: &str| {
        controls_right_of(&nodes, label, Role::ComboBox)[1]
            .value()
            .expect("the target combo's text")
            .to_owned()
    };
    assert_eq!(combo_text("Attack 1"), "An opponent", "the tab's own text");
    assert_eq!(combo_text("Attack 2"), "Unassigned", "the tab's own text");
    let starter_text = combo_text("Defence 1");
    assert_eq!(
        starter_text,
        format!("6 {}", name_at(&roster, tactics.starting_eleven[5])),
        "the tab's own text"
    );

    // The row's sides read the same words for the same instruction and
    // byte.
    let term = |side, index, old, new| {
        tactics_term(
            version,
            TacticsField::Instruction {
                preset: 0,
                side,
                index,
                part: InstructionPart::PlayerId,
            },
            old,
            new,
            &tactics,
            &tactics,
            &roster,
        )
    };
    assert_eq!(
        term(InstructionSide::Attack, 0, 4, 255).1,
        "An opponent",
        "the row's Tight Marking side"
    );
    assert_eq!(
        term(InstructionSide::Attack, 1, 0, 9).1,
        "Unassigned",
        "the row's target-less side"
    );
    assert_eq!(
        term(
            InstructionSide::Defence,
            0,
            u32::from(tactics.starting_eleven[5]),
            20
        )
        .1,
        starter_text,
        "the row's player-taking side"
    );

    // Each side of the row reads its own instruction: the same byte under
    // the baseline's Tight Marking and the current's target-less one.
    let baseline = tactics.clone();
    let mut current = tactics.clone();
    current.presets[0]
        .attack_instructions
        .as_mut()
        .expect("instructions")[0]
        .instruction = encode(Instruction::TikiTaka);
    let (_, old, new) = tactics_term(
        version,
        TacticsField::Instruction {
            preset: 0,
            side: InstructionSide::Attack,
            index: 0,
            part: InstructionPart::PlayerId,
        },
        4,
        4,
        &baseline,
        &current,
        &roster,
    );
    assert_eq!(
        (old.as_str(), new.as_str()),
        ("An opponent", "Unassigned"),
        "each side under its own instruction"
    );
}

// The Substitution pair's cell is as wide as its label, the DragValue
// and the item gap (`sub_cell_width`): the drawn label's left edge to
// the field's right edge measures it at the fixture's stored value.
#[test]
fn the_substitution_cell_is_as_wide_as_its_pair() {
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    assert!(
        tactics.auto.substitution.is_some(),
        "premise: PES 21 stores the byte"
    );
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);
    let mut window = Headless::sized(800.0, 900.0);
    let mut expected = 0.0f32;
    window.frame(vec![], |ui| expected = sub_cell_width(ui));

    let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
    let label = nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some("Substitution"))
        .and_then(|node| node.bounds())
        .expect("the Substitution label");
    let field = control_right_of(&nodes, "Substitution", Role::SpinButton)
        .bounds()
        .expect("the Substitution field");
    assert!(
        (field.x1 - label.x0 - f64::from(expected)).abs() < 1.0,
        "the cell is as wide as its pair (label {label:?}, field {field:?}, cell {expected})"
    );
}
