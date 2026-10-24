//! The settings menu (core plan, "Settings menu"; draft `docs/prototypes/shell.html`):
//! an `egui::Modal` the sidebar's gear opens, as does a notice's
//! `ActionEffect::OpenSettings`. It holds the common grid every tool shares — one row
//! per `CommonSettings` field a member edits, in the draft's order and wording — then
//! one `CollapsingHeader` per registered tool that has settings, its body the tool's
//! `settings_view`. A tool has settings when its `default_settings()` is not empty;
//! `last_tool`, `last_update_check` and `skipped_version`, the state the shell writes
//! itself, have no row.
//!
//! There is no Save or Apply: a widget's change writes the setting under the settings
//! lock and asks the shell to save at the end of the frame. A text row commits when its
//! edit loses focus or Enter is pressed, not per keystroke: a half-typed path would
//! otherwise reach every tool reading it.

use std::path::PathBuf;
use std::thread::available_parallelism;

use pes_version::PesVersion;

use crate::settings::{CommonSettings, Theme};
use crate::tool::{ShellRequest, StudioTool, ToolContext};
use crate::widgets::{PathKind, path_row, row_label};

use super::sidebar::{Sidebar, exe_tooltip, version_blocker, version_label};

/// The modal's width, the draft's `.settings-body`.
const WIDTH: f32 = 560.0;

/// The menu's state across frames: whether it is open, the tool the next drawn frame
/// expands, and the two path rows' edit buffers.
#[derive(Default)]
pub(super) struct SettingsMenu {
    /// The modal is drawn while set.
    open: bool,
    /// The index of the tool whose section the next drawn frame expands, every other
    /// collapsed: the open set is fixed at each opening, then the member's toggles hold
    /// while the menu stays open (ST-SET-01, the draft's `openSettings`).
    pending: Option<usize>,
    /// `pes_folder_path` while its row is edited.
    pes_folder: String,
    /// `exports_folder_path` while its row is edited.
    exports_folder: String,
}

impl SettingsMenu {
    /// Opens the menu on `tools[active]`'s section when it has one, the buffers filled
    /// from the settings.
    pub(super) fn open(&mut self, active: usize, ctx: &ToolContext) {
        self.open = true;
        self.pending = Some(active);
        let common = ctx.common();
        self.pes_folder = common.pes_folder_path;
        self.exports_folder = common.exports_folder_path.display().to_string();
    }

    /// Closes the menu without a click: a held window close makes the holder's prompt the
    /// view, and the menu's modal must not cover it (gui.md "Cancellation").
    pub(super) fn close(&mut self) {
        self.open = false;
    }

    /// Draws the modal while open: the common grid, then one `CollapsingHeader` per tool
    /// with settings in sidebar order. `sidebar` is the PES version row's: a pick goes
    /// through the selector's one function, and the row's tooltips are the selector's
    /// blocker reason and exe check.
    pub(super) fn show(
        &mut self,
        egui_ctx: &egui::Context,
        tools: &mut [Box<dyn StudioTool>],
        ctx: &ToolContext,
        sidebar: &mut Sidebar,
    ) {
        if !self.open {
            return;
        }
        let modal = egui::Modal::new(egui::Id::unique("settings-menu")).show(egui_ctx, |ui| {
            ui.set_min_width(WIDTH);
            ui.heading("Settings");
            self.common_grid(ui, tools, ctx, sidebar);
            let pending = self.pending.take();
            for (index, tool) in tools.iter_mut().enumerate() {
                if tool.default_settings().is_empty() {
                    continue;
                }
                let mut header =
                    egui::CollapsingHeader::new(tool.label()).id_salt(("settings", tool.id()));
                // Each opening expands the active tool's section and collapses the rest;
                // without a pending one the stored state stands, so the member's toggles
                // hold while the menu stays open.
                if let Some(active) = pending {
                    header = header.open(Some(index == active));
                }
                header.show(ui, |ui| tool.settings_view(ui, ctx));
            }
            ui.button("Close").clicked()
        });
        if modal.inner || modal.should_close() {
            self.open = false;
        }
    }

    /// The common grid, always shown (no header): one row per field a member edits. A
    /// widget's change writes the common setting under the lock and asks the shell to
    /// save, as every settings change does.
    fn common_grid(
        &mut self,
        ui: &mut egui::Ui,
        tools: &[Box<dyn StudioTool>],
        ctx: &ToolContext,
        sidebar: &mut Sidebar,
    ) {
        let mut common = ctx.common();
        let mut changed = false;
        egui::Grid::new("settings-common")
            .num_columns(2)
            .show(ui, |ui| {
                version_row(ui, tools, &common, ctx, sidebar);
                if path_row(
                    ui,
                    "PES folder",
                    "pes_folder_path",
                    "The PES installation folder, with ** standing for the two-digit \
                     version (20**).",
                    &mut self.pes_folder,
                    &mut common.pes_folder_path,
                    PathKind::Folder,
                ) {
                    changed = true;
                }
                let mut exports = common.exports_folder_path.display().to_string();
                if path_row(
                    ui,
                    "Exports folder",
                    "exports_folder_path",
                    "Where exports are picked up.",
                    &mut self.exports_folder,
                    &mut exports,
                    PathKind::Folder,
                ) {
                    common.exports_folder_path = PathBuf::from(exports);
                    changed = true;
                }
                let threads = row_label(
                    ui,
                    "Threads",
                    "thread_count",
                    "Worker threads for pipelines; 0 means automatic (logical cores \
                     minus one).",
                );
                let max = available_parallelism()
                    .map(|count| count.get())
                    .unwrap_or(1);
                let mut thread_count = common.thread_count;
                // Every `DragValue` here clamps only what the member enters: egui's default
                // clamps the stored value on sight and reports it changed, so merely opening
                // the menu would rewrite a hand-set value (a portable folder's thread count
                // from a bigger PC).
                if ui
                    .add(
                        egui::DragValue::new(&mut thread_count)
                            .range(0..=max)
                            .clamp_existing_to_range(false),
                    )
                    .labelled_by(threads.id)
                    .changed()
                {
                    common.thread_count = thread_count;
                    changed = true;
                }
                ui.end_row();

                let memory = row_label(
                    ui,
                    "Memory cap",
                    "memory_cap_percent",
                    "Memory budget cap for pipelines, as a percentage of physical \
                     memory.",
                );
                let mut memory_cap = common.memory_cap_percent;
                if ui
                    .add(
                        egui::DragValue::new(&mut memory_cap)
                            .range(0.1..=100.0)
                            .clamp_existing_to_range(false)
                            .suffix(" %"),
                    )
                    .labelled_by(memory.id)
                    .changed()
                {
                    common.memory_cap_percent = memory_cap;
                    changed = true;
                }
                ui.end_row();

                let hide_boots = row_label(
                    ui,
                    "FPC hide boots ID",
                    "fpc_hide_boots_id",
                    "The boots ID Hide body and an fpc_on marker write: an ID the \
                     game has no boots for. 4cc uses 55.",
                );
                let mut hide_boots_id = common.fpc_hide_boots_id;
                if ui
                    .add(
                        egui::DragValue::new(&mut hide_boots_id)
                            .range(fpc::HideIds::BOOTS)
                            .clamp_existing_to_range(false),
                    )
                    .labelled_by(hide_boots.id)
                    .changed()
                {
                    common.fpc_hide_boots_id = hide_boots_id;
                    changed = true;
                }
                ui.end_row();

                let hide_gloves = row_label(
                    ui,
                    "FPC hide gloves ID",
                    "fpc_hide_gloves_id",
                    "The gloves ID Hide body and an fpc_on marker write: an ID the \
                     game has no gloves for. 4cc uses 11.",
                );
                let mut hide_gloves_id = common.fpc_hide_gloves_id;
                if ui
                    .add(
                        egui::DragValue::new(&mut hide_gloves_id)
                            .range(fpc::HideIds::GLOVES)
                            .clamp_existing_to_range(false),
                    )
                    .labelled_by(hide_gloves.id)
                    .changed()
                {
                    common.fpc_hide_gloves_id = hide_gloves_id;
                    changed = true;
                }
                ui.end_row();

                let theme = row_label(ui, "Theme", "theme", "GUI color theme.");
                let mut picked = common.theme;
                egui::ComboBox::from_id_salt("settings-theme")
                    .selected_text(theme_label(picked))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut picked, Theme::Dark, "Dark");
                        ui.selectable_value(&mut picked, Theme::Light, "Light");
                    })
                    .response
                    .labelled_by(theme.id);
                if picked != common.theme {
                    common.theme = picked;
                    changed = true;
                }
                ui.end_row();

                let updates = row_label(
                    ui,
                    "Check for updates",
                    "check_for_updates",
                    "Whether the shell looks for new releases.",
                );
                let mut check = common.check_for_updates;
                if ui
                    .add(egui::Checkbox::without_text(&mut check))
                    .labelled_by(updates.id)
                    .changed()
                {
                    common.check_for_updates = check;
                    changed = true;
                }
                ui.end_row();

                let interval = row_label(
                    ui,
                    "Check interval (hours)",
                    "check_interval_hours",
                    "Hours between release checks.",
                );
                let mut hours = common.check_interval_hours;
                if ui
                    .add_enabled(
                        common.check_for_updates,
                        egui::DragValue::new(&mut hours)
                            .range(1..=u32::MAX)
                            .clamp_existing_to_range(false),
                    )
                    .labelled_by(interval.id)
                    .changed()
                {
                    common.check_interval_hours = hours;
                    changed = true;
                }
                ui.end_row();
            });
        if changed {
            ctx.settings().lock().unwrap().common = common;
            ctx.request(ShellRequest::SettingsChanged);
        }
    }
}

/// The `PES version` row: the sidebar selector's twin — the same entries, the same
/// blocker disabling it with `<label>: <reason>` on the disabled hover, and a pick
/// through `Sidebar::pick_version`, which spends the once-per-life auto-switch.
/// Its enabled tooltip is the shared exe check's, as the draft draws it.
fn version_row(
    ui: &mut egui::Ui,
    tools: &[Box<dyn StudioTool>],
    common: &CommonSettings,
    ctx: &ToolContext,
    sidebar: &mut Sidebar,
) {
    let label = row_label(
        ui,
        "PES version",
        "pes_version",
        "The PES version the suite targets; the sidebar selector.",
    );
    let blocker = version_blocker(tools);
    let mut picked = common.pes_version;
    let combo = ui
        .add_enabled_ui(blocker.is_none(), |ui| {
            egui::ComboBox::from_id_salt("settings-pes-version")
                .selected_text(version_label(picked, false))
                .show_ui(ui, |ui| {
                    for version in PesVersion::ALL {
                        ui.selectable_value(&mut picked, version, version_label(version, false));
                    }
                })
                .response
        })
        .inner
        .labelled_by(label.id);
    match blocker {
        Some((blocking, reason)) => {
            combo.on_disabled_hover_text(format!("{blocking}: {reason}"));
        }
        None => {
            let check = sidebar.exe_check(common, ui.input(|input| input.focused));
            combo.on_hover_text(exe_tooltip(check, &common.pes_folder()));
        }
    }
    ui.end_row();
    sidebar.pick_version(picked, ctx);
}

/// The theme combo's item label.
fn theme_label(theme: Theme) -> &'static str {
    match theme {
        Theme::Dark => "Dark",
        Theme::Light => "Light",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use egui::accesskit::{self, Role};
    use pes_version::PesVersion;

    use crate::pes_process::RunningPes;
    use crate::settings::{Settings, Theme};
    use crate::shell::test_harness::*;
    use crate::status::{ActionEffect, Notice, NoticeAction};

    /// The node carrying `text` as a label-role node's `value`.
    fn text_node<'a>(nodes: &'a [accesskit::Node], text: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| node.value() == Some(text))
            .unwrap_or_else(|| panic!("no node with text {text:?}"))
    }

    /// The widget of `role` drawn on `label`'s grid row (the same y span).
    fn widget_on_row<'a>(
        nodes: &'a [accesskit::Node],
        label: &str,
        role: Role,
    ) -> &'a accesskit::Node {
        let row = text_node(nodes, label)
            .bounds()
            .expect("the label has bounds");
        nodes
            .iter()
            .find(|node| {
                node.role() == role
                    && node
                        .bounds()
                        .is_some_and(|bounds| bounds.y0 < row.y1 && bounds.y1 > row.y0)
            })
            .unwrap_or_else(|| panic!("no {role:?} on the {label:?} row"))
    }

    /// A key event.
    fn key(key: egui::Key, pressed: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        }
    }

    /// Nodes with `label` as their widget label: a tool's sidebar entry plus, when it
    /// has settings, its section header.
    fn labeled(nodes: &[accesskit::Node], label: &str) -> usize {
        nodes
            .iter()
            .filter(|node| node.label() == Some(label))
            .count()
    }

    // ST-SET-01
    #[test]
    fn the_gear_opens_the_menu_on_the_active_tools_section_and_resets_each_opening() {
        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let (c, calls_c) = stub("c", "C", None);
        // A tool whose `default_settings()` is empty gets no section.
        calls_c.default_settings.lock().unwrap().clear();
        let dir = TestDir::new("studio_core_settings_sections");
        let Shell { mut app, ctx, .. } = shell_with(
            test_settings(dir.path()),
            vec![a, b, c],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a, calls_b, calls_c],
        );
        ctx.switch_to_tool("b");
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert_eq!(app.active, 1);
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        // The collapsing animation settles over a few frames.
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        assert!(has_text(&nodes, "Threads"), "the common grid is drawn");
        assert_eq!(labeled(&nodes, "A"), 2, "A's sidebar entry and its header");
        assert_eq!(labeled(&nodes, "B"), 2, "B's sidebar entry and its header");
        assert_eq!(
            labeled(&nodes, "C"),
            1,
            "C has no settings: the entry alone"
        );
        assert!(
            has_text(&nodes, "B settings body"),
            "the active tool's section is expanded"
        );
        assert!(
            !has_text(&nodes, "A settings body"),
            "the other sections are collapsed"
        );

        // A click on B's header collapses its section, and the collapse holds while the
        // menu stays open: the member's toggle is the state, not the open set.
        let header = nodes
            .iter()
            .find(|node| node.label() == Some("B") && node.toggled().is_none())
            .expect("B's section header");
        click(&egui_ctx, &mut app, header);
        let nodes = idle(&egui_ctx, &mut app, 15).nodes;
        assert!(
            !has_text(&nodes, "B settings body"),
            "the member's collapse holds"
        );
        for _ in 0..3 {
            let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
            assert!(
                !has_text(&nodes, "B settings body"),
                "B's section stays collapsed"
            );
            assert!(has_text(&nodes, "Threads"), "the menu is still open");
        }

        // Re-expand B before the close: without the reset, a reopening would show its
        // section still open, so the collapsed assertion below proves the reset.
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let header = nodes
            .iter()
            .find(|node| node.label() == Some("B") && node.toggled().is_none())
            .expect("B's section header");
        click(&egui_ctx, &mut app, header);
        let nodes = idle(&egui_ctx, &mut app, 15).nodes;
        assert!(
            has_text(&nodes, "B settings body"),
            "B expanded again: the reopening can only collapse it through the reset"
        );

        // Close, make A active, reopen: the open set is reset to the active tool.
        click(&egui_ctx, &mut app, entry(&nodes, "Close"));
        ctx.switch_to_tool("a");
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert_eq!(app.active, 0);
        assert!(!has_text(&nodes, "B settings body"), "the menu is closed");
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        // B's collapse animates out over 0.2 s of frames before its body stops drawing.
        let nodes = idle(&egui_ctx, &mut app, 15).nodes;
        assert!(
            has_text(&nodes, "A settings body"),
            "now the active tool's section is expanded"
        );
        assert!(
            !has_text(&nodes, "B settings body"),
            "the open set was reset at the reopening"
        );
    }

    // ST-SET-02
    #[test]
    fn a_threads_change_writes_the_common_setting_and_keeps_the_tool_tables() {
        let dir = test_dir("studio_core_settings_threads");
        let data = dir.join("data");
        fs::create_dir_all(&data).unwrap();
        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let table_a = a.default_settings();
        let table_b = b.default_settings();
        let mut settings = test_settings(&dir);
        settings.set_tool("a", table_a.clone());
        settings.set_tool("b", table_b.clone());
        let Shell { mut app, ctx, .. } = shell_with(
            settings,
            vec![a, b],
            &[],
            Some(data.clone()),
            vec![calls_a, calls_b],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();
        assert_ne!(
            ctx.common().thread_count,
            1,
            "the test's pick really changes the setting"
        );

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        // A click on a DragValue turns it into a text field; type and press Enter.
        click(
            &egui_ctx,
            &mut app,
            widget_on_row(&nodes, "Threads", Role::SpinButton),
        );
        frame(
            &egui_ctx,
            &mut app,
            vec![egui::Event::Text("1".to_owned())],
            true,
        );
        frame(&egui_ctx, &mut app, vec![key(egui::Key::Enter, true)], true);
        assert_eq!(ctx.common().thread_count, 1);
        frame(&egui_ctx, &mut app, Vec::new(), true);

        let text = fs::read_to_string(data.join(crate::settings::SETTINGS_FILE_NAME)).unwrap();
        let file: toml::Table = toml::from_str(&text).unwrap();
        assert_eq!(
            file["common"]["thread_count"],
            toml::Value::Integer(1),
            "the file holds the new value under [common]"
        );
        assert_eq!(
            file["a"],
            toml::Value::Table(table_a),
            "a's table as it was"
        );
        assert_eq!(
            file["b"],
            toml::Value::Table(table_b),
            "b's table as it was"
        );
        assert_eq!(ctx.common().thread_count, 1);
        fs::remove_dir_all(&dir).unwrap();
    }

    // The `Memory cap` row's entry range accepts a fraction of a percent (the common
    // settings allow `0 < percent <= 100`).
    #[test]
    fn the_memory_cap_row_accepts_a_fraction_of_a_percent() {
        let dir = TestDir::new("studio_core_settings_memcap");
        let (a, calls_a) = stub("a", "A", None);
        let settings = test_settings(dir.path());
        assert_ne!(
            settings.common.memory_cap_percent, 0.5,
            "the test's pick really changes the setting"
        );
        let Shell { mut app, ctx, .. } = shell_with(
            settings,
            vec![a],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        click(
            &egui_ctx,
            &mut app,
            widget_on_row(&nodes, "Memory cap", Role::SpinButton),
        );
        frame(
            &egui_ctx,
            &mut app,
            vec![egui::Event::Text("0.5".to_owned())],
            true,
        );
        frame(&egui_ctx, &mut app, vec![key(egui::Key::Enter, true)], true);
        assert_eq!(ctx.common().memory_cap_percent, 0.5);
    }

    // The FPC hide ID rows, after the memory cap: the boots row's entry range is
    // the stock band (1 to 100), the gloves row's its own (11 to 100), and a
    // typed value writes the common setting.
    #[test]
    fn the_hide_id_rows_write_the_common_settings() {
        let dir = TestDir::new("studio_core_settings_hide_ids");
        let (a, calls_a) = stub("a", "A", None);
        let settings = test_settings(dir.path());
        assert_ne!(
            settings.common.fpc_hide_boots_id, 60,
            "the test's pick really changes the setting"
        );
        assert_ne!(
            settings.common.fpc_hide_gloves_id, 12,
            "the test's pick really changes the setting"
        );
        let Shell { mut app, ctx, .. } = shell_with(
            settings,
            vec![a],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        for (label, min, max) in [
            ("FPC hide boots ID", 1.0, 100.0),
            ("FPC hide gloves ID", 11.0, 100.0),
        ] {
            let field = widget_on_row(&nodes, label, Role::SpinButton);
            assert_eq!(
                (field.min_numeric_value(), field.max_numeric_value()),
                (Some(min), Some(max)),
                "{label}'s entry range"
            );
        }

        click(
            &egui_ctx,
            &mut app,
            widget_on_row(&nodes, "FPC hide boots ID", Role::SpinButton),
        );
        frame(
            &egui_ctx,
            &mut app,
            vec![egui::Event::Text("60".to_owned())],
            true,
        );
        frame(&egui_ctx, &mut app, vec![key(egui::Key::Enter, true)], true);
        assert_eq!(ctx.common().fpc_hide_boots_id, 60);

        click(
            &egui_ctx,
            &mut app,
            widget_on_row(&nodes, "FPC hide gloves ID", Role::SpinButton),
        );
        frame(
            &egui_ctx,
            &mut app,
            vec![egui::Event::Text("12".to_owned())],
            true,
        );
        frame(&egui_ctx, &mut app, vec![key(egui::Key::Enter, true)], true);
        assert_eq!(ctx.common().fpc_hide_gloves_id, 12);
        assert_eq!(ctx.common().fpc_hide_boots_id, 60, "the boots write stands");
    }

    // The `Check interval (hours)` row is disabled while `check_for_updates` is off
    // and enabled while it is on.
    #[test]
    fn the_interval_row_follows_check_for_updates() {
        let dir = TestDir::new("studio_core_settings_interval");
        let (a, calls_a) = stub("a", "A", None);
        let mut settings = test_settings(dir.path());
        settings.common.check_for_updates = false;
        let Shell { mut app, .. } = shell_with(
            settings,
            vec![a],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        let interval = widget_on_row(&nodes, "Check interval (hours)", Role::SpinButton);
        assert!(
            interval.is_disabled(),
            "the interval is disabled while checks are off"
        );

        click(
            &egui_ctx,
            &mut app,
            widget_on_row(&nodes, "Check for updates", Role::CheckBox),
        );
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert!(
            !widget_on_row(&nodes, "Check interval (hours)", Role::SpinButton).is_disabled(),
            "the interval is enabled once checks are on"
        );
    }

    // Opening the menu changes nothing by itself: a hand-set `thread_count` above this
    // machine's cores (a portable folder moved to a smaller PC) stays as written, though
    // the row's drag range stops at the core count.
    #[test]
    fn opening_the_menu_keeps_a_thread_count_above_the_core_count() {
        let (a, calls_a) = stub("a", "A", None);
        let dir = TestDir::new("studio_core_settings_no_clamp");
        let mut settings = test_settings(dir.path());
        settings.common.thread_count = 4096;
        let Shell { mut app, ctx, .. } = shell_with(
            settings,
            vec![a],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        assert!(has_text(&nodes, "Threads"), "the menu is open");
        assert_eq!(ctx.common().thread_count, 4096);
    }

    // The theme setting, never the system's theme, decides (gui.md "Other UI
    // components"); a pick applies at the next frame. Cites no scenario: the
    // Acceptance section has none for it.
    #[test]
    fn the_theme_setting_decides_and_a_pick_applies_at_the_next_frame() {
        let dir = test_dir("studio_core_settings_theme");
        let data = dir.join("data");
        fs::create_dir_all(&data).unwrap();
        let (a, calls_a) = stub("a", "A", None);
        let mut settings = test_settings(&dir);
        settings.common.theme = Theme::Dark;
        let Shell { mut app, ctx, .. } =
            shell_with(settings, vec![a], &[], Some(data.clone()), vec![calls_a]);
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        // A light desktop does not lift the dark default.
        frame_theme(
            &egui_ctx,
            &mut app,
            Vec::new(),
            true,
            Some(egui::Theme::Light),
        );
        assert_eq!(
            egui_ctx.theme(),
            egui::Theme::Dark,
            "the setting wins over the system's theme"
        );

        let nodes = frame_theme(
            &egui_ctx,
            &mut app,
            Vec::new(),
            true,
            Some(egui::Theme::Light),
        )
        .nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        assert_eq!(
            widget_on_row(&nodes, "Theme", Role::ComboBox).value(),
            Some("Dark"),
            "the combo shows the setting's theme"
        );
        click(
            &egui_ctx,
            &mut app,
            widget_on_row(&nodes, "Theme", Role::ComboBox),
        );
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Light"));
        assert_eq!(ctx.common().theme, Theme::Light);
        frame_theme(
            &egui_ctx,
            &mut app,
            Vec::new(),
            true,
            Some(egui::Theme::Dark),
        );
        assert_eq!(
            egui_ctx.theme(),
            egui::Theme::Light,
            "the pick applied at the next frame, on a dark desktop too"
        );
        let text = fs::read_to_string(data.join(crate::settings::SETTINGS_FILE_NAME)).unwrap();
        let file: toml::Table = toml::from_str(&text).unwrap();
        assert_eq!(file["common"]["theme"].as_str(), Some("light"));
        fs::remove_dir_all(&dir).unwrap();
    }

    // A text row commits when its edit loses focus or Enter is pressed, not per
    // keystroke (gui.md "Settings menu", "No Save or Apply").
    #[test]
    fn a_path_row_commits_on_enter_not_per_keystroke() {
        let dir = test_dir("studio_core_settings_path_commit");
        let data = dir.join("data");
        fs::create_dir_all(&data).unwrap();
        let (a, calls_a) = stub("a", "A", None);
        let Shell { mut app, ctx, .. } = shell_with(
            test_settings(&dir),
            vec![a],
            &[],
            Some(data.clone()),
            vec![calls_a],
        );
        let before = ctx.common().pes_folder_path;
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        // The committed path is under the test's own folder: the next frame's exe
        // check lists it, so it must not reach a developer path.
        let pes_folder = dir.join("pes19").display().to_string();
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        click(
            &egui_ctx,
            &mut app,
            widget_on_row(&nodes, "PES folder", Role::TextInput),
        );
        frame(
            &egui_ctx,
            &mut app,
            vec![
                egui::Event::Key {
                    key: egui::Key::A,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::COMMAND,
                },
                egui::Event::Text(pes_folder.clone()),
            ],
            true,
        );
        assert_eq!(
            ctx.common().pes_folder_path,
            before,
            "a half-typed path does not reach the setting"
        );
        frame(&egui_ctx, &mut app, vec![key(egui::Key::Enter, true)], true);
        assert_eq!(
            ctx.common().pes_folder_path,
            pes_folder,
            "Enter commits the edit"
        );
        frame(&egui_ctx, &mut app, Vec::new(), true);
        let written = Settings::load(&data.join(crate::settings::SETTINGS_FILE_NAME)).unwrap();
        assert_eq!(
            written.common.pes_folder_path,
            dir.join("pes19").display().to_string()
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    // A path row draws its Browse button right of the edit (gui.md "Settings
    // menu", the path rows' bullet). The picker itself is `rfd`'s native
    // dialog, so the test sees the button, never a click.
    #[test]
    fn a_path_row_draws_browse_right_of_the_edit() {
        let dir = TestDir::new("studio_core_settings_browse");
        let (a, calls_a) = stub("a", "A", None);
        let Shell { mut app, .. } = shell_with(
            test_settings(dir.path()),
            vec![a],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        let edit = widget_on_row(&nodes, "PES folder", Role::TextInput);
        let browse = widget_on_row(&nodes, "PES folder", Role::Button);
        assert_eq!(browse.label(), Some("Browse"));
        assert!(
            centre(browse).x > centre(edit).x,
            "the button stands right of the edit"
        );
    }

    // The `PES version` row is the sidebar selector's twin: a pick writes the setting
    // through the same function, which spends the once-per-life auto-switch.
    #[test]
    fn a_menu_pick_writes_pes_version_and_spends_the_auto_switch() {
        fn running_19() -> Vec<RunningPes> {
            vec![RunningPes {
                version: PesVersion::Pes19,
                exe: None,
            }]
        }
        let (a, calls_a) = stub("a", "A", None);
        let dir = TestDir::new("studio_core_settings_pick");
        let mut settings = test_settings(dir.path());
        settings.common.pes_version = PesVersion::Pes19;
        let Shell { mut app, ctx, .. } = shell_with(
            settings,
            vec![a],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        click(
            &egui_ctx,
            &mut app,
            widget_on_row(&nodes, "PES version", Role::ComboBox),
        );
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "PES 2021"));
        assert_eq!(ctx.common().pes_version, PesVersion::Pes21);
        assert!(
            app.sidebar.auto_switch_used,
            "the pick spent the once-per-life auto-switch"
        );

        // A probe reporting another running version switches nothing at the next poll.
        app.sidebar.probe = running_19;
        frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(5.0));
        assert_eq!(ctx.common().pes_version, PesVersion::Pes21);
    }

    // While a tool reports a `version_change_blocker` the row is disabled and its
    // disabled-hover tooltip is the blocking tool's `<label>: <reason>` (the
    // sidebar's rule, ST-SIDE-04).
    #[test]
    fn a_blocking_tool_disables_the_menu_version_row_with_its_reason() {
        let (a, calls_a) = stub("a", "A", Some("a compile is running"));
        let (b, calls_b) = stub("b", "B", None);
        let dir = TestDir::new("studio_core_settings_blocked");
        let Shell { mut app, .. } = shell_with(
            test_settings(dir.path()),
            vec![a, b],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a, calls_b],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();
        egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        let combo = widget_on_row(&nodes, "PES version", Role::ComboBox);
        assert!(combo.is_disabled(), "the blocked row is disabled");
        // The click's tooltip cooldown ages out first, then `is_still` takes about
        // 0.1 s of idle frames.
        idle(&egui_ctx, &mut app, 8);
        frame(&egui_ctx, &mut app, hover(combo), true);
        let nodes = idle(&egui_ctx, &mut app, 8).nodes;
        assert!(
            has_text(&nodes, "A: a compile is running"),
            "the disabled hover is the blocking tool's reason"
        );
    }

    // A notice's `ActionEffect::OpenSettings` opens the menu on the active tool's
    // section, like the gear.
    #[test]
    fn a_notice_with_open_settings_opens_the_menu_on_the_active_tool() {
        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let dir = TestDir::new("studio_core_settings_notice");
        let Shell { mut app, ctx, .. } = shell_with(
            test_settings(dir.path()),
            vec![a, b],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a, calls_b],
        );
        ctx.switch_to_tool("b");
        ctx.notify(Notice {
            text: "choose".to_owned(),
            action: Some(NoticeAction {
                label: "Open settings".to_owned(),
                effect: ActionEffect::OpenSettings,
            }),
        });
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert_eq!(app.active, 1);
        click(&egui_ctx, &mut app, entry(&nodes, "Open settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        assert!(
            has_text(&nodes, "B settings body"),
            "the menu opened on the active tool's section"
        );
        assert!(!has_text(&nodes, "A settings body"));
    }

    // A row label's tooltip leads with the settings key, then the field's doc
    // comment first sentence (the draft's `sRow`).
    #[test]
    fn a_row_labels_tooltip_leads_with_the_settings_key() {
        let (a, calls_a) = stub("a", "A", None);
        let dir = TestDir::new("studio_core_settings_tooltip");
        let Shell { mut app, .. } = shell_with(
            test_settings(dir.path()),
            vec![a],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();
        egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Settings"));
        let nodes = idle(&egui_ctx, &mut app, 10).nodes;
        idle(&egui_ctx, &mut app, 8);
        frame(
            &egui_ctx,
            &mut app,
            hover(text_node(&nodes, "Threads")),
            true,
        );
        let nodes = idle(&egui_ctx, &mut app, 8).nodes;
        assert!(
            has_text(
                &nodes,
                "thread_count: Worker threads for pipelines; 0 means automatic \
                 (logical cores minus one)."
            ),
            "the tooltip is the settings key, then the doc comment's first sentence"
        );
    }
}
