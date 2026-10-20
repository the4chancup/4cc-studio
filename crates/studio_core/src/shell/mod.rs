//! The app shell: `StudioApp`, the window with a sidebar listing the registered tools and the
//! active tool's view, every tool ticked each frame and the tools' shell requests applied; and
//! the launch modes' argument parsing (`launch`). The sidebar (`sidebar`) is built; the status
//! bar, the settings menu, the help window and the common widgets join this module in the GUI
//! phase (core plan, "Shell layout", "Crate structure").

mod about;
mod launch;
mod sidebar;

use crossbeam_channel::Receiver;
use log::debug;
#[cfg(not(target_arch = "wasm32"))]
use log::warn;

pub use launch::{Launch, LaunchMode, parse_launch, run_cli};

use crate::tool::{PlannedTool, ShellRequest, StudioTool, ToolContext};

use self::sidebar::Sidebar;

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
    /// the session and the `data_dir_read_only` status-bar condition (which lands with the
    /// status bar) is the only notice.
    #[cfg(not(target_arch = "wasm32"))]
    fn save_settings(&self) {
        if let Some(dir) = &self.ctx.paths().data_dir {
            let path = dir.join(crate::settings::SETTINGS_FILE_NAME);
            let settings = self.ctx.settings().lock().unwrap();
            if let Err(error) = settings.save(&path) {
                warn!(
                    "the settings file could not be written to {}: {error}",
                    path.display()
                );
            }
        }
    }

    /// The browser build has no settings file; its storage is the web phase's.
    #[cfg(target_arch = "wasm32")]
    fn save_settings(&self) {}

    /// Applies every request the tools queued since the last frame.
    fn apply_requests(&mut self) {
        while let Ok(request) = self.requests.try_recv() {
            match request {
                ShellRequest::SwitchTool(id) => {
                    match self.tools.iter().position(|tool| tool.id() == id) {
                        Some(index) => self.set_active(index),
                        None => debug!("switch to unknown tool `{id}` ignored"),
                    }
                }
                // A settings write queues this; the save is best-effort, at the next frame.
                ShellRequest::SettingsChanged => self.save_settings(),
                // The status bar, which shows notices, arrives with the GUI phase; until then
                // a notice has no consumer.
                ShellRequest::Notify(_) => {}
            }
        }
    }
}

impl eframe::App for StudioApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        for tool in &mut self.tools {
            tool.tick(&self.ctx);
        }
        self.apply_requests();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let clicked = egui::Panel::left("sidebar")
            .exact_size(self.sidebar.width())
            .show(ui, |ui| {
                self.sidebar
                    .show(ui, &self.tools, self.active, self.planned, &self.ctx)
            })
            .inner;
        if let Some(index) = clicked {
            self.set_active(index);
        }
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(tool) = self.tools.get_mut(self.active) {
                tool.view(ui, &self.ctx);
            }
        });
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
            Ok(Box::new(app))
        }),
    )
    .map_err(|error| anyhow::anyhow!("cannot open the window: {error}"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use crossbeam_channel::{TryRecvError, unbounded};
    use eframe::App;
    use egui::accesskit::{self, Role, Toggled};

    use super::sidebar::{ExeCheck, selector_color};
    use super::*;
    use crate::help::HelpSection;
    use crate::settings::Settings;
    use crate::status::Notice;
    use crate::tool::{AppPaths, CliError};
    use pes_version::PesVersion;

    /// How often the shell called a stub tool's `tick` and `view`, plus the common settings
    /// the last `tick` read, shared with the test.
    #[derive(Default)]
    struct Calls {
        ticks: AtomicUsize,
        views: AtomicUsize,
        version_seen: Mutex<Option<PesVersion>>,
    }

    impl Calls {
        fn ticks(&self) -> usize {
            self.ticks.load(Ordering::SeqCst)
        }
        fn views(&self) -> usize {
            self.views.load(Ordering::SeqCst)
        }
        fn version_seen(&self) -> Option<PesVersion> {
            *self.version_seen.lock().unwrap()
        }
    }

    struct StubTool {
        id: &'static str,
        label: &'static str,
        icon: &'static str,
        blocker: Option<&'static str>,
        calls: Arc<Calls>,
    }

    impl StudioTool for StubTool {
        fn id(&self) -> &'static str {
            self.id
        }
        fn label(&self) -> &'static str {
            self.label
        }
        fn icon(&self) -> &'static str {
            self.icon
        }
        fn view(&mut self, _ui: &mut egui::Ui, _ctx: &ToolContext) {
            self.calls.views.fetch_add(1, Ordering::SeqCst);
        }
        fn tick(&mut self, ctx: &ToolContext) {
            *self.calls.version_seen.lock().unwrap() = Some(ctx.common().pes_version);
            self.calls.ticks.fetch_add(1, Ordering::SeqCst);
        }
        fn settings_view(&mut self, _ui: &mut egui::Ui) {}
        fn default_settings(&self) -> toml::Table {
            toml::Table::new()
        }
        fn help(&self) -> HelpSection {
            HelpSection {
                title: "Stub",
                topics: Vec::new(),
            }
        }
        fn cli_command(&self) -> clap::Command {
            clap::Command::new("stub")
        }
        fn cli_run(&self, _matches: &clap::ArgMatches, _ctx: &ToolContext) -> Result<u8, CliError> {
            Ok(0)
        }
        fn version_change_blocker(&self) -> Option<&str> {
            self.blocker
        }
    }

    /// A stub tool with its call counts shared with the test; `blocker` reports a
    /// `version_change_blocker` while `Some`.
    fn stub(
        id: &'static str,
        label: &'static str,
        blocker: Option<&'static str>,
    ) -> (Box<dyn StudioTool>, Arc<Calls>) {
        let calls = Arc::new(Calls::default());
        (
            Box::new(StubTool {
                id,
                label,
                icon: "i",
                blocker,
                calls: Arc::clone(&calls),
            }),
            calls,
        )
    }

    /// A shell over its stub tools, the context the tools send requests through, the shared
    /// settings and each stub's call counts.
    struct Shell {
        app: StudioApp,
        ctx: ToolContext,
        settings: Arc<Mutex<Settings>>,
        calls: Vec<Arc<Calls>>,
    }

    /// A shell over `tools` with `settings` (written through the shared handle), `planned`
    /// as the sidebar's registry and `data_dir` as the settings file's folder.
    fn shell_with(
        settings: Settings,
        tools: Vec<Box<dyn StudioTool>>,
        planned: &'static [PlannedTool],
        data_dir: Option<PathBuf>,
        calls: Vec<Arc<Calls>>,
    ) -> Shell {
        let (events_tx, _events_rx) = unbounded();
        let (requests_tx, requests_rx) = unbounded();
        let settings = Arc::new(Mutex::new(settings));
        let ctx = ToolContext::new(
            Arc::clone(&settings),
            AppPaths {
                exe_dir: PathBuf::from("exe"),
                data_dir,
            },
            events_tx,
            requests_tx,
        );
        Shell {
            app: StudioApp::new(tools, planned, ctx.clone(), requests_rx),
            ctx,
            settings,
            calls,
        }
    }

    /// A shell over the stub tools `a` (labeled `A`) and `b` (`B`).
    fn shell() -> Shell {
        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        shell_with(
            test_settings(&test_dir("studio_core_shell")),
            vec![a, b],
            &[],
            None,
            vec![calls_a, calls_b],
        )
    }

    /// A temp folder unique to this test process, as `settings`' `test_dir` does it.
    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{name}_{}", std::process::id()))
    }

    /// The settings a shell test starts from: the defaults, except `pes_folder_path`
    /// under `dir` so the exe check never lists the developer's real install
    /// (CONTRIBUTING.md "Testing": a setting whose default finds something on the machine
    /// is set to a path under the test's own folder).
    fn test_settings(dir: &Path) -> Settings {
        let mut settings = Settings::default();
        settings.common.pes_folder_path = dir.display().to_string();
        settings
    }

    /// What one headless frame produced: the frame's AccessKit nodes and its platform
    /// output (clipboard, opened links).
    struct FrameOutput {
        nodes: Vec<accesskit::Node>,
        platform: egui::PlatformOutput,
    }

    /// One headless frame of the shell on an 800 by 600 screen with `events` as its input
    /// and `focused` as the window's focus: `logic` first, then `ui`, as eframe calls them.
    /// The context must have AccessKit enabled.
    fn frame(
        ctx: &egui::Context,
        app: &mut StudioApp,
        events: Vec<egui::Event>,
        focused: bool,
    ) -> FrameOutput {
        // eframe's own constructor of a `Frame` for testing an `App` without a window.
        app.logic(ctx, &mut eframe::Frame::_new_kittest());
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            focused,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()));
        let update = output
            .platform_output
            .accesskit_update
            .take()
            .expect("AccessKit is enabled on the context");
        let platform = std::mem::take(&mut output.platform_output);
        // Nothing renders here, so the frame's texture deltas are dropped on purpose.
        output.drop_without_applying_deltas();
        FrameOutput {
            nodes: update.nodes.into_iter().map(|(_, node)| node).collect(),
            platform,
        }
    }

    /// The widget labeled `label`.
    fn entry<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| node.label() == Some(label))
            .unwrap_or_else(|| panic!("no widget labeled {label:?}"))
    }

    /// The centre of `node`'s bounds.
    fn centre(node: &accesskit::Node) -> egui::Pos2 {
        let bounds = node.bounds().expect("a widget has bounds");
        egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        )
    }

    /// The primary button pressed (or released) at the centre of `node`'s bounds, with the
    /// pointer moved there first. egui reports the click on the frame of the release.
    fn pointer(node: &accesskit::Node, pressed: bool) -> Vec<egui::Event> {
        let pos = centre(node);
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            },
        ]
    }

    /// The pointer moved to the centre of `node` without a button: a hover.
    fn hover(node: &accesskit::Node) -> Vec<egui::Event> {
        vec![egui::Event::PointerMoved(centre(node))]
    }

    /// A click (press frame, release frame) on `node`, returning the release frame: the one
    /// `clicked()` lands in, so the one whose platform output carries the click's commands.
    fn click(ctx: &egui::Context, app: &mut StudioApp, node: &accesskit::Node) -> FrameOutput {
        let press = pointer(node, true);
        let release = pointer(node, false);
        frame(ctx, app, press, true);
        frame(ctx, app, release, true)
    }

    /// The PES version selector's node.
    fn combo(nodes: &[accesskit::Node]) -> &accesskit::Node {
        nodes
            .iter()
            .find(|node| node.role() == Role::ComboBox)
            .expect("the selector is drawn")
    }

    /// `n` frames with no input, returning the last one's output. egui's clock advances by
    /// the predicted delta each frame; ten frames clear the post-click tooltip cooldown
    /// (`clicked_more_recently_than_moved`, about 100 ms).
    fn idle(ctx: &egui::Context, app: &mut StudioApp, n: usize) -> FrameOutput {
        let mut out = frame(ctx, app, Vec::new(), true);
        for _ in 1..n {
            out = frame(ctx, app, Vec::new(), true);
        }
        out
    }

    /// A node carries `needle` as its text: label-role nodes (plain text, tooltips, the
    /// planned entries) keep theirs in `value`.
    fn has_text(nodes: &[accesskit::Node], needle: &str) -> bool {
        nodes
            .iter()
            .any(|node| node.value().is_some_and(|text| text.contains(needle)))
    }

    #[test]
    fn a_switch_request_makes_the_named_tool_active() {
        let Shell { mut app, ctx, .. } = shell();
        assert_eq!(app.active, 0);
        ctx.switch_to_tool("b");
        app.apply_requests();
        assert_eq!(app.active, 1);
    }

    #[test]
    fn a_switch_to_an_unknown_tool_changes_nothing() {
        let Shell { mut app, ctx, .. } = shell();
        ctx.switch_to_tool("b");
        ctx.switch_to_tool("zz");
        app.apply_requests();
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
        app.apply_requests();
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

    // TC-GUI-03
    #[test]
    fn the_sidebar_marks_the_active_tool_and_a_click_on_an_entry_switches_to_it() {
        let Shell { mut app, calls, .. } = shell();
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert_eq!(
            (calls[0].views(), calls[1].views()),
            (1, 0),
            "only the active view runs"
        );
        assert_eq!(entry(&nodes, "A").toggled(), Some(Toggled::True));
        assert_eq!(entry(&nodes, "B").toggled(), Some(Toggled::False));

        // egui hit-tests a click against the previous frame's widgets: the press comes a frame
        // after the layout, the release a frame after the press.
        let press = pointer(entry(&nodes, "B"), true);
        let release = pointer(entry(&nodes, "B"), false);
        frame(&egui_ctx, &mut app, press, true);
        frame(&egui_ctx, &mut app, release, true);
        assert_eq!(app.active, 1);
        assert_eq!(
            (calls[0].views(), calls[1].views()),
            (2, 1),
            "B's view ran on the click frame"
        );

        // The sidebar is drawn before the click is applied, so the entries show the switch on
        // the next frame.
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert_eq!(entry(&nodes, "A").toggled(), Some(Toggled::False));
        assert_eq!(entry(&nodes, "B").toggled(), Some(Toggled::True));
        assert_eq!((calls[0].views(), calls[1].views()), (2, 2));
    }

    // ST-SIDE-01
    #[test]
    fn the_gui_opens_on_the_tool_last_tool_names_and_lists_all_tools() {
        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let (c, calls_c) = stub("c", "C", None);
        let mut settings = test_settings(&test_dir("studio_core_sidebar_last_named"));
        settings.common.last_tool = Some("b".to_owned());
        let Shell { mut app, calls, .. } = shell_with(
            settings,
            vec![a, b, c],
            &[],
            None,
            vec![calls_a, calls_b, calls_c],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert_eq!(app.active, 1, "last_tool names the second tool");
        assert_eq!(
            (calls[0].views(), calls[1].views(), calls[2].views()),
            (0, 1, 0),
            "only the second tool's view is drawn"
        );
        // The full labels, in registration order (drawn top to bottom).
        let top = |label: &str| entry(&nodes, label).bounds().unwrap().y0;
        assert!(top("A") < top("B") && top("B") < top("C"), "registry order");
        assert_eq!(entry(&nodes, "B").toggled(), Some(Toggled::True));
    }

    // ST-SIDE-02
    #[test]
    fn the_collapse_button_turns_the_sidebar_into_a_working_icon_rail() {
        let Shell { mut app, calls, .. } = shell();
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();
        egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

        // Expanded: the entries are full-width rows.
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let expanded_width = entry(&nodes, "A").bounds().unwrap().width();
        assert!(expanded_width > 60.0, "expanded entries span the sidebar");

        click(&egui_ctx, &mut app, entry(&nodes, "«"));
        idle(&egui_ctx, &mut app, 10);
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let rail_width = entry(&nodes, "A").bounds().unwrap().width();
        assert!(
            rail_width < expanded_width / 2.0,
            "collapsed entries shrink to the icon ({rail_width} < {})",
            expanded_width / 2.0
        );

        // The rail entry's tooltip is its label.
        frame(&egui_ctx, &mut app, hover(entry(&nodes, "B")), true);
        // `is_still` takes about 0.1 s of frames with no pointer movement.
        let nodes = idle(&egui_ctx, &mut app, 8).nodes;
        assert!(
            has_text(&nodes, "B"),
            "the rail entry's tooltip carries the label"
        );

        // A click on a rail entry still switches the tool.
        click(&egui_ctx, &mut app, entry(&nodes, "B"));
        assert_eq!(app.active, 1);
        assert!(calls[1].views() >= 1, "B's view runs");

        // A second click on the collapse button restores the labels.
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "»"));
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert_eq!(
            entry(&nodes, "A").bounds().unwrap().width(),
            expanded_width,
            "the labels are back"
        );
    }

    // ST-SIDE-03
    #[test]
    fn the_selector_is_colored_by_the_exe_found_in_the_pes_folder() {
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();
        egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

        // A shell at PES 21 with `data_dir` unset; `pes_folder_path` is the given folder.
        let with_pes_folder = |folder: &PathBuf| {
            let (a, calls_a) = stub("a", "A", None);
            let mut settings = Settings::default();
            settings.common.pes_version = PesVersion::Pes21;
            settings.common.pes_folder_path = folder.display().to_string();
            shell_with(settings, vec![a], &[], None, vec![calls_a])
        };
        let hover_selector = |app: &mut StudioApp| {
            let nodes = frame(&egui_ctx, app, Vec::new(), true).nodes;
            frame(&egui_ctx, app, hover(combo(&nodes)), true);
            // `is_still` takes about 0.1 s of frames with no pointer movement.
            idle(&egui_ctx, app, 8).nodes
        };
        let visuals = egui::Visuals::dark();
        let color_of =
            |app: &StudioApp| selector_color(app.sidebar.exe_check.as_ref().unwrap().2, &visuals);

        // Another version's exe: red, the tooltip names what was found.
        let dir = test_dir("studio_core_sidebar_pes19");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("PES2019.exe"), b"").unwrap();
        let Shell { mut app, .. } = with_pes_folder(&dir);
        frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(
            app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
            Some(ExeCheck::OtherVersion(PesVersion::Pes19))
        );
        assert_eq!(color_of(&app), Some(visuals.error_fg_color), "red");
        let nodes = hover_selector(&mut app);
        let expected = format!("Found PES2019.exe in {}", dir.display());
        assert!(
            has_text(&nodes, &expected),
            "the tooltip names the exe found"
        );
        fs::remove_dir_all(&dir).unwrap();

        // No PES exe: yellow, the tooltip says none was found.
        let dir = test_dir("studio_core_sidebar_noexe");
        fs::create_dir_all(&dir).unwrap();
        let Shell { mut app, .. } = with_pes_folder(&dir);
        frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(
            app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
            Some(ExeCheck::NoExe)
        );
        assert_eq!(color_of(&app), Some(visuals.warn_fg_color), "yellow");
        let nodes = hover_selector(&mut app);
        assert!(has_text(
            &nodes,
            &format!("No PES exe in {}", dir.display())
        ));
        fs::remove_dir_all(&dir).unwrap();

        // A directory named like the exe is not an exe (the compiler's preflight
        // counts files only): yellow like an empty folder.
        let dir = test_dir("studio_core_sidebar_dir_named_exe");
        fs::create_dir_all(dir.join("PES2021.exe")).unwrap();
        let Shell { mut app, .. } = with_pes_folder(&dir);
        frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(
            app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
            Some(ExeCheck::NoExe)
        );
        assert_eq!(color_of(&app), Some(visuals.warn_fg_color), "yellow");
        fs::remove_dir_all(&dir).unwrap();

        // The folder missing: yellow too, the tooltip names it.
        let dir = test_dir("studio_core_sidebar_missing");
        let Shell { mut app, .. } = with_pes_folder(&dir);
        frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(
            app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
            Some(ExeCheck::NoFolder)
        );
        assert_eq!(color_of(&app), Some(visuals.warn_fg_color), "yellow");
        let nodes = hover_selector(&mut app);
        assert!(has_text(
            &nodes,
            &format!("{} does not exist", dir.display())
        ));

        // The selected version's exe: no colour.
        let dir = test_dir("studio_core_sidebar_pes21");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("PES2021.exe"), b"").unwrap();
        let Shell { mut app, .. } = with_pes_folder(&dir);
        frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(
            app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
            Some(ExeCheck::Found)
        );
        assert_eq!(color_of(&app), None, "no colour");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_exe_check_relists_on_a_change_or_on_focus_never_per_frame() {
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();
        let dir = test_dir("studio_core_sidebar_recheck");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("PES2021.exe"), b"").unwrap();

        let (a, calls_a) = stub("a", "A", None);
        let mut settings = Settings::default();
        settings.common.pes_version = PesVersion::Pes21;
        settings.common.pes_folder_path = dir.display().to_string();
        let Shell {
            mut app, settings, ..
        } = shell_with(settings, vec![a], &[], None, vec![calls_a]);

        frame(&egui_ctx, &mut app, Vec::new(), true);
        frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(app.sidebar.exe_listings, 1, "no re-list per frame");

        // The window's focus returning re-lists.
        frame(&egui_ctx, &mut app, Vec::new(), false);
        frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(app.sidebar.exe_listings, 2, "focus regained re-lists");

        // A version or path change re-lists.
        settings.lock().unwrap().common.pes_version = PesVersion::Pes19;
        frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(app.sidebar.exe_listings, 3, "the version changed");
        fs::remove_dir_all(&dir).unwrap();
    }

    // ST-SIDE-04
    #[test]
    fn a_version_change_blocker_disables_the_selector_and_a_pick_saves_the_version() {
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();
        egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

        // While a tool blocks, the selector is disabled and its tooltip is the reason.
        let (a, calls_a) = stub("a", "A", Some("a compile is running"));
        let (b, calls_b) = stub("b", "B", None);
        let dir = test_dir("studio_core_sidebar_blocked");
        fs::create_dir_all(&dir).unwrap();
        let Shell {
            mut app, settings, ..
        } = shell_with(
            test_settings(&dir),
            vec![a, b],
            &[],
            Some(dir.clone()),
            vec![calls_a, calls_b],
        );
        let version_before = settings.lock().unwrap().common.pes_version;
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert!(combo(&nodes).is_disabled(), "the selector is disabled");
        frame(&egui_ctx, &mut app, hover(combo(&nodes)), true);
        let nodes = idle(&egui_ctx, &mut app, 8).nodes;
        assert!(
            has_text(&nodes, "A: a compile is running"),
            "the tooltip is the blocking tool's reason"
        );
        assert_eq!(
            settings.lock().unwrap().common.pes_version,
            version_before,
            "the setting keeps its value"
        );

        // Unblocked: a pick writes the settings file and the tools read it next frame.
        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let Shell {
            mut app,
            settings,
            calls,
            ..
        } = shell_with(
            test_settings(&dir),
            vec![a, b],
            &[],
            Some(dir.clone()),
            vec![calls_a, calls_b],
        );
        assert_ne!(
            Settings::default().common.pes_version,
            PesVersion::Pes21,
            "the test's pick really changes the setting"
        );
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, combo(&nodes));
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "PES 2021"));
        assert_eq!(
            settings.lock().unwrap().common.pes_version,
            PesVersion::Pes21
        );
        // The save is applied at the next frame's logic.
        frame(&egui_ctx, &mut app, Vec::new(), true);
        let written = Settings::load(&dir.join(crate::settings::SETTINGS_FILE_NAME)).unwrap();
        assert_eq!(written.common.pes_version, PesVersion::Pes21);
        assert_eq!(calls[0].version_seen(), Some(PesVersion::Pes21));
        assert_eq!(calls[1].version_seen(), Some(PesVersion::Pes21));
        fs::remove_dir_all(&dir).unwrap();
    }

    // ST-SIDE-07
    #[test]
    fn the_planned_tools_follow_the_registered_ones_grayed_and_inert() {
        const PLANNED: &[PlannedTool] = &[
            PlannedTool {
                id: "a",
                label: "A",
                icon: "a",
            },
            PlannedTool {
                id: "planned-one",
                label: "Planned one",
                icon: "p",
            },
            PlannedTool {
                id: "planned-two",
                label: "Planned two",
                icon: "q",
            },
        ];
        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let Shell { mut app, .. } = shell_with(
            test_settings(&test_dir("studio_core_sidebar_planned")),
            vec![a, b],
            PLANNED,
            None,
            vec![calls_a, calls_b],
        );
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();
        egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        // The planned entries are label-role nodes: the text sits in `value`.
        let planned = |label: &str| {
            nodes
                .iter()
                .find(|node| node.value() == Some(label))
                .unwrap_or_else(|| panic!("no text node {label:?}"))
        };
        let top = |node: &accesskit::Node| node.bounds().unwrap().y0;
        assert!(
            top(entry(&nodes, "B")) < top(planned("Planned one"))
                && top(planned("Planned one")) < top(planned("Planned two")),
            "the planned entries follow the registered ones, in registry order"
        );
        assert!(
            planned("Planned one").is_disabled(),
            "the planned entry is grayed"
        );

        // Not clickable: a click changes nothing.
        click(&egui_ctx, &mut app, planned("Planned one"));
        assert_eq!(app.active, 0);

        // Its tooltip says the tool is not in this build.
        idle(&egui_ctx, &mut app, 10);
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let node = nodes
            .iter()
            .find(|node| node.value() == Some("Planned one"))
            .unwrap();
        frame(&egui_ctx, &mut app, hover(node), true);
        let nodes = idle(&egui_ctx, &mut app, 8).nodes;
        assert!(
            has_text(&nodes, "Planned one is not in this build"),
            "the tooltip names the tool and the build"
        );
    }

    // The logo's About block and Copy (ST-SIDE-06's help link arrives with the help window).
    #[test]
    fn the_logo_opens_about_and_copy_puts_the_version_block_on_the_clipboard() {
        let dir = test_dir("studio_core_sidebar_about");
        fs::create_dir_all(&dir).unwrap();
        let (a, calls_a) = stub("a", "A", None);
        let mut settings = test_settings(&dir);
        settings.common.pes_version = PesVersion::Pes21;
        let Shell { mut app, .. } =
            shell_with(settings, vec![a], &[], Some(dir.clone()), vec![calls_a]);
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "4cc"));

        let out = frame(&egui_ctx, &mut app, Vec::new(), true);
        let block = format!(
            "4cc Studio v{}\nBuild: {}\nTarget: {}\nData location: {}\nPES version: PES 2021",
            env!("CARGO_PKG_VERSION"),
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            env!("STUDIO_TARGET_TRIPLE"),
            dir.display(),
        );
        assert!(
            out.nodes
                .iter()
                .any(|node| node.value() == Some(block.as_str())),
            "the version block is shown"
        );
        entry(&out.nodes, "Release page");
        entry(&out.nodes, "Repository");

        // The links open their URLs.
        let out = click(&egui_ctx, &mut app, entry(&out.nodes, "Repository"));
        assert!(
            out.platform
                .commands
                .contains(&egui::OutputCommand::OpenUrl(egui::OpenUrl::same_tab(
                    about::REPOSITORY_URL
                ))),
            "the repository link opens {}",
            env!("CARGO_PKG_REPOSITORY")
        );

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let out = click(&egui_ctx, &mut app, entry(&nodes, "Release page"));
        assert!(
            out.platform
                .commands
                .contains(&egui::OutputCommand::OpenUrl(egui::OpenUrl::same_tab(
                    about::RELEASE_PAGE_URL
                ))),
            "the release-page link opens the releases listing"
        );

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let out = click(&egui_ctx, &mut app, entry(&nodes, "Copy"));
        assert!(
            out.platform
                .commands
                .contains(&egui::OutputCommand::CopyText(block)),
            "Copy puts exactly the version block on the clipboard"
        );

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Close"));
        assert!(!app.sidebar.about_open, "Close dismisses the popup");
        fs::remove_dir_all(&dir).unwrap();
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
