//! The AATF results list of `plans/save_editor.md` "`libs/team_widgets`"
//! (`violations` bullet), drafted in `docs/prototypes/save_editor.html`
//! (`aatfPanel`, `violationSentence`, `playerHead`): one team's check grouped
//! per player, a row's click reporting the player to select. The AATF panel,
//! the Live AATF dock and the Team creator all draw this list.

/// A rostered player as the list's group header shows him.
pub struct ListedPlayer<'a> {
    /// The roster slot.
    pub slot: u8,
    /// The player id.
    pub id: u32,
    /// The shirt number, as wide as `pes_savefile`'s `RosterSlot::number` (PES 19 on stores
    /// numbers past 99).
    pub number: u16,
    /// The name as the save stores it: the host passes the raw name (colour
    /// codes included), the header draws it code-free.
    pub name: &'a str,
}

/// One team's check: its players by slot and `aatf::Rules::check_team`'s violations, in the
/// order the check returned them.
pub struct TeamViolations<'a> {
    /// The team id.
    pub team_id: u32,
    /// The team name.
    pub team_name: &'a str,
    /// The rostered players, for the group headers.
    pub players: &'a [ListedPlayer<'a>],
    /// The violations `check_team` returned, in its order.
    pub violations: &'a [aatf::Violation],
}

/// The player a row's click selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectedPlayer {
    /// The team the player is in.
    pub team_id: u32,
    /// His roster slot.
    pub slot: u8,
}

/// The AATF results of `teams`, grouped per player; suggestions only when
/// `show_suggestions`. Returns the player whose row was clicked this frame.
pub fn violations_list(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    teams: &[TeamViolations<'_>],
    show_suggestions: bool,
) -> Option<SelectedPlayer> {
    ui.push_id(id_salt, |ui| {
        let mut selected = None;
        for team in teams {
            let slots = slot_order(
                team.violations
                    .iter()
                    .filter(|v| v.severity != aatf::Severity::Suggestion),
            );
            // A team the check found nothing wrong with gets no header of its own.
            if slots.is_empty() {
                continue;
            }
            ui.strong(format!("Team {} {}", team.team_id, team.team_name));
            for slot in slots {
                group_header(ui, team, slot);
                for severity in [aatf::Severity::Error, aatf::Severity::Warning] {
                    for violation in team
                        .violations
                        .iter()
                        .filter(|v| v.slot == slot && v.severity == severity)
                    {
                        if let Some(player) = row(ui, team.team_id, violation) {
                            selected = Some(player);
                        }
                    }
                }
            }
        }
        let any_suggestions = teams
            .iter()
            .flat_map(|team| team.violations)
            .any(|v| v.severity == aatf::Severity::Suggestion);
        if show_suggestions && any_suggestions {
            ui.strong("Suggestions");
            for team in teams {
                for slot in slot_order(
                    team.violations
                        .iter()
                        .filter(|v| v.severity == aatf::Severity::Suggestion),
                ) {
                    group_header(ui, team, slot);
                    for violation in team
                        .violations
                        .iter()
                        .filter(|v| v.slot == slot && v.severity == aatf::Severity::Suggestion)
                    {
                        if let Some(player) = row(ui, team.team_id, violation) {
                            selected = Some(player);
                        }
                    }
                }
            }
        }
        selected
    })
    .inner
}

/// The slots the given violations fall on, in first-appearance order (the
/// check's: players by slot, the team's own rows last).
fn slot_order<'a>(violations: impl Iterator<Item = &'a aatf::Violation>) -> Vec<Option<u8>> {
    let mut order = Vec::new();
    for violation in violations {
        if !order.contains(&violation.slot) {
            order.push(violation.slot);
        }
    }
    order
}

/// One group's header: `Player {id}  {number} {name}` for a rostered player,
/// `Slot {slot}` for one the host lists no player for, `the team` for the
/// team's own rows.
fn group_header(ui: &mut egui::Ui, team: &TeamViolations<'_>, slot: Option<u8>) {
    let text = match slot {
        None => "the team".to_string(),
        Some(slot) => match team.players.iter().find(|p| p.slot == slot) {
            Some(player) => format!(
                "Player {}  {} {}",
                player.id,
                player.number,
                pes_savefile::model::names::display_name(player.name)
            ),
            None => format!("Slot {slot}"),
        },
    };
    ui.weak(text);
}

/// The rule name's column, the draft's `.f`: 190 points of proportional text,
/// cut with an ellipsis when longer, never wrapping.
const RULE_COL: f32 = 190.0;

/// `text` laid out in `font`/`color`, wrapped as `wrap` says.
fn layout(
    ui: &egui::Ui,
    text: String,
    font: egui::FontId,
    color: egui::Color32,
    wrap: egui::text::TextWrapping,
) -> std::sync::Arc<egui::Galley> {
    let mut job =
        egui::text::LayoutJob::single_section(text, egui::TextFormat::simple(font, color));
    job.wrap = wrap;
    ui.fonts_mut(|f| f.layout_job(job))
}

/// One row, the draft's `.drow`: the rule's name in `RULE_COL` points of
/// severity-colored proportional text, one line cut with an ellipsis (the
/// draft's `text-overflow: ellipsis`), then `sentence` in the strong text
/// color, wrapped so a long custom check's text stays readable; the whole row
/// one click target. A player's row
/// returns `SelectedPlayer` on click; a team row only hovers.
fn row(ui: &mut egui::Ui, team_id: u32, violation: &aatf::Violation) -> Option<SelectedPlayer> {
    let (rule, _) = violation.kind.rule();
    let (rule_color, strong_color) = {
        let visuals = ui.visuals();
        (
            match violation.severity {
                aatf::Severity::Error => visuals.error_fg_color,
                aatf::Severity::Warning => visuals.warn_fg_color,
                aatf::Severity::Suggestion => visuals.text_color(),
            },
            visuals.strong_text_color(),
        )
    };
    let font = egui::TextStyle::Body.resolve(ui.style());
    let rule_galley = layout(
        ui,
        rule.to_string(),
        font.clone(),
        rule_color,
        egui::text::TextWrapping::truncate_at_width(RULE_COL),
    );
    let text = sentence(violation);
    let sentence_galley = layout(
        ui,
        text.clone(),
        font,
        strong_color,
        egui::text::TextWrapping::wrap_at_width((ui.available_width() - RULE_COL).max(0.0)),
    );
    let height = rule_galley.size().y.max(sentence_galley.size().y);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        if violation.slot.is_some() {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    if ui.is_rect_visible(rect) {
        if response.hovered() {
            ui.painter()
                .rect_filled(rect, 2.0, ui.style().visuals.widgets.hovered.bg_fill);
        }
        ui.painter()
            .add(egui::Shape::galley(rect.min, rule_galley, rule_color));
        ui.painter().add(egui::Shape::galley(
            rect.min + egui::vec2(RULE_COL, 0.0),
            sentence_galley,
            strong_color,
        ));
    }
    let label = format!("{rule} {text}");
    match violation.slot {
        Some(slot) => {
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label.as_str())
            });
            response
                .on_hover_text("Select this player")
                .clicked()
                .then_some(SelectedPlayer { team_id, slot })
        }
        None => {
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), label.as_str())
            });
            response.on_hover_text("A team-level violation");
            None
        }
    }
}

/// What is wrong with `violation`, in the draft's `violationSentence` wording:
/// one arm per `ViolationKind`.
fn sentence(violation: &aatf::Violation) -> String {
    use aatf::ViolationKind as K;
    match &violation.kind {
        K::RegisteredNotA => "the registered position is not rated A".to_string(),
        K::FieldedNotA { preset, position } => format!(
            "fielded at {} in preset {preset} without an A rating",
            position.label()
        ),
        K::BRating => "a B rating where the ruleset refuses one".to_string(),
        K::GkSecondA => "a second A rating at GK".to_string(),
        K::Age { age, min, max } => format!("age {age} outside {min} to {max}"),
        K::Weight { weight, min, max } => format!("weight {weight} outside {min} to {max}"),
        K::PositionRange => "a registered position past the version's positions".to_string(),
        K::PlayingStyleRange => "a playing style the version does not allow".to_string(),
        K::NoTier => "fits no tier".to_string(),
        K::Stat {
            stat,
            value,
            target,
            at_most,
        } => {
            let stat = stat.label();
            if violation.severity == aatf::Severity::Warning {
                format!("{stat} {value} over the target {target}, inside the warning band")
            } else if *at_most {
                format!("{stat} {value} over the maximum {target}")
            } else if value < target {
                format!("{stat} {value}, under the {target} the rule wants")
            } else {
                format!("{stat} {value}, not the target {target}")
            }
        }
        K::StatTotal { total, limit } => {
            format!("stats added up {total}, over the limit {limit}")
        }
        K::Form { value, target } => format!("form {value}, not the target {target}"),
        K::InjuryResistance { value, max } => {
            if value > max {
                format!("injury resistance {value} over the maximum {max}")
            } else {
                format!("injury resistance {value}, under the {max} the rule wants")
            }
        }
        K::WeakFootUsage { value, max } => {
            if value > max {
                format!("weak foot usage {value} over the maximum {max}")
            } else {
                format!("weak foot usage {value}, under the {max} the rule wants")
            }
        }
        K::WeakFootAccuracy { value, max } => {
            if value > max {
                format!("weak foot accuracy {value} over the maximum {max}")
            } else {
                format!("weak foot accuracy {value}, under the {max} the rule wants")
            }
        }
        K::NameColor { color, expected } => match expected.as_deref() {
            Some(expected) => format!(
                "name colour {}, the tier wants {}",
                color.as_deref().unwrap_or("none"),
                expected
            ),
            // A colour another tier reserved on a player outside it.
            None => format!(
                "name colour {} is reserved for a tier",
                color.as_deref().unwrap_or("none")
            ),
        },
        K::Cards { paid, limit } => format!("{paid} paid cards over the limit of {limit}"),
        K::MinCards { paid, min } => {
            format!("{paid} paid skill cards and extra positions under the minimum of {min}")
        }
        K::SkillCardCap { count, max } => {
            format!("{count} skill cards over the cap of {max}")
        }
        K::ComStyleCap { count, max } => format!("{count} COM styles over the cap of {max}"),
        K::RequiredCard { card } => format!("the required card {} is not held", card.label()),
        K::APositions { count, max } => {
            format!("{count} A positions over the allowance of {max}")
        }
        K::Height { height } => format!("height {height} takes no bracket"),
        K::TierCount {
            tier,
            count,
            expected,
        } => format!("{count} {tier} players, expected {expected}"),
        K::BracketQuota {
            system,
            bracket,
            count,
            quota,
        } => format!("{count} players in bracket {bracket} of {system}, quota {quota}"),
        K::MedalSystem { systems } => format!(
            "the tier counts match no medal system [{}]",
            systems.join(", ")
        ),
        K::ExtraHeight { used, total } => {
            format!("{used} cm of extra height over the pool of {total}")
        }
        K::Captain => "no captain".to_string(),
        K::Goalkeepers { count, min } => {
            format!("{count} goalkeepers, the minimum is {min}")
        }
        K::TeamSetting { label, setting } => {
            format!("team setting {label}: {setting} does not hold")
        }
        K::FormationShape {
            preset,
            formation,
            shape,
        } => {
            format!("formation {formation} of preset {preset} matches a forbidden shape {shape:?}")
        }
        K::Special { label, text } => format!("{label}: {text}"),
        K::Custom { text } => text.clone(),
        K::Unused {
            allowance,
            used,
            allowed,
        } => format!("only {used} of {allowed} {allowance} used"),
        K::UnusedChoice { special } => format!("no option of {special} taken"),
    }
}

#[cfg(test)]
mod tests {
    use egui::accesskit::{self, Role};
    use pes_version::PesVersion;

    use super::*;
    use crate::test_support::{Headless, centre, painted_text_color, pointer_at, text_shape};

    impl Headless {
        /// One frame of `violations_list` over `teams`, returning the nodes,
        /// the painted shapes and the selected player.
        fn list(
            &mut self,
            teams: &[TeamViolations],
            show_suggestions: bool,
            events: Vec<egui::Event>,
        ) -> (
            Vec<accesskit::Node>,
            Vec<egui::epaint::ClippedShape>,
            Option<SelectedPlayer>,
        ) {
            let mut selected = None;
            let (nodes, shapes) = self.frame(events, |ui| {
                selected = Some(violations_list(ui, "list", teams, show_suggestions));
            });
            (nodes, shapes, selected.expect("the list ran"))
        }

        /// A click at `pos` over the list: the press frame, then the release
        /// frame whose result is returned (`clicked()` lands on the release).
        fn click_list(
            &mut self,
            pos: egui::Pos2,
            teams: &[TeamViolations],
            show_suggestions: bool,
        ) -> (
            Vec<accesskit::Node>,
            Vec<egui::epaint::ClippedShape>,
            Option<SelectedPlayer>,
        ) {
            self.list(teams, show_suggestions, pointer_at(pos, true));
            self.list(teams, show_suggestions, pointer_at(pos, false))
        }
    }

    /// Every header and row the frame drew, in top-to-bottom order: a Label's
    /// `value` or a Button's `label` (`nodes` come in `IdMap` order, not
    /// layout order).
    fn drawn_lines(nodes: &[accesskit::Node]) -> Vec<String> {
        let mut lines: Vec<(f64, f64, String)> = Vec::new();
        for node in nodes {
            let text = match node.role() {
                Role::Label => node.value().map(str::to_owned),
                Role::Button => node.label().map(str::to_owned),
                _ => None,
            };
            if let (Some(text), Some(bounds)) = (text, node.bounds()) {
                lines.push((bounds.y0, bounds.x0, text));
            }
        }
        lines.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .expect("drawn bounds")
                .then(a.1.partial_cmp(&b.1).expect("drawn bounds"))
        });
        lines.into_iter().map(|(_, _, text)| text).collect()
    }

    /// A `Violation` on `slot` of `severity` and `kind`.
    fn violation(
        slot: Option<u8>,
        severity: aatf::Severity,
        kind: aatf::ViolationKind,
    ) -> aatf::Violation {
        aatf::Violation {
            slot,
            severity,
            kind,
        }
    }

    /// The stat `name` names.
    fn stat(name: &str) -> aatf::Stat {
        aatf::Stat::try_from(name.to_string()).expect("a stat name")
    }

    /// The position `name` names.
    fn position(name: &str) -> aatf::Position {
        aatf::Position::try_from(name.to_string()).expect("a position name")
    }

    /// A `ListedPlayer` (the roster slot, id, number and name).
    fn listed(slot: u8, id: u32, number: u16, name: &str) -> ListedPlayer<'_> {
        ListedPlayer {
            slot,
            id,
            number,
            name,
        }
    }

    /// The PES 17 fixture save.
    fn fixture() -> pes_savefile::EditFile {
        pes_savefile::EditFile::from_bytes(&pes_savefile::fixture_save(PesVersion::Pes17))
            .expect("fixture save")
    }

    /// Team `id`'s `ListedPlayer`s in `file` (the rostered ones, in slot
    /// order).
    fn listed_players<'a>(file: &'a pes_savefile::EditFile, id: u32) -> Vec<ListedPlayer<'a>> {
        let team = file.team(id).unwrap_or_else(|| panic!("team {id}"));
        team.roster
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.player_id != 0)
            .map(|(i, slot)| ListedPlayer {
                slot: u8::try_from(i).expect("a slot fits u8"),
                id: slot.player_id,
                number: slot.number,
                name: file
                    .player(slot.player_id)
                    .expect("a rostered player")
                    .name
                    .as_str(),
            })
            .collect()
    }

    /// Team `id`'s check under the official ruleset on `file`.
    fn check(file: &pes_savefile::EditFile, id: u32) -> Vec<aatf::Violation> {
        let view = aatf::TeamView::new(file, id).expect("the team views");
        aatf::Rules::official()
            .check_team(&view)
            .expect("the check runs")
    }

    /// The official ruleset's check on team 734 ("/lit/") of the PES 17
    /// fixture save reports 17 violations: a `Height` error on slots 0, 1, 3,
    /// 5 and 19, then the team's own (three `TierCount` and a
    /// `BracketQuota`), and eight `Unused` suggestions on slots 9, 10, 14 and
    /// 22. The list groups them in the check's slot order, `the team` last.
    // SE-GUI-15
    #[test]
    fn the_list_groups_the_checks_violations_per_player() {
        let file = fixture();
        let players = listed_players(&file, 734);
        let violations = check(&file, 734);
        assert_eq!(violations.len(), 17, "the check's count");
        let team = file.team(734).expect("team 734");
        let teams = [TeamViolations {
            team_id: 734,
            team_name: &team.name,
            players: &players,
            violations: &violations,
        }];
        let mut window = Headless::new();
        let (nodes, _, selected) = window.list(&teams, false, vec![]);
        assert_eq!(selected, None);
        assert_eq!(
            drawn_lines(&nodes),
            [
                "Team 734 /lit/",
                "Player 73401  49 Muh Spooks",
                "Height height 185 takes no bracket",
                "Player 73402  55 knee-chee",
                "Height height 185 takes no bracket",
                "Player 73404  45 Yuki-kun ^~^",
                "Height height 185 takes no bracket",
                "Player 73406  85 Huge Orson Welles",
                "Height height 185 takes no bracket",
                "Player 73420  82 Tolstoyevsky's War and Punishment",
                "Height height 185 takes no bracket",
                "the team",
                "TierCount 2 gold players, expected 1",
                "TierCount 0 bronze players, expected 2",
                "TierCount 19 regular players, expected 18",
                "BracketQuota 5 players in bracket tall of red, quota 10",
            ]
        );
    }

    // plans/save_editor.md "libs/team_widgets", `violations` bullet: a group's
    // errors come before its warnings, the check's order kept inside each.
    #[test]
    fn a_groups_errors_come_before_its_warnings() {
        let violations = [
            violation(
                Some(5),
                aatf::Severity::Warning,
                aatf::ViolationKind::Stat {
                    stat: stat("speed"),
                    value: 80,
                    target: 92,
                    at_most: false,
                },
            ),
            violation(
                Some(5),
                aatf::Severity::Error,
                aatf::ViolationKind::StatTotal {
                    total: 1772,
                    limit: 1771,
                },
            ),
            violation(
                Some(5),
                aatf::Severity::Error,
                aatf::ViolationKind::Height { height: 185 },
            ),
        ];
        let players = [listed(5, 1, 9, "X")];
        let teams = [TeamViolations {
            team_id: 1,
            team_name: "T",
            players: &players,
            violations: &violations,
        }];
        let mut window = Headless::new();
        let (nodes, shapes, _) = window.list(&teams, false, vec![]);
        let error_color = window
            .ctx
            .style_of(window.ctx.theme())
            .visuals
            .error_fg_color;
        let warn_color = window
            .ctx
            .style_of(window.ctx.theme())
            .visuals
            .warn_fg_color;
        let stat_total = "StatTotal stats added up 1772, over the limit 1771";
        let height = "Height height 185 takes no bracket";
        let stat = "Stat speed 80 over the target 92, inside the warning band";
        assert_eq!(
            drawn_lines(&nodes),
            ["Team 1 T", "Player 1  9 X", stat_total, height, stat]
        );
        let strong_color = window
            .ctx
            .style_of(window.ctx.theme())
            .visuals
            .strong_text_color();
        // The rule name carries the severity's color, the sentence always the
        // strong text color (the draft's `.f`/`.v`).
        for (rule_name, color) in [
            ("StatTotal", error_color),
            ("Height", error_color),
            ("Stat", warn_color),
        ] {
            assert_eq!(painted_text_color(&shapes, rule_name), color, "{rule_name}");
        }
        for sentence in [
            "stats added up 1772, over the limit 1771",
            "height 185 takes no bracket",
            "speed 80 over the target 92, inside the warning band",
        ] {
            assert_eq!(
                painted_text_color(&shapes, sentence),
                strong_color,
                "{sentence}"
            );
        }
    }

    /// Clicking a player's row selects him; a team's row is not clickable.
    // SE-GUI-15
    #[test]
    fn a_player_rows_click_selects_him_on_that_frame_only() {
        let violations = [
            violation(
                Some(3),
                aatf::Severity::Error,
                aatf::ViolationKind::Height { height: 185 },
            ),
            violation(None, aatf::Severity::Error, aatf::ViolationKind::Captain),
        ];
        let players = [listed(3, 70103, 3, "NAME")];
        let teams = [TeamViolations {
            team_id: 701,
            team_name: "/3/",
            players: &players,
            violations: &violations,
        }];
        let mut window = Headless::new();
        let (nodes, _, _) = window.list(&teams, false, vec![]);
        let player_row = nodes
            .iter()
            .find(|node| {
                node.role() == Role::Button
                    && node.label() == Some("Height height 185 takes no bracket")
            })
            .expect("the player's row");
        let (_, _, selected) = window.click_list(centre(player_row), &teams, false);
        assert_eq!(
            selected,
            Some(SelectedPlayer {
                team_id: 701,
                slot: 3
            })
        );
        let (_, _, selected) = window.list(&teams, false, vec![]);
        assert_eq!(selected, None, "the next frame reports nothing new");

        let team_row = nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some("Captain no captain"))
            .expect("the team's row");
        let (_, _, selected) = window.click_list(centre(team_row), &teams, false);
        assert_eq!(selected, None, "a team row is not clickable");
    }

    /// Suggestions draw only when asked, in their own group after the
    /// violations: team 734's eight `Unused` rows on slots 9, 10, 14 and 22.
    // SE-GUI-15
    #[test]
    fn suggestions_draw_on_request_in_their_own_group() {
        let file = fixture();
        let players = listed_players(&file, 734);
        let violations = check(&file, 734);
        let team = file.team(734).expect("team 734");
        let teams = [TeamViolations {
            team_id: 734,
            team_name: &team.name,
            players: &players,
            violations: &violations,
        }];
        let mut window = Headless::new();
        let (nodes, _, _) = window.list(&teams, false, vec![]);
        assert!(
            !drawn_lines(&nodes)
                .iter()
                .any(|line| line == "Suggestions" || line.starts_with("Unused")),
            "no suggestion drawn when off"
        );
        let (nodes, shapes, _) = window.list(&teams, true, vec![]);
        let lines = drawn_lines(&nodes);
        let at = lines
            .iter()
            .position(|line| line == "Suggestions")
            .expect("the Suggestions header");
        assert_eq!(
            lines[at..],
            [
                "Suggestions",
                "Player 73410  46 DFW",
                "Unused only 4 of 6 cards used",
                "Unused only 2 of 4 weak_foot used",
                "Player 73411  1 Start With the Greeks",
                "Unused only 5 of 6 cards used",
                "Unused only 2 of 4 weak_foot used",
                "Player 73415  87 A.M. Pinecone",
                "Unused only 4 of 5 cards used",
                "Unused only 2 of 4 weak_foot used",
                "Player 73423  2 my diary desu",
                "Unused only 4 of 5 cards used",
                "Unused only 2 of 4 weak_foot used",
            ]
        );
        assert_eq!(
            painted_text_color(&shapes, "Unused"),
            window.ctx.style_of(window.ctx.theme()).visuals.text_color(),
            "a suggestion's rule name is the normal text color"
        );
    }

    // plans/save_editor.md "libs/team_widgets", `violations` bullet: the row's
    // sentence is `violationSentence`'s wording (`Stat`'s warning wording is
    // its `sev === "Warning"` branch, an absent `NameColor` value `none`).
    #[test]
    fn the_row_sentences_word_the_kinds() {
        use aatf::Severity::{Error, Warning};
        use aatf::ViolationKind as K;
        let player = |severity, kind| violation(Some(1), severity, kind);
        assert_eq!(
            sentence(&player(
                Error,
                K::FieldedNotA {
                    preset: 1,
                    position: position("CB")
                }
            )),
            "fielded at CB in preset 1 without an A rating"
        );
        assert_eq!(
            sentence(&player(
                Error,
                K::Stat {
                    stat: stat("speed"),
                    value: 80,
                    target: 92,
                    at_most: false
                }
            )),
            "speed 80, under the 92 the rule wants"
        );
        assert_eq!(
            sentence(&player(
                Error,
                K::Stat {
                    stat: stat("speed"),
                    value: 95,
                    target: 92,
                    at_most: false
                }
            )),
            "speed 95, not the target 92"
        );
        assert_eq!(
            sentence(&player(
                Error,
                K::Stat {
                    stat: stat("speed"),
                    value: 80,
                    target: 92,
                    at_most: true
                }
            )),
            "speed 80 over the maximum 92"
        );
        assert_eq!(
            sentence(&player(
                Warning,
                K::Stat {
                    stat: stat("speed"),
                    value: 80,
                    target: 92,
                    at_most: false
                }
            )),
            "speed 80 over the target 92, inside the warning band"
        );
        assert_eq!(
            sentence(&player(
                Error,
                K::NameColor {
                    color: None,
                    expected: Some("cc9900ff".to_string())
                }
            )),
            "name colour none, the tier wants cc9900ff"
        );
        assert_eq!(
            sentence(&player(Error, K::Cards { paid: 10, limit: 8 })),
            "10 paid cards over the limit of 8"
        );
        assert_eq!(
            sentence(&player(
                Error,
                K::TierCount {
                    tier: "gold".to_string(),
                    count: 0,
                    expected: 1
                }
            )),
            "0 gold players, expected 1"
        );

        // A value under its bound says so: VTL11's weak foot is an exact
        // target (`aatf`'s `weak_foot_below_the_exact_target`, slot 14), 3
        // against the required 4.
        for (kind, expected) in [
            (
                K::WeakFootUsage { value: 3, max: 4 },
                "weak foot usage 3, under the 4 the rule wants",
            ),
            (
                K::WeakFootAccuracy { value: 3, max: 4 },
                "weak foot accuracy 3, under the 4 the rule wants",
            ),
            (
                K::InjuryResistance { value: 1, max: 3 },
                "injury resistance 1, under the 3 the rule wants",
            ),
            (
                K::WeakFootUsage { value: 4, max: 3 },
                "weak foot usage 4 over the maximum 3",
            ),
            (
                K::MinCards { paid: 2, min: 5 },
                "2 paid skill cards and extra positions under the minimum of 5",
            ),
            (
                K::NameColor {
                    color: Some("cc9900ff".to_string()),
                    expected: None,
                },
                "name colour cc9900ff is reserved for a tier",
            ),
        ] {
            assert_eq!(sentence(&player(Error, kind.clone())), expected, "{kind:?}");
        }
    }

    // plans/save_editor.md "libs/team_widgets", `violations` bullet: a team the
    // check found nothing wrong with gets no header of its own.
    #[test]
    fn a_team_with_no_violations_draws_no_header() {
        let empty = TeamViolations {
            team_id: 1,
            team_name: "EMPTY",
            players: &[],
            violations: &[],
        };
        let violations = [violation(
            Some(2),
            aatf::Severity::Error,
            aatf::ViolationKind::Height { height: 185 },
        )];
        let players = [listed(2, 70202, 2, "GUILTY")];
        let guilty = TeamViolations {
            team_id: 702,
            team_name: "/a/",
            players: &players,
            violations: &violations,
        };
        let teams = [empty, guilty];
        let mut window = Headless::new();
        let (nodes, _, _) = window.list(&teams, false, vec![]);
        assert_eq!(
            drawn_lines(&nodes),
            [
                "Team 702 /a/",
                "Player 70202  2 GUILTY",
                "Height height 185 takes no bracket",
            ],
            "the empty team's header is not drawn"
        );
    }

    // plans/save_editor.md "libs/team_widgets", `violations` bullet: the group
    // header draws the name code-free (a cup save's medal players carry colour
    // codes).
    #[test]
    fn a_coded_name_draws_as_plain_text() {
        // Player 71801 of the PES 17 fixture save: his name carries two colour
        // codes.
        let file = fixture();
        let raw = file.player(71801).expect("player 71801").name.as_str();
        assert!(
            pes_savefile::model::names::name_color(raw).is_some(),
            "premise: the fixture name is coded"
        );
        let violations = [violation(
            Some(0),
            aatf::Severity::Error,
            aatf::ViolationKind::Height { height: 185 },
        )];
        let players = [listed(0, 71801, 50, raw)];
        let teams = [TeamViolations {
            team_id: 718,
            team_name: "/f/",
            players: &players,
            violations: &violations,
        }];
        let mut window = Headless::new();
        let (nodes, _, _) = window.list(&teams, false, vec![]);
        let lines = drawn_lines(&nodes);
        assert_eq!(
            lines,
            [
                "Team 718 /f/",
                "Player 71801  50 ROADTRIP",
                "Height height 185 takes no bracket",
            ]
        );
        assert!(
            lines.iter().all(|line| !line.contains('\u{11}')),
            "no colour code byte is drawn"
        );
    }

    // plans/save_editor.md "libs/team_widgets", `violations` bullet, the draft's
    // `.drow`: a sentence longer than the row wraps under its own column, so a
    // long check's text stays readable rather than cut off.
    #[test]
    fn a_long_sentence_wraps_rather_than_being_cut() {
        let violations = [
            violation(
                Some(3),
                aatf::Severity::Error,
                aatf::ViolationKind::Height { height: 185 },
            ),
            violation(
                Some(3),
                aatf::Severity::Error,
                aatf::ViolationKind::MedalSystem {
                    systems: [
                        "ggss regulars",
                        "ggs regulars",
                        "gggs regulars",
                        "gssss regulars",
                        "ggss giants",
                        "ggs giants",
                        "gggs giants",
                        "gssss giants",
                        "ggss manlets",
                        "ggs manlets",
                        "gggs manlets",
                        "gssss manlets",
                    ]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                },
            ),
        ];
        let players = [listed(3, 70103, 3, "NAME")];
        let teams = [TeamViolations {
            team_id: 701,
            team_name: "/3/",
            players: &players,
            violations: &violations,
        }];
        let mut window = Headless::new();
        let (nodes, shapes, _) = window.list(&teams, false, vec![]);
        let bounds_of = |text: &str| {
            nodes
                .iter()
                .find(|node| node.role() == Role::Button && node.label() == Some(text))
                .unwrap_or_else(|| panic!("no row {text:?}"))
                .bounds()
                .expect("bounds")
        };
        let height_of = |text: &str| {
            let bounds = bounds_of(text);
            bounds.y1 - bounds.y0
        };
        let one_line = height_of("Height height 185 takes no bracket");
        let longest_label = "MedalSystem the tier counts match no medal system [ggss regulars, ggs regulars, gggs regulars, gssss regulars, ggss giants, ggs giants, gggs giants, gssss giants, ggss manlets, ggs manlets, gggs manlets, gssss manlets]";
        let longest = height_of(longest_label);
        assert!(
            longest >= 2.0 * one_line,
            "the long sentence wraps onto more rows: {longest} vs {one_line}"
        );

        // The sentence sits in its own column, inside the row: its origin is
        // the row's left edge plus `RULE_COL`, its right edge inside the row.
        let medal_bounds = bounds_of(longest_label);
        let rule_shape = text_shape(&shapes, "MedalSystem");
        let sentence_shape = text_shape(&shapes, "the tier counts match");
        assert!(
            (f64::from(rule_shape.pos.x) - medal_bounds.x0).abs() <= 0.5,
            "the rule name starts at the row's left edge"
        );
        assert!(
            (f64::from(sentence_shape.pos.x) - (medal_bounds.x0 + f64::from(RULE_COL))).abs()
                <= 0.5,
            "the sentence starts at its column"
        );
        assert!(
            f64::from(sentence_shape.pos.x + sentence_shape.galley.rect.width())
                <= medal_bounds.x1 + 0.5,
            "the sentence does not run past the row's right edge"
        );
    }

    // plans/save_editor.md "libs/team_widgets", `violations` bullet: the
    // `Suggestions` header needs a suggestion row to head, not only
    // `show_suggestions`.
    #[test]
    fn no_suggestions_header_without_a_suggestion() {
        let violations = [violation(
            Some(2),
            aatf::Severity::Error,
            aatf::ViolationKind::Height { height: 185 },
        )];
        let players = [listed(2, 70202, 2, "NAME")];
        let teams = [TeamViolations {
            team_id: 702,
            team_name: "/a/",
            players: &players,
            violations: &violations,
        }];
        let mut window = Headless::new();
        let (nodes, _, _) = window.list(&teams, true, vec![]);
        assert!(
            !drawn_lines(&nodes).iter().any(|line| line == "Suggestions"),
            "a team with violations but no suggestion draws no Suggestions header"
        );
    }
}
