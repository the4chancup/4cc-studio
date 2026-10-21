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
    /// The display name, colour codes included, drawn as plain text.
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
            Some(player) => format!("Player {}  {} {}", player.id, player.number, player.name),
            None => format!("Slot {slot}"),
        },
    };
    ui.weak(text);
}

/// One row: the rule's name (monospace) and `sentence`, the severity's color.
/// A player's row is clickable, returning `SelectedPlayer` on click.
fn row(ui: &mut egui::Ui, team_id: u32, violation: &aatf::Violation) -> Option<SelectedPlayer> {
    let (rule, _) = violation.kind.rule();
    let visuals = ui.visuals();
    let color = match violation.severity {
        aatf::Severity::Error => visuals.error_fg_color,
        aatf::Severity::Warning => visuals.warn_fg_color,
        aatf::Severity::Suggestion => visuals.text_color(),
    };
    let mut job = egui::text::LayoutJob::default();
    job.append(
        rule,
        0.0,
        egui::TextFormat::simple(egui::TextStyle::Monospace.resolve(ui.style()), color),
    );
    job.append(
        &format!(" {}", sentence(violation)),
        0.0,
        egui::TextFormat::simple(egui::TextStyle::Body.resolve(ui.style()), color),
    );
    match violation.slot {
        Some(slot) => {
            let response = ui
                .selectable_label(false, job)
                .on_hover_text("Select this player");
            response
                .clicked()
                .then_some(SelectedPlayer { team_id, slot })
        }
        None => {
            ui.label(job).on_hover_text("A team-level violation");
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
            } else {
                format!("{stat} {value}, not the target {target}")
            }
        }
        K::StatTotal { total, limit } => {
            format!("stats added up {total}, over the limit {limit}")
        }
        K::Form { value, target } => format!("form {value}, not the target {target}"),
        K::InjuryResistance { value, max } => {
            format!("injury resistance {value} over the maximum {max}")
        }
        K::WeakFootUsage { value, max } => {
            format!("weak foot usage {value} over the maximum {max}")
        }
        K::WeakFootAccuracy { value, max } => {
            format!("weak foot accuracy {value} over the maximum {max}")
        }
        K::NameColor { color, expected } => format!(
            "name colour {}, the tier wants {}",
            color.as_deref().unwrap_or("none"),
            expected.as_deref().unwrap_or("none")
        ),
        K::Cards { paid, limit } => format!("{paid} paid cards over the limit of {limit}"),
        K::MinCards { paid, min } => format!("{paid} paid cards under the minimum of {min}"),
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
    use crate::test_support::{Headless, centre, painted_text_color, pointer_at};

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

    // SE-GUI-15
    /// The official ruleset's check on team 734 ("/lit/") of the PES 17
    /// fixture save reports 17 violations: a `Height` error on slots 0, 1, 3,
    /// 5 and 19, then the team's own (three `TierCount` and a
    /// `BracketQuota`), and eight `Unused` suggestions on slots 9, 10, 14 and
    /// 22. The list groups them in the check's slot order, `the team` last.
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
        assert_eq!(painted_text_color(&shapes, stat_total), error_color);
        assert_eq!(painted_text_color(&shapes, height), error_color);
        assert_eq!(painted_text_color(&shapes, stat), warn_color);
    }

    // SE-GUI-15
    /// Clicking a player's row selects him; a team's row is not clickable.
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

    // SE-GUI-15
    /// Suggestions draw only when asked, in their own group after the
    /// violations: team 734's eight `Unused` rows on slots 9, 10, 14 and 22.
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
        let (nodes, _, _) = window.list(&teams, true, vec![]);
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
            "speed 80, not the target 92"
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
}
