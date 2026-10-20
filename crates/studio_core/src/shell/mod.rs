//! The app shell: `StudioApp`, the window with a sidebar listing the registered tools, the
//! status bar under everything, the window title, the settings menu, the active tool's
//! view, every tool ticked each frame and the tools' shell requests applied; and the
//! launch modes' argument parsing (`launch`). The help window and the common widgets join
//! this module later in the GUI phase (core plan, "Shell layout", "Crate structure").

mod about;
mod launch;
mod settings_menu;
mod sidebar;
mod status_bar;
#[cfg(test)]
mod test_harness;
mod title;

use crossbeam_channel::Receiver;
use log::debug;
#[cfg(not(target_arch = "wasm32"))]
use log::warn;

pub use launch::{Launch, LaunchMode, parse_launch, run_cli};

use crate::settings::Theme;
use crate::status::ActionEffect;
#[cfg(not(target_arch = "wasm32"))]
use crate::status::ShellCondition;
use crate::tool::{PlannedTool, ShellRequest, StudioTool, ToolContext};

use self::settings_menu::SettingsMenu;
use self::sidebar::{Sidebar, SidebarClick};
use self::status_bar::StatusBar;
use self::title::WindowTitle;

/// The window: the registered tools in sidebar order, the platform handle they share, the
/// receiving end of their shell requests, and which tool's view is on screen.
pub struct StudioApp {
    tools: Vec<Box<dyn StudioTool>>,
    /// The suite's whole registry as data: the entries no registered tool has are the
    /// sidebar's grayed planned list.
    planned: &'static [PlannedTool],
    ctx: ToolContext,
    requests: Receiver<ShellRequest>,
    /// Index into `tools` of the active view.
    active: usize,
    sidebar: Sidebar,
    status_bar: StatusBar,
    settings_menu: SettingsMenu,
    title: WindowTitle,
    /// The common `theme` setting last applied to the context; `None` until the first
    /// frame, which applies it.
    applied_theme: Option<Theme>,
}

impl StudioApp {
    /// A shell over `tools`, opening on the one the `last_tool` setting names (the first when
    /// it names none or an unregistered one; ST-SIDE-01, ST-RUN-02). `planned` is the suite's
    /// registry including the tools this build does not register. `requests` is the receiving
    /// end of the channel `ctx` sends shell requests to.
    pub fn new(
        tools: Vec<Box<dyn StudioTool>>,
        planned: &'static [PlannedTool],
        ctx: ToolContext,
        requests: Receiver<ShellRequest>,
    ) -> StudioApp {
        let active = ctx
            .common()
            .last_tool
            .and_then(|last| tools.iter().position(|tool| tool.id() == last))
            .unwrap_or(0);
        StudioApp {
            tools,
            planned,
            ctx,
            requests,
            active,
            sidebar: Sidebar::default(),
            status_bar: StatusBar::default(),
            settings_menu: SettingsMenu::default(),
            title: WindowTitle::default(),
            applied_theme: None,
        }
    }

    /// Makes `tools[index]` the active view and records it in `last_tool`, through the same
    /// best-effort settings save every change uses: a switch writes it, not the exit, so a
    /// crash keeps the record (ST-RUN-02).
    fn set_active(&mut self, index: usize) {
        if index == self.active {
            return;
        }
        self.active = index;
        self.ctx.settings().lock().unwrap().common.last_tool =
            Some(self.tools[index].id().to_owned());
        self.ctx.request(ShellRequest::SettingsChanged);
    }

    /// Writes the settings file, best-effort: a failed save keeps the in-memory change for
    /// the session and raises the `data_dir_read_only` condition in the status bar (its only
    /// notice — no dialog, no retry); a save that succeeds drops it (ST-SET-03).
    #[cfg(not(target_arch = "wasm32"))]
    fn save_settings(&mut self) {
        if let Some(dir) = &self.ctx.paths().data_dir {
            let path = dir.join(crate::settings::SETTINGS_FILE_NAME);
            let result = self.ctx.settings().lock().unwrap().save(&path);
            self.status_bar
                .set_condition(ShellCondition::DataDirReadOnly, result.is_err());
            if let Err(error) = result {
                warn!(
                    "the settings file could not be written to {}: {error}",
                    path.display()
                );
            }
        }
    }

    /// The browser build has no settings file; its storage is the web phase's.
    #[cfg(target_arch = "wasm32")]
    fn save_settings(&mut self) {}

    /// Applies every request the tools queued since the last frame.
    fn apply_requests(&mut self, egui_ctx: &egui::Context) {
        let focused = egui_ctx.input(|input| input.focused);
        while let Ok(request) = self.requests.try_recv() {
            match request {
                ShellRequest::SwitchTool(id) => self.switch_to(id.as_str()),
                // A settings write queues this; the save is best-effort, at the next frame.
                ShellRequest::SettingsChanged => self.save_settings(),
                // The newest notice replaces the one before it; while the window is
                // unfocused its text also holds in the title until focus returns.
                ShellRequest::Notify(notice) => {
                    self.title.notice(&notice.text, focused);
                    self.status_bar.set_notice(notice);
                }
            }
        }
    }

    /// Makes the tool with `id` the active view, if it is registered.
    fn switch_to(&mut self, id: &str) {
        match self.tools.iter().position(|tool| tool.id() == id) {
            Some(index) => self.set_active(index),
            None => debug!("switch to unknown tool `{id}` ignored"),
        }
    }

    /// The `activity()` texts of the registered tools in sidebar order; `include_active`
    /// puts the active tool's first (the window title counts it too — the status bar's
    /// middle slot skips it because the tool's own run strip already shows it).
    fn activities(&self, include_active: bool) -> Vec<String> {
        let mut texts = Vec::new();
        if include_active
            && let Some(activity) = self.tools.get(self.active).and_then(|tool| tool.activity())
        {
            texts.push(activity.text());
        }
        for (index, tool) in self.tools.iter().enumerate() {
            if index == self.active {
                continue;
            }
            if let Some(activity) = tool.activity() {
                texts.push(activity.text());
            }
        }
        texts
    }

    /// Applies a clicked notice action (the notice is already dismissed). `SwitchTool` goes
    /// through the shell's own switch; `FixPesPath` rewrites `pes_folder_path` from the
    /// running exe's folder, the year of the version the notice was raised for in its last
    /// segment becoming `20**` again (a segment without the year is kept), then saves as any
    /// change; `OpenSettings` opens the settings menu on the active tool's section.
    /// `OpenFolder` and `OpenHelp` have no target yet (the compile's "Open output" and the
    /// help window are later GUI-phase steps): they only dismiss.
    fn apply_notice_effect(&mut self, effect: ActionEffect) {
        match effect {
            ActionEffect::SwitchTool(id) => self.switch_to(&id),
            ActionEffect::FixPesPath { folder, version } => {
                let year = version.year().to_string();
                let path = match folder.file_name().and_then(|name| name.to_str()) {
                    Some(name) => folder.with_file_name(name.replace(&year, "20**")),
                    None => folder,
                };
                self.ctx.settings().lock().unwrap().common.pes_folder_path =
                    path.display().to_string();
                self.ctx.request(ShellRequest::SettingsChanged);
            }
            ActionEffect::OpenSettings => self.settings_menu.open(self.active, &self.ctx),
            effect @ (ActionEffect::OpenFolder(_) | ActionEffect::OpenHelp(_)) => {
                debug!("notice action {effect:?} has no target yet");
            }
        }
    }
}

impl eframe::App for StudioApp {
    fn logic(&mut self, egui_ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // The setting, never the system's theme, decides (gui.md "Other UI components"):
        // applied once per change, so a change is live at the next frame.
        let theme = self.ctx.common().theme;
        if self.applied_theme != Some(theme) {
            crate::theme::apply(egui_ctx, theme);
            self.applied_theme = Some(theme);
        }
        for tool in &mut self.tools {
            tool.tick(&self.ctx);
        }
        // The poll runs before the requests drain so a switch's own requests (the settings
        // save, the notice) apply in the frame that switches, not one repaint later.
        self.sidebar
            .poll_running_pes(egui_ctx, &self.tools, &self.ctx);
        self.apply_requests(egui_ctx);
        let focused = egui_ctx.input(|input| input.focused);
        if let Some(title) = self.title.frame(&self.activities(true), focused) {
            egui_ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The status bar claims its row first, so it spans the whole width under the sidebar
        // and the tool view (gui.md "Shell layout").
        let activities = self.activities(false);
        let effect = egui::Panel::bottom("status_bar")
            .exact_size(24.0)
            .frame(
                egui::Frame::side_top_panel(ui.style()).inner_margin(egui::Margin::symmetric(8, 0)),
            )
            .show(ui, |ui| self.status_bar.show(ui, &activities))
            .inner;
        if let Some(effect) = effect {
            self.apply_notice_effect(effect);
        }
        let clicked = egui::Panel::left("sidebar")
            .exact_size(self.sidebar.width())
            .show(ui, |ui| {
                self.sidebar
                    .show(ui, &self.tools, self.active, self.planned, &self.ctx)
            })
            .inner;
        match clicked {
            Some(SidebarClick::SwitchTool(index)) => self.set_active(index),
            Some(SidebarClick::OpenSettings) => {
                self.settings_menu.open(self.active, &self.ctx);
            }
            None => {}
        }
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(tool) = self.tools.get_mut(self.active) {
                tool.view(ui, &self.ctx);
            }
        });
        self.settings_menu
            .show(ui.ctx(), &mut self.tools, &self.ctx, &mut self.sidebar);
    }
}

/// Opens the window titled "4cc Studio" on `app` and runs it until it closes; an error is a
/// window or graphics context that could not be set up.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_gui(app: StudioApp) -> anyhow::Result<()> {
    eframe::run_native(
        "4cc Studio",
        eframe::NativeOptions::default(),
        Box::new(|creation| {
            crate::theme::install_fonts(&creation.egui_ctx);
            // The context's clones share the wake slot, so a `with_events` sink made before
            // the window exists still wakes it (gui.md "Event wiring").
            let egui_ctx = creation.egui_ctx.clone();
            app.ctx
                .set_wake(move |delay| egui_ctx.request_repaint_after(delay));
            Ok(Box::new(app))
        }),
    )
    .map_err(|error| anyhow::anyhow!("cannot open the window: {error}"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crossbeam_channel::TryRecvError;
    use eframe::App;

    use super::test_harness::*;
    use super::*;
    use crate::settings::Settings;
    use crate::status::Notice;

    /// `apply_requests` outside a frame: a default egui context.
    fn apply_requests(app: &mut StudioApp) {
        app.apply_requests(&egui::Context::default());
    }

    #[test]
    fn a_switch_request_makes_the_named_tool_active() {
        let Shell { mut app, ctx, .. } = shell();
        assert_eq!(app.active, 0);
        ctx.switch_to_tool("b");
        apply_requests(&mut app);
        assert_eq!(app.active, 1);
    }

    #[test]
    fn a_switch_to_an_unknown_tool_changes_nothing() {
        let Shell { mut app, ctx, .. } = shell();
        ctx.switch_to_tool("b");
        ctx.switch_to_tool("zz");
        apply_requests(&mut app);
        assert_eq!(app.active, 1);
    }

    #[test]
    fn notices_and_settings_changes_are_drained() {
        let Shell { mut app, ctx, .. } = shell();
        ctx.notify(Notice {
            text: "hello".to_owned(),
            action: None,
        });
        ctx.set_tool_settings("a", toml::Table::new());
        apply_requests(&mut app);
        assert_eq!(app.requests.try_recv(), Err(TryRecvError::Empty));
        assert_eq!(app.active, 0);
    }

    #[test]
    fn logic_ticks_every_tool_then_applies_the_queued_requests() {
        let Shell {
            mut app,
            ctx,
            calls,
            ..
        } = shell();
        ctx.switch_to_tool("b");
        // eframe's own constructor of a `Frame` for testing an `App` without a window.
        app.logic(
            &egui::Context::default(),
            &mut eframe::Frame::_new_kittest(),
        );
        assert_eq!((calls[0].ticks(), calls[1].ticks()), (1, 1));
        assert_eq!(app.active, 1);
    }

    // ST-RUN-02
    #[test]
    fn the_gui_reopens_on_the_tool_active_when_it_closed_on() {
        let dir = test_dir("studio_core_sidebar_last_tool");
        fs::create_dir_all(&dir).unwrap();
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        {
            let (a, calls_a) = stub("a", "A", None);
            let (b, calls_b) = stub("b", "B", None);
            let Shell { mut app, .. } = shell_with(
                test_settings(&dir),
                vec![a, b],
                &[],
                Some(dir.clone()),
                vec![calls_a, calls_b],
            );
            let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
            click(&egui_ctx, &mut app, entry(&nodes, "B"));
            assert_eq!(app.active, 1);
            frame(&egui_ctx, &mut app, Vec::new(), true);
        }

        let written = Settings::load(&dir.join(crate::settings::SETTINGS_FILE_NAME)).unwrap();
        assert_eq!(
            written.common.last_tool.as_deref(),
            Some("b"),
            "the switch wrote last_tool"
        );

        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let Shell { mut app, calls, .. } = shell_with(
            written,
            vec![a, b],
            &[],
            Some(dir.clone()),
            vec![calls_a, calls_b],
        );
        frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(app.active, 1, "the GUI reopens on the second tool");
        assert_eq!(calls[1].views(), 1);
        fs::remove_dir_all(&dir).unwrap();
    }
}
