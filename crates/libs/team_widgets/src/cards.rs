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

/// One badge: a tier's label and the player's card economy under it (the host's
/// `aatf::Rules::card_limits` result).
pub struct TierCards<'a> {
    /// The tier's display label ("Gold", "Regular", ...).
    pub label: &'a str,
    /// The player's card economy read as this tier.
    pub limits: &'a aatf::CardLimits,
}

/// The skills grid and the COM playing styles of `skills`, as the version stores them,
/// with one badge per tier: the cards it leaves, in the error color when overdrawn.
/// Returns whether a card was ticked or unticked this frame.
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
            ui.horizontal(|ui| {
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
        egui::Grid::new("skills").num_columns(5).show(ui, |ui| {
            for (index, name) in labels::SKILLS.iter().enumerate() {
                if fields.contains(&PlayerField::Skill(index as u8)) {
                    changed |= ui
                        .checkbox(&mut skills.skills[index], title_case(name))
                        .changed();
                }
            }
        });
        ui.horizontal(|ui| {
            for (index, name) in labels::COM_STYLES.iter().enumerate() {
                if fields.contains(&PlayerField::ComStyle(index as u8)) {
                    changed |= ui
                        .checkbox(&mut skills.com_styles[index], title_case(name))
                        .changed();
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
            let mut changed = None;
            let (nodes, shapes) = self.frame(events, |ui| {
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

    // plans/save_editor.md "libs/team_widgets", the `cards` bullet: the pickers
    // are version-gated; a card the save cannot store is not drawn (the draft
    // greys it, egui drops it).
    #[test]
    fn the_pickers_draw_the_versions_cards() {
        let mut window = Headless::new();
        // Player 71802 of the PES 15 fixture save; PES 15 stores 22 skills
        // with holes in the canonical order (schema/pes15.rs: `malicia` at
        // index 21 is one the version does not store).
        let mut skills = fixture_skills(PesVersion::Pes15, 71802);
        let (nodes, _, _) = window.pickers(PesVersion::Pes15, &mut skills, &[], vec![]);
        let (drawn_skills, drawn_coms) = drawn_cards(&nodes);
        assert_eq!(
            (drawn_skills.len(), drawn_coms.len()),
            (22, 7),
            "PES 15: 22 skills, 7 COM styles"
        );
        assert!(
            nodes.iter().all(|node| node.label() != Some("Malicia")),
            "a skill PES 15 does not store is not drawn"
        );

        // Player 70203 of the PES 17 fixture save; PES 17 stores the first 28
        // canonical skills.
        let mut skills = fixture_skills(PesVersion::Pes17, 70203);
        let (nodes, _, changed) = window.pickers(PesVersion::Pes17, &mut skills, &[], vec![]);
        assert!(!changed);
        let (drawn_skills, drawn_coms) = drawn_cards(&nodes);
        assert_eq!(
            (drawn_skills.len(), drawn_coms.len()),
            (28, 7),
            "PES 17: 28 skills, 7 COM styles"
        );
        for label in ["Scissors Feint", "Track Back", "Trickster", "Long Ranger"] {
            entry(&nodes, Role::CheckBox, label);
        }
        assert!(
            nodes
                .iter()
                .all(|node| node.label() != Some("Chip Shot")
                    && node.label() != Some("Through Passing")),
            "PES 19+ skills are not drawn on PES 17"
        );

        // Player 78102 of the PES 21 fixture save; PES 21 stores all 41.
        let mut skills = fixture_skills(PesVersion::Pes21, 78102);
        let (nodes, _, _) = window.pickers(PesVersion::Pes21, &mut skills, &[], vec![]);
        let (drawn_skills, drawn_coms) = drawn_cards(&nodes);
        assert_eq!(
            (drawn_skills.len(), drawn_coms.len()),
            (41, 7),
            "PES 21: 41 skills, 7 COM styles"
        );
        entry(&nodes, Role::CheckBox, "Through Passing");
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
        let (nodes, _) = window.style_picker(PesVersion::Pes17, &mut style, vec![]);
        let combo = nodes
            .iter()
            .find(|node| node.role() == Role::ComboBox)
            .expect("the combo");
        let (_, _) = window.click_style(centre(combo), PesVersion::Pes17, &mut style);
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
