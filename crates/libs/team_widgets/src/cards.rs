//! The card pickers of `plans/save_editor.md` "`libs/team_widgets`" (`cards`
//! bullet), drafted in `docs/prototypes/save_editor.html` (`abilitiesTab`'s
//! skills grid and COM styles row, `cardBadges`, the playing-style combo).
//! A player's skills and COM playing styles are his *cards*; the ruleset's
//! tiers each say how many paid ones he may hold, so the pickers show what
//! each tier leaves him.

use pes_savefile::interchange::team_toml::labels;
use pes_savefile::model::playstyle::PlayStyle;
use pes_savefile::schema::fields::PlayerField;
use pes_savefile::schema::{playstyle, schema_for};

/// The skills grid's most columns, the draft's five to a row.
const SKILL_COLUMNS: usize = 5;

/// A skills-grid column's width in points: the five the draft lays in
/// ~675px, so a narrower pane takes fewer columns rather than clip.
const SKILL_COLUMN_WIDTH: f32 = 135.0;

/// One badge: a tier's label and the player's card economy under it (the host's
/// `aatf::Rules::card_limits` result).
pub struct TierCards<'a> {
    /// The tier's display label ("Gold", "Regular", ...).
    pub label: &'a str,
    /// The player's card economy read as this tier.
    pub limits: &'a aatf::CardLimits,
}

/// The skills grid and the COM playing styles of `skills`, with one badge per
/// tier: the cards it leaves, in the error color when overdrawn. Every card is
/// drawn on every version; one the version cannot store stays in its cell,
/// disabled with a `PES NN+` tooltip. Returns whether a card was ticked or
/// unticked this frame.
pub fn card_pickers(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    version: pes_version::PesVersion,
    skills: &mut pes_savefile::model::PlayerSkills,
    tiers: &[TierCards<'_>],
) -> bool {
    ui.push_id(id_salt, |ui| {
        let mut changed = false;
        if !tiers.is_empty() {
            ui.horizontal_wrapped(|ui| {
                ui.weak("Cards left");
                for tier in tiers {
                    let left = i32::from(tier.limits.limit) - i32::from(tier.limits.paid);
                    let mut text = egui::RichText::new(format!("{} {}", tier.label, left));
                    if left < 0 {
                        text = text.color(ui.visuals().error_fg_color);
                    }
                    ui.label(text).on_hover_text(format!(
                        "{}: {} paid of {}",
                        tier.label, tier.limits.paid, tier.limits.limit
                    ));
                }
            });
        }
        let fields = schema_for(version).player_field_set();
        // The draft's five to a row at ~675px; a pane that holds fewer
        // wraps instead of clipping the last columns.
        let per_row = ((ui.available_width() / SKILL_COLUMN_WIDTH).floor().max(1.0) as usize)
            .min(SKILL_COLUMNS);
        egui::Grid::new("skills")
            .num_columns(per_row)
            .show(ui, |ui| {
                for (index, name) in labels::SKILLS.iter().enumerate() {
                    let field = PlayerField::Skill(index as u8);
                    if fields.contains(&field) {
                        changed |= ui
                            .checkbox(&mut skills.skills[index], title_case(name))
                            .changed();
                    } else {
                        // A card the version cannot store stays, disabled, its
                        // tooltip naming the first version that has it
                        // (plans/save_editor.md "Editing UI").
                        ui.add_enabled(
                            false,
                            egui::Checkbox::new(&mut skills.skills[index], title_case(name)),
                        )
                        .on_disabled_hover_text(format!("PES {}+", first_with(field).number()));
                    }
                    if (index + 1) % per_row == 0 {
                        ui.end_row();
                    }
                }
            });
        ui.strong("COM playing styles");
        ui.horizontal_wrapped(|ui| {
            for (index, name) in labels::COM_STYLES.iter().enumerate() {
                let field = PlayerField::ComStyle(index as u8);
                if fields.contains(&field) {
                    changed |= ui
                        .checkbox(&mut skills.com_styles[index], title_case(name))
                        .changed();
                } else {
                    ui.add_enabled(
                        false,
                        egui::Checkbox::new(&mut skills.com_styles[index], title_case(name)),
                    )
                    .on_disabled_hover_text(format!("PES {}+", first_with(field).number()));
                }
            }
        });
        changed
    })
    .inner
}

/// The playing-style choice, the version's styles only. Returns whether it changed.
pub fn playing_style_picker(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    version: pes_version::PesVersion,
    style: &mut u8,
) -> bool {
    ui.push_id(id_salt, |ui| {
        // The version's styles in its stored order, not the canonical one:
        // the order the game's own Edit-mode list shows (plans/save_editor.md
        // "`libs/team_widgets`", the `cards` bullet). A stored value indexes
        // the version's list, whose holes are skipped.
        let styles: Vec<(u8, String)> = (0..=u8::MAX)
            .filter_map(|index| {
                let style = playstyle::decode(version, index).ok()?;
                Some((index, style_label(style)))
            })
            .collect();
        // A value the version cannot decode shows as its number, so the
        // combo still says what the save holds.
        let selected = playstyle::decode(version, *style)
            .map(style_label)
            .unwrap_or_else(|_| style.to_string());
        let mut changed = false;
        egui::ComboBox::from_id_salt("playing style")
            .selected_text(selected)
            .show_ui(ui, |ui| {
                for (index, label) in &styles {
                    changed |= ui.selectable_value(style, *index, label).changed();
                }
            });
        changed
    })
    .inner
}

/// The oldest version storing `field` (for the disabled cell's `PES NN+`
/// tooltip; every card is stored by some version).
fn first_with(field: PlayerField) -> pes_version::PesVersion {
    pes_savefile::schema::first_version_with(field).expect("every card is stored by some version")
}

/// `style`'s label: its Team TOML name, title-cased.
fn style_label(style: PlayStyle) -> String {
    let at = PlayStyle::ALL
        .iter()
        .position(|canonical| *canonical == style)
        .expect("`PlayStyle::ALL` lists every style");
    title_case(labels::PLAY_STYLES[at])
}

/// `name`'s label as the draft's `titleCase` writes it: underscores as spaces,
/// each word capitalized.
fn title_case(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let (first, rest) = word.split_at(1);
            format!("{}{rest}", first.to_uppercase())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use egui::accesskit::{self, Role};
    use pes_savefile::interchange::team_toml::labels;
    use pes_savefile::model::player::PlayerSkills;
    use pes_savefile::model::playstyle::PlayStyle;
    use pes_savefile::schema::fields::PlayerField;
    use pes_savefile::schema::{playstyle, schema_for};
    use pes_version::PesVersion;

    use super::*;
    use crate::test_support::{Headless, centre, painted_text_color, pointer_at};

    impl Headless {
        /// One frame of `card_pickers` over `skills` and `tiers`, returning the
        /// nodes, the painted shapes and whether the frame changed a card.
        fn pickers(
            &mut self,
            version: PesVersion,
            skills: &mut PlayerSkills,
            tiers: &[TierCards],
            events: Vec<egui::Event>,
        ) -> (Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>, bool) {
            self.pickers_at(None, version, skills, tiers, events)
        }

        /// `pickers` with `time` as the frame's timestamp (for tooltips).
        fn pickers_at(
            &mut self,
            time: Option<f64>,
            version: PesVersion,
            skills: &mut PlayerSkills,
            tiers: &[TierCards],
            events: Vec<egui::Event>,
        ) -> (Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>, bool) {
            let mut changed = None;
            let (nodes, shapes) = self.frame_at(time, events, |ui| {
                changed = Some(card_pickers(ui, "pickers", version, skills, tiers));
            });
            (nodes, shapes, changed.expect("the pickers ran"))
        }

        /// A click at `pos` over the pickers: the press frame, then the release
        /// frame whose result is returned (`clicked()` lands on the release).
        fn click_pickers(
            &mut self,
            pos: egui::Pos2,
            version: PesVersion,
            skills: &mut PlayerSkills,
            tiers: &[TierCards],
        ) -> (Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>, bool) {
            self.pickers(version, skills, tiers, pointer_at(pos, true));
            self.pickers(version, skills, tiers, pointer_at(pos, false))
        }

        /// One frame of `playing_style_picker` over `style`, returning the
        /// nodes and whether the frame changed the style.
        fn style_picker(
            &mut self,
            version: PesVersion,
            style: &mut u8,
            events: Vec<egui::Event>,
        ) -> (Vec<accesskit::Node>, bool) {
            let mut changed = None;
            let (nodes, _) = self.frame(events, |ui| {
                changed = Some(playing_style_picker(ui, "style", version, style));
            });
            (nodes, changed.expect("the picker ran"))
        }

        /// A click at `pos` over the style picker (press frame, then release).
        fn click_style(
            &mut self,
            pos: egui::Pos2,
            version: PesVersion,
            style: &mut u8,
        ) -> (Vec<accesskit::Node>, bool) {
            self.style_picker(version, style, pointer_at(pos, true));
            self.style_picker(version, style, pointer_at(pos, false))
        }
    }

    /// The widget of `role` labeled `label`.
    fn entry<'a>(nodes: &'a [accesskit::Node], role: Role, label: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| node.role() == role && node.label() == Some(label))
            .unwrap_or_else(|| panic!("no {role:?} labeled {label:?}"))
    }

    /// The `Role::Label` node carrying `text` (a Label's text lands in `value`,
    /// not `label`).
    fn text_entry<'a>(nodes: &'a [accesskit::Node], text: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some(text))
            .unwrap_or_else(|| panic!("no label {text:?}"))
    }

    /// `name`'s label as the pickers show it: underscores as spaces, each word
    /// capitalized (the draft's `titleCase`, restated so the tests check it
    /// rather than share it).
    fn title_case(name: &str) -> String {
        name.split('_')
            .map(|word| {
                let (first, rest) = word.split_at(1);
                format!("{}{rest}", first.to_uppercase())
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The `PlayerSkills` of player `id` in `version`'s fixture save.
    fn fixture_skills(version: PesVersion, id: u32) -> PlayerSkills {
        let file = pes_savefile::EditFile::from_bytes(&pes_savefile::fixture_save(version))
            .expect("fixture save");
        file.player(id)
            .unwrap_or_else(|| panic!("player {id}"))
            .skills
            .clone()
    }

    /// The stored playing-style index of player `id` in `version`'s fixture
    /// save.
    fn fixture_style(version: PesVersion, id: u32) -> u8 {
        let file = pes_savefile::EditFile::from_bytes(&pes_savefile::fixture_save(version))
            .expect("fixture save");
        file.player(id)
            .unwrap_or_else(|| panic!("player {id}"))
            .positions
            .playing_style
    }

    /// The checkbox labels a frame drew, split by the name table they come
    /// from.
    fn drawn_cards(nodes: &[accesskit::Node]) -> (Vec<String>, Vec<String>) {
        let skills: BTreeSet<String> = labels::SKILLS.iter().map(|name| title_case(name)).collect();
        let coms: BTreeSet<String> = labels::COM_STYLES
            .iter()
            .map(|name| title_case(name))
            .collect();
        let mut drawn_skills = vec![];
        let mut drawn_coms = vec![];
        for label in nodes.iter().filter_map(|node| {
            (node.role() == Role::CheckBox)
                .then(|| node.label().map(str::to_owned))
                .flatten()
        }) {
            if skills.contains(&label) {
                drawn_skills.push(label);
            } else if coms.contains(&label) {
                drawn_coms.push(label);
            } else {
                panic!("a checkbox named no card: {label:?}");
            }
        }
        (drawn_skills, drawn_coms)
    }

    /// An empty `CardLimits` but for `paid` and `limit`.
    fn limits(paid: u8, limit: i8) -> aatf::CardLimits {
        aatf::CardLimits {
            paid,
            limit,
            free: BTreeMap::new(),
            free_cards: vec![],
            required_cards: vec![],
            a_positions: 1,
            max_a_positions: None,
        }
    }

    // plans/save_editor.md "libs/team_widgets", the `cards` bullet, and
    // "Editing UI" (a version-gated widget renders disabled, never
    // disappearing): every card is drawn on every version, a card the save
    // cannot store disabled.
    #[test]
    fn the_pickers_draw_the_versions_cards() {
        /// The labels of the disabled checkboxes.
        fn disabled(nodes: &[accesskit::Node]) -> Vec<&str> {
            nodes
                .iter()
                .filter(|node| node.role() == Role::CheckBox && node.is_disabled())
                .filter_map(|node| node.label())
                .collect()
        }

        let mut window = Headless::new();
        // Player 71802 of the PES 15 fixture save; PES 15 stores 22 of the 41
        // skills and 7 COM styles (schema/pes15.rs: `malicia` at index 21 is
        // one the version does not store, so its cell is disabled).
        let mut skills = fixture_skills(PesVersion::Pes15, 71802);
        let (nodes, _, _) = window.pickers(PesVersion::Pes15, &mut skills, &[], vec![]);
        let (drawn_skills, drawn_coms) = drawn_cards(&nodes);
        assert_eq!((drawn_skills.len(), drawn_coms.len()), (41, 7));
        let off = disabled(&nodes);
        assert_eq!(off.len(), 41 - 22, "19 skills PES 15 does not store");
        for label in ["Malicia", "Heading", "Chip Shot", "Through Passing"] {
            assert!(off.contains(&label), "{label} disabled on PES 15");
        }
        for label in ["Scissors Feint", "Track Back", "Trickster", "Long Ranger"] {
            assert!(!entry(&nodes, Role::CheckBox, label).is_disabled());
        }

        // Player 70203 of the PES 17 fixture save; PES 17 stores the first 28
        // canonical skills.
        let mut skills = fixture_skills(PesVersion::Pes17, 70203);
        let (nodes, _, changed) = window.pickers(PesVersion::Pes17, &mut skills, &[], vec![]);
        assert!(!changed);
        let (drawn_skills, drawn_coms) = drawn_cards(&nodes);
        assert_eq!((drawn_skills.len(), drawn_coms.len()), (41, 7));
        let off = disabled(&nodes);
        assert_eq!(off.len(), 41 - 28, "13 skills PES 17 does not store");
        for label in ["Scissors Feint", "Track Back", "Trickster", "Long Ranger"] {
            assert!(!entry(&nodes, Role::CheckBox, label).is_disabled());
        }

        // Player 78102 of the PES 21 fixture save; PES 21 stores all 41.
        let mut skills = fixture_skills(PesVersion::Pes21, 78102);
        let (nodes, _, _) = window.pickers(PesVersion::Pes21, &mut skills, &[], vec![]);
        let (drawn_skills, drawn_coms) = drawn_cards(&nodes);
        assert_eq!((drawn_skills.len(), drawn_coms.len()), (41, 7));
        assert!(disabled(&nodes).is_empty(), "PES 21 stores every card");
    }

    // plans/save_editor.md "libs/team_widgets", the `cards` bullet: a checkbox
    // click is one change (the host's undo takes one step per click).
    #[test]
    fn a_tick_is_one_change_on_that_frame_only() {
        let mut window = Headless::new();
        // Player 70203 of the PES 17 fixture save ("SMUG ANIME FACE").
        let mut skills = fixture_skills(PesVersion::Pes17, 70203);
        // The first skill PES 17 stores that he does not hold.
        let fields = schema_for(PesVersion::Pes17).player_field_set();
        let index = (0..labels::SKILLS.len())
            .find(|&i| fields.contains(&PlayerField::Skill(i as u8)) && !skills.skills[i])
            .expect("an unticked PES 17 skill");
        let label = title_case(labels::SKILLS[index]);
        let (nodes, _, _) = window.pickers(PesVersion::Pes17, &mut skills, &[], vec![]);
        let checkbox = entry(&nodes, Role::CheckBox, &label);

        let (_, _, changed) =
            window.click_pickers(centre(checkbox), PesVersion::Pes17, &mut skills, &[]);
        assert!(changed, "the tick is a change");
        assert!(skills.skills[index], "the flag is set");
        let (_, _, changed) = window.pickers(PesVersion::Pes17, &mut skills, &[], vec![]);
        assert!(!changed, "the next frame reports nothing new");

        let (_, _, changed) =
            window.click_pickers(centre(checkbox), PesVersion::Pes17, &mut skills, &[]);
        assert!(changed, "the untick is a change");
        assert!(!skills.skills[index], "the flag is cleared");
    }

    // plans/save_editor.md "libs/team_widgets", the `cards` bullet; the draft's
    // `abilitiesTab` draws the skills five to a row.
    #[test]
    fn the_skills_grid_wraps_at_five() {
        let mut window = Headless::new();
        // Player 78102 of the PES 21 fixture save.
        let mut skills = fixture_skills(PesVersion::Pes21, 78102);
        let (nodes, _, _) = window.pickers(PesVersion::Pes21, &mut skills, &[], vec![]);
        // The skill checkboxes in draw order, left to right, top to bottom.
        let mut cells: Vec<&accesskit::Node> = nodes
            .iter()
            .filter(|node| node.role() == Role::CheckBox)
            .collect();
        cells.sort_by(|a, b| {
            let key = |node: &&accesskit::Node| {
                let bounds = node.bounds().expect("drawn bounds");
                (bounds.y0, bounds.x0)
            };
            key(a).partial_cmp(&key(b)).expect("ordered bounds")
        });
        assert_eq!(cells[0].label(), Some("Scissors Feint"));
        let y0 = |node: &accesskit::Node| node.bounds().expect("bounds").y0;
        let x0 = |node: &accesskit::Node| node.bounds().expect("bounds").x0;
        // Cells 0 to 4 sit on row one, cell 5 starts row two: written as five
        // on purpose, so a wrong `SKILL_COLUMNS` fails here, not follows it.
        for cell in &cells[1..5] {
            assert_eq!(y0(cell), y0(cells[0]), "the first five sit on one row");
        }
        assert!(y0(cells[5]) > y0(cells[0]), "the sixth starts the next row");
        assert_eq!(x0(cells[5]), x0(cells[0]), "the row starts at the same x");
    }

    // The same bullet, narrower: a pane that holds fewer than the draft's
    // five columns wraps instead of clipping (~460 points, three columns);
    // the COM styles get their own heading and wrap like the grid.
    #[test]
    fn the_pickers_wrap_at_the_panes_width() {
        let mut window = Headless::sized(480.0, 700.0);
        // Player 78102 of the PES 21 fixture save.
        let mut skills = fixture_skills(PesVersion::Pes21, 78102);
        let lim = limits(0, 8);
        let tiers = [TierCards {
            label: "Gold",
            limits: &lim,
        }];
        let (nodes, _, _) = window.pickers(PesVersion::Pes21, &mut skills, &tiers, vec![]);
        // The badges' row draws under its label.
        text_entry(&nodes, "Cards left");
        let mut cells: Vec<&accesskit::Node> = nodes
            .iter()
            .filter(|node| node.role() == Role::CheckBox)
            .collect();
        cells.sort_by(|a, b| {
            let key = |node: &&accesskit::Node| {
                let bounds = node.bounds().expect("drawn bounds");
                (bounds.y0, bounds.x0)
            };
            key(a).partial_cmp(&key(b)).expect("ordered bounds")
        });
        let y0 = |node: &accesskit::Node| node.bounds().expect("bounds").y0;
        // Three to a row here: the fourth sits below the first, the grid
        // having wrapped rather than run off the pane.
        for cell in &cells[1..3] {
            assert!(
                (y0(cell) - y0(cells[0])).abs() < 1.0,
                "the first three sit on one row"
            );
        }
        assert!(y0(cells[3]) > y0(cells[0]), "the fourth wraps");
        // The COM row under its heading wraps too: the last style sits
        // below the first.
        text_entry(&nodes, "COM playing styles");
        assert!(
            y0(entry(&nodes, Role::CheckBox, "Long Ranger"))
                > y0(entry(&nodes, Role::CheckBox, "Trickster")),
            "the COM styles wrap"
        );
    }

    // plans/save_editor.md "Editing UI": a card the version cannot store stays
    // in its cell, disabled (the layout stays put across versions), the tooltip
    // naming the first version that has it (`PES NN+`, the draft's wording).
    #[test]
    fn a_card_the_version_lacks_keeps_its_cell_disabled() {
        let mut window = Headless::new();
        // "Through Passing" is the last skill (index 40), PES 20 and 21 only.
        let mut skills15 = fixture_skills(PesVersion::Pes15, 71802);
        let mut skills21 = fixture_skills(PesVersion::Pes21, 78102);
        let (nodes15, _, _) = window.pickers(PesVersion::Pes15, &mut skills15, &[], vec![]);
        let (nodes21, _, _) = window.pickers(PesVersion::Pes21, &mut skills21, &[], vec![]);
        let at = |nodes: &[accesskit::Node], label: &str| {
            let bounds = entry(nodes, Role::CheckBox, label)
                .bounds()
                .expect("bounds");
            (bounds.x0, bounds.y0)
        };
        assert_eq!(
            at(&nodes15, "Through Passing"),
            at(&nodes21, "Through Passing"),
            "the same card sits in the same cell on every version"
        );
        // Malicia (index 21, mid-grid): the assert that proves the cell, not
        // only the row's left edge.
        assert_eq!(
            at(&nodes15, "Malicia"),
            at(&nodes21, "Malicia"),
            "a mid-grid card sits in the same cell on every version"
        );
        let cell = entry(&nodes15, Role::CheckBox, "Through Passing");
        assert!(cell.is_disabled(), "PES 20+, disabled on PES 15");

        // A click on the disabled cell changes nothing and reports no change.
        let held = skills15.skills[40];
        let (_, _, changed) =
            window.click_pickers(centre(cell), PesVersion::Pes15, &mut skills15, &[]);
        assert!(!changed);
        assert_eq!(skills15.skills[40], held);
    }

    // plans/save_editor.md "Editing UI": the disabled cell's hover text names
    // the first version that stores the card (the draft's `PES NN+`).
    #[test]
    fn the_disabled_cell_names_the_first_version() {
        let mut window = Headless::new();
        // Player 71802 of the PES 15 fixture save; `malicia` first stores in
        // PES 16.
        let mut skills = fixture_skills(PesVersion::Pes15, 71802);
        let (nodes, _, _) = window.pickers(PesVersion::Pes15, &mut skills, &[], vec![]);
        let cell = entry(&nodes, Role::CheckBox, "Malicia");
        let pos = centre(cell);
        // The pointer rests on the cell while the clock passes the tooltip's
        // delay.
        window.pickers_at(
            Some(0.0),
            PesVersion::Pes15,
            &mut skills,
            &[],
            vec![egui::Event::PointerMoved(pos)],
        );
        let mut shown = false;
        for frame in 1..20 {
            let (nodes, _, _) = window.pickers_at(
                Some(f64::from(frame) * 0.5),
                PesVersion::Pes15,
                &mut skills,
                &[],
                vec![],
            );
            if nodes
                .iter()
                .any(|node| node.role() == Role::Label && node.value() == Some("PES 16+"))
            {
                shown = true;
                break;
            }
        }
        assert!(shown, "the tooltip names the first version, PES 16+");
    }

    // plans/save_editor.md "libs/team_widgets", the `cards` bullet: a COM
    // style's tick is a change like a skill's.
    #[test]
    fn a_com_style_tick_is_one_change_on_that_frame_only() {
        let mut window = Headless::new();
        // Player 78102 of the PES 21 fixture save ("THE YOINKY SPLOINKY") holds
        // no COM style.
        let mut skills = fixture_skills(PesVersion::Pes21, 78102);
        assert!(!skills.com_styles[0], "premise: no Trickster");
        let (nodes, _, _) = window.pickers(PesVersion::Pes21, &mut skills, &[], vec![]);
        let checkbox = entry(&nodes, Role::CheckBox, "Trickster");

        let (_, _, changed) =
            window.click_pickers(centre(checkbox), PesVersion::Pes21, &mut skills, &[]);
        assert!(changed, "the tick is a change");
        assert!(skills.com_styles[0], "the flag is set");
        let (_, _, changed) = window.pickers(PesVersion::Pes21, &mut skills, &[], vec![]);
        assert!(!changed, "the next frame reports nothing new");

        let (_, _, changed) =
            window.click_pickers(centre(checkbox), PesVersion::Pes21, &mut skills, &[]);
        assert!(changed, "the untick is a change");
        assert!(!skills.com_styles[0], "the flag is cleared");
    }

    // plans/save_editor.md "libs/team_widgets", the `cards` bullet: "Cards
    // left" then one badge per tier, `limit - paid`, the error colour when
    // overdrawn. A colour is no AccessKit property, so it is read back from
    // the frame's painted shapes: the badge's `TextShape` carries the
    // `RichText` colour in its galley's sections.
    #[test]
    fn the_badges_read_limit_minus_paid_coloured_when_overdrawn() {
        let mut window = Headless::new();
        let under = limits(3, 5);
        let over = limits(7, 4);
        let spent = limits(4, 4);
        let tiers = [
            TierCards {
                label: "Gold",
                limits: &under,
            },
            TierCards {
                label: "Silver",
                limits: &over,
            },
            TierCards {
                label: "Bronze",
                limits: &spent,
            },
        ];
        // Player 70203 of the PES 17 fixture save; the badges only display the
        // `CardLimits` values.
        let mut skills = fixture_skills(PesVersion::Pes17, 70203);
        let (nodes, shapes, _) = window.pickers(PesVersion::Pes17, &mut skills, &tiers, vec![]);
        text_entry(&nodes, "Cards left");
        text_entry(&nodes, "Gold 2");
        text_entry(&nodes, "Silver -3");
        let text_color = window.ctx.style_of(window.ctx.theme()).visuals.text_color();
        let error_color = window
            .ctx
            .style_of(window.ctx.theme())
            .visuals
            .error_fg_color;
        assert_eq!(painted_text_color(&shapes, "Gold 2"), text_color);
        assert_eq!(painted_text_color(&shapes, "Silver -3"), error_color);
        // No card left is not overdrawn.
        assert_eq!(painted_text_color(&shapes, "Bronze 0"), text_color);
    }

    // plans/save_editor.md "libs/team_widgets", the `cards` bullet: the
    // playing-style choice lists the version's styles only.
    #[test]
    fn the_style_picker_lists_the_versions_styles() {
        let mut window = Headless::new();
        // Player 70203 of the PES 17 fixture save; stored playing style 8.
        let mut style = fixture_style(PesVersion::Pes17, 70203);
        let (nodes, changed) = window.style_picker(PesVersion::Pes17, &mut style, vec![]);
        assert!(!changed, "an idle frame is no change");
        let combo = nodes
            .iter()
            .find(|node| node.role() == Role::ComboBox)
            .expect("the combo");
        let (_, changed) = window.click_style(centre(combo), PesVersion::Pes17, &mut style);
        assert!(!changed, "opening the combo is no change");
        let (nodes, _) = window.style_picker(PesVersion::Pes17, &mut style, vec![]);
        // `nodes` come back in `IdMap` order, not layout order: sort the open
        // items top to bottom by their bounds.
        let mut items: Vec<&accesskit::Node> = nodes
            .iter()
            .filter(|node| node.role() == Role::Button)
            .collect();
        items.sort_by(|a, b| {
            let (ay, by) = (a.bounds().map(|r| r.y0), b.bounds().map(|r| r.y0));
            ay.partial_cmp(&by).expect("drawn bounds")
        });
        let open_labels: Vec<String> = items
            .iter()
            .filter_map(|node| node.label().map(str::to_owned))
            .collect();
        let expected: Vec<String> = (0..=u8::MAX)
            .filter_map(|index| playstyle::decode(PesVersion::Pes17, index).ok())
            .map(|style| {
                let at = PlayStyle::ALL
                    .iter()
                    .position(|canonical| *canonical == style)
                    .expect("a canonical style");
                title_case(labels::PLAY_STYLES[at])
            })
            .collect();
        assert_eq!(open_labels, expected, "PES 17's eighteen styles");
        assert!(
            !open_labels.contains(&"Roaming Flank".to_string()),
            "a PES 19+ style is not listed"
        );

        // The current style picked again: no change, the value unmoved.
        let item = entry(&nodes, Role::Button, "Anchor Man");
        let (_, changed) = window.click_style(centre(item), PesVersion::Pes17, &mut style);
        assert!(!changed, "the current style picked again is no change");
        assert_eq!(style, 8);

        let (_, changed) = window.click_style(centre(combo), PesVersion::Pes17, &mut style);
        assert!(!changed, "reopening the combo is no change");
        let (nodes, _) = window.style_picker(PesVersion::Pes17, &mut style, vec![]);
        let item = entry(&nodes, Role::Button, "Goal Poacher");
        let wanted = playstyle::encode(PesVersion::Pes17, PlayStyle::GoalPoacher)
            .expect("PES 17 has Goal Poacher");
        assert_ne!(style, wanted, "premise: the player is no Goal Poacher");
        let (_, changed) = window.click_style(centre(item), PesVersion::Pes17, &mut style);
        assert!(changed, "the pick is a change");
        assert_eq!(style, wanted);
    }
}
