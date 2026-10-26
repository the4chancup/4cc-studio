//! The master list (`plans/save_editor.md` "Editing UI", the Player list bullet): the
//! team selector, the cross-team filter box, one team's row heading its players'
//! rows, and the quick actions under the list. A player row shows the name drawn
//! in the colors its savefile codes set; a team row the team's id, its colours
//! 1-2 as swatches and its name.

use pes_savefile::EditFile;
use pes_savefile::interchange::team_toml::labels::POSITIONS;
use pes_savefile::model::names::display_name;
use pes_savefile::model::{PlayerEntry, TeamEntry};

use team_widgets::names::{append_name, plain_format};

use crate::session::Session;
use crate::session::selection::{self, ListRow, Row, TeamChoice};
use crate::view::team::channel_to_show;
use crate::view::{SaveView, Selection, quick_actions};

/// A swatch square's side (the draft's 10px).
const SWATCH: f32 = 10.0;

/// The selector, the filter box, the rows and the quick actions, top to
/// bottom of the side panel: the list's scroll area takes the height left
/// above the actions.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some(session) = view.session.as_ref() else {
        return;
    };
    team_selector(ui, &mut view.team, session.file(), "save_editor_team");
    ui.add(
        egui::TextEdit::singleline(&mut view.filter)
            .hint_text("Filter every team (name, shirt name or id)"),
    );
    // The quick actions sit at the panel's bottom, under the list (the
    // draft's `qagrid`): a bottom panel takes its height off the scroll's.
    egui::Panel::bottom("save_editor_quick_actions").show(ui, |ui| {
        quick_actions::show(ui, view);
    });
    // A filter in use lists every team; the rows also name their team under
    // `ALL` (the draft's `tshort`).
    let show_team = !view.filter.trim().is_empty() || view.team == TeamChoice::All;
    // A drop works on a row of the dragged player's own team, in one team's
    // list or under ALL where each team's rows run unbroken under its team
    // row; drag is off while a filter is typed, where neighbouring rows are
    // not one team's neighbouring slots.
    let draggable = view.filter.trim().is_empty();
    let session = view.session.as_mut().expect("checked above");
    let rows = selection::list(session.file(), &view.team, &view.filter);
    // Only the visible rows are laid out: `show_rows` draws the range the
    // viewport shows, each row's click target at the text row's height plus
    // the button padding (`interact_size.y`, what a selectable occupies).
    let row_height = ui.spacing().interact_size.y;
    egui::ScrollArea::vertical().show_rows(ui, row_height, rows.len(), |ui, range| {
        for row in &rows[range] {
            match row {
                ListRow::Team(team_id) => {
                    team_row(ui, &mut view.selected, session, *team_id, row_height);
                }
                ListRow::Player(row) => {
                    player_row(ui, &mut view.selected, session, row, show_team, draggable);
                }
            }
        }
    });
}

/// The team selector: `ALL`, then each team by name in the save's order.
/// `id_salt` is the combo's: two combos with one salt share their popup
/// state, so the comparator's filter passes its own (the player list's
/// stays `save_editor_team`).
pub(super) fn team_selector(
    ui: &mut egui::Ui,
    choice: &mut TeamChoice,
    file: &EditFile,
    id_salt: &str,
) {
    let mut chosen = *choice;
    egui::ComboBox::from_id_salt(id_salt)
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

/// A team's own row (the draft's `trow`): one click target the row's width
/// — the team's id in the weak colour, its colours 1 and 2 as small filled
/// squares (the shown colour, `channel_to_show`), the name with its colour
/// codes in the strong text colour, and a `●` while the team's own entry
/// has unsaved changes (`Session::team_dirty`). A click selects the team
/// (the detail shows the team view); it is never a drag source nor a drop
/// target.
fn team_row(
    ui: &mut egui::Ui,
    selected: &mut Option<Selection>,
    session: &Session,
    team_id: u32,
    row_height: f32,
) {
    let Some(team) = session.file().team(team_id) else {
        return;
    };
    let is_selected = *selected == Some(Selection::Team(team_id));
    let dirty = session.team_dirty(team_id);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::click(),
    );
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::Role::Button,
            ui.is_enabled(),
            is_selected,
            format!("{team_id} {}", display_name(&team.name)),
        )
    });
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, is_selected);
        if is_selected || response.hovered() || response.has_focus() {
            ui.painter().rect(
                rect.expand(visuals.expansion),
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            );
        }
        let padding = ui.spacing().button_padding.x;
        let centre = rect.center().y;
        let mut x = rect.min.x + padding;
        let weak = ui.visuals().weak_text_color();
        let id_galley =
            ui.painter()
                .layout_no_wrap(team_id.to_string(), egui::FontId::default(), weak);
        ui.painter().galley(
            egui::pos2(x, centre - id_galley.size().y / 2.0),
            id_galley.clone(),
            weak,
        );
        x += id_galley.size().x + ui.spacing().item_spacing.x;
        if let Some(colors) = &team.colors {
            for color in colors {
                let rgb = [
                    channel_to_show(color.red),
                    channel_to_show(color.green),
                    channel_to_show(color.blue),
                ];
                let swatch = egui::Rect::from_center_size(
                    egui::pos2(x + SWATCH / 2.0, centre),
                    egui::vec2(SWATCH, SWATCH),
                );
                ui.painter().rect_filled(
                    swatch,
                    egui::CornerRadius::same(1),
                    egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
                );
                x += SWATCH + 3.0;
            }
        }
        x += ui.spacing().item_spacing.x;
        let mut job = egui::text::LayoutJob::default();
        append_name(&mut job, &team.name);
        if dirty {
            job.append(" \u{25cf}", 0.0, plain_format());
        }
        job.wrap = egui::text::TextWrapping::truncate_at_width((rect.max.x - x - padding).max(0.0));
        let name_galley = ui.painter().layout_job(job);
        // The name's plain runs take the strong text colour, or the
        // selection's while the row is selected.
        let text = if is_selected {
            visuals.fg_stroke.color
        } else {
            ui.visuals().strong_text_color()
        };
        ui.painter().galley(
            egui::pos2(x, centre - name_galley.size().y / 2.0),
            name_galley,
            text,
        );
    }
    let hover = display_name(&team.name);
    let response = if dirty {
        response.on_hover_text(format!("{hover}\nUnsaved changes"))
    } else {
        response.on_hover_text(hover)
    };
    if response.clicked() {
        *selected = Some(Selection::Team(team_id));
    }
}

/// One list row: number, position, the team's name while the list crosses
/// teams (`ALL` or a filter in use), and the name with its colour codes drawn.
/// The whole row is one click target; a click selects it. `draggable` (no
/// filter typed) also makes it a drag source and a drop target: a drop on
/// a row of the dragged player's own team moves him to it
/// (`Session::move_player`, SE-GUI-08), a drop on another team's row does
/// nothing.
fn player_row(
    ui: &mut egui::Ui,
    selected: &mut Option<Selection>,
    session: &mut Session,
    row: &Row,
    show_team: bool,
    draggable: bool,
) {
    let file = session.file();
    let team = file.team(row.team_id);
    let player = file.player(row.player_id);
    let dirty = session.player_dirty(row.team_id, row.slot);
    let is_selected = *selected == Some(Selection::Player(row.team_id, row.slot));
    let mut job = row_text(team, player, row, show_team);
    if dirty {
        job.append(" \u{25cf}", 0.0, plain_format());
    }
    // One line, truncated when the name is too long for the panel: the full
    // display name rides the hover, a dirty row's with the dot's text.
    let hover = player.map_or_else(|| "?".to_owned(), |player| display_name(&player.name));
    let sense = if draggable {
        egui::Sense::click_and_drag()
    } else {
        egui::Sense::click()
    };
    let response = ui.add(
        egui::Button::selectable(is_selected, job)
            .truncate()
            .sense(sense),
    );
    let response = if dirty {
        response.on_hover_text(format!("{hover}\nUnsaved changes"))
    } else {
        response.on_hover_text(hover)
    };
    if response.clicked() {
        *selected = Some(Selection::Player(row.team_id, row.slot));
    }
    if draggable {
        response.dnd_set_drag_payload(*row);
        if let Some(dropped) = response.dnd_release_payload::<Row>()
            && dropped.team_id == row.team_id
        {
            session.move_player(row.team_id, dropped.slot, row.slot);
            // The selection follows the dragged record to its new slot.
            *selected = Some(Selection::Player(row.team_id, row.slot));
        }
    }
}

/// The row's text as one `LayoutJob`: number, position, team name while the
/// list crosses teams, then the name's runs, each colored as `name_spans`
/// gives it (`None` runs the normal text color, `append_name`).
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
            append_name(&mut job, &player.name);
        }
        None => job.append(" ?", 0.0, plain_format()),
    }
    job
}
