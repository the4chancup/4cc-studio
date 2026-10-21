//! The master list (`plans/save_editor.md` "Editing UI", the Player list bullet): the
//! team selector, the cross-team filter box and one row per rostered player, the
//! name drawn in the colors its savefile codes set.

use pes_savefile::EditFile;
use pes_savefile::interchange::team_toml::labels::POSITIONS;
use pes_savefile::model::names::{display_name, name_spans};
use pes_savefile::model::{PlayerEntry, TeamEntry};

use crate::session::Session;
use crate::session::selection::{self, Row, TeamChoice};
use crate::view::SaveView;

/// The selector, the filter box and the list, top to bottom of the side panel.
/// `view`'s fields are borrowed apart: the session feeds the rows while the
/// selector, the filter box and the rows' clicks write the rest.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let SaveView {
        session,
        team,
        filter,
        selected,
        refusal: _,
        rules: _,
        cards: _,
    } = view;
    let Some(session) = session.as_ref() else {
        return;
    };
    let file = session.file();
    team_selector(ui, team, file);
    ui.add(egui::TextEdit::singleline(filter).hint_text("Filter all teams (name, shirt, id)"));
    // A filter in use lists every team, so each row names its team — as it
    // does under `ALL` (the draft's `tshort`).
    let show_team = !filter.trim().is_empty() || *team == TeamChoice::All;
    let rows = selection::rows(file, team, filter);
    // Only the visible rows are laid out: `show_rows` draws the range the
    // viewport shows, each row's click target at the text row's height plus
    // the button padding (`interact_size.y`, what a selectable occupies).
    let row_height = ui.spacing().interact_size.y;
    egui::ScrollArea::vertical().show_rows(ui, row_height, rows.len(), |ui, range| {
        for row in &rows[range] {
            player_row(ui, selected, session, row, show_team);
        }
    });
}

/// The team selector: `ALL`, then each team by name in the save's order.
fn team_selector(ui: &mut egui::Ui, choice: &mut TeamChoice, file: &EditFile) {
    let mut chosen = *choice;
    egui::ComboBox::from_id_salt("save_editor_team")
        .selected_text(match chosen {
            TeamChoice::All => "ALL".to_owned(),
            TeamChoice::Team(id) => file
                .team(id)
                .map(|team| team.name.clone())
                .unwrap_or_else(|| id.to_string()),
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut chosen, TeamChoice::All, "ALL");
            for team in file.teams() {
                ui.selectable_value(&mut chosen, TeamChoice::Team(team.id), &team.name);
            }
        });
    *choice = chosen;
}

/// One list row: number, position, the team's name while the list crosses
/// teams (`ALL` or a filter in use), and the name with its colour codes drawn.
/// The whole row is one click target; a click selects it.
fn player_row(
    ui: &mut egui::Ui,
    selected: &mut Option<(u32, u8)>,
    session: &Session,
    row: &Row,
    show_team: bool,
) {
    let file = session.file();
    let team = file.team(row.team_id);
    let player = file.player(row.player_id);
    let dirty = session.player_dirty(row.team_id, row.slot);
    let is_selected = *selected == Some((row.team_id, row.slot));
    let mut job = row_text(team, player, row, show_team);
    if dirty {
        job.append(" \u{25cf}", 0.0, plain_format());
    }
    // One line, truncated when the name is too long for the panel: the full
    // display name rides the hover, a dirty row's with the dot's text.
    let hover = player.map_or_else(|| "?".to_owned(), |player| display_name(&player.name));
    let response = ui.add(egui::Button::selectable(is_selected, job).truncate());
    let response = if dirty {
        response.on_hover_text(format!("{hover}\nUnsaved changes"))
    } else {
        response.on_hover_text(hover)
    };
    if response.clicked() {
        *selected = Some((row.team_id, row.slot));
    }
}

/// The format of a plain run (no code, after a reset or a non-hex code):
/// `PLACEHOLDER`, which the widget replaces with its own text color —
/// `TextFormat::default()`'s gray is not it.
fn plain_format() -> egui::TextFormat {
    egui::TextFormat::simple(egui::FontId::default(), egui::Color32::PLACEHOLDER)
}

/// The row's text as one `LayoutJob`: number, position, team name while the
/// list crosses teams, then the name's runs, each colored as `name_spans`
/// gives it (`None` runs the normal text color).
fn row_text(
    team: Option<&TeamEntry>,
    player: Option<&PlayerEntry>,
    row: &Row,
    show_team: bool,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let number = team
        .and_then(|team| team.roster.get(usize::from(row.slot)))
        .map_or_else(|| "?".to_owned(), |slot| slot.number.to_string());
    let position = player
        .and_then(|player| POSITIONS.get(usize::from(player.positions.registered)))
        .copied()
        .unwrap_or("?");
    job.append(&format!("{number} {position}"), 0.0, plain_format());
    if show_team && let Some(name) = team.map(|team| team.name.as_str()) {
        job.append(&format!(" {name}"), 0.0, plain_format());
    }
    match player {
        Some(player) => {
            job.append(" ", 0.0, plain_format());
            for (color, text) in name_spans(&player.name) {
                let format = match color {
                    Some([red, green, blue]) => egui::TextFormat::simple(
                        egui::FontId::default(),
                        egui::Color32::from_rgb(red, green, blue),
                    ),
                    None => plain_format(),
                };
                job.append(text, 0.0, format);
            }
        }
        None => job.append(" ?", 0.0, plain_format()),
    }
    job
}
