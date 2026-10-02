//! The app shell: `StudioApp`, the window with a sidebar listing the registered tools and the
//! active tool's view, every tool ticked each frame and the tools' shell requests applied; and
//! the launch modes' argument parsing (`launch`). The status bar, the settings menu, the help
//! window and the common widgets join this module in the GUI phase (core plan, "Shell layout",
//! "Crate structure").

pub mod launch;

use crossbeam_channel::Receiver;
use log::debug;

use crate::tool::{ShellRequest, StudioTool, ToolContext};

/// The window: the registered tools in sidebar order, the platform handle they share, the
/// receiving end of their shell requests, and which tool's view is on screen.
pub struct StudioApp {
    tools: Vec<Box<dyn StudioTool>>,
    ctx: ToolContext,
    requests: Receiver<ShellRequest>,
    /// Index into `tools` of the active view: the first tool at launch (reopening the last
    /// used one needs the settings save, which arrives with the GUI phase).
    active: usize,
}

impl StudioApp {
    /// A shell over `tools`, opening on the first of them. `requests` is the receiving end of
    /// the channel `ctx` sends shell requests to.
    pub fn new(
        tools: Vec<Box<dyn StudioTool>>,
        ctx: ToolContext,
        requests: Receiver<ShellRequest>,
    ) -> StudioApp {
        StudioApp {
            tools,
            ctx,
            requests,
            active: 0,
        }
    }

    /// Applies every request the tools queued since the last frame.
    fn apply_requests(&mut self) {
        while let Ok(request) = self.requests.try_recv() {
            match request {
                ShellRequest::SwitchTool(id) => {
                    match self.tools.iter().position(|tool| tool.id() == id) {
                        Some(index) => self.active = index,
                        None => debug!("switch to unknown tool `{id}` ignored"),
                    }
                }
                // The status bar, which shows notices, and the settings save, which follows a
                // change, both arrive with the GUI phase; until then neither request has a
                // consumer.
                ShellRequest::Notify(_) | ShellRequest::SettingsChanged => {}
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
        let mut selected = self.active;
        egui::Panel::left("sidebar").show(ui, |ui| {
            for (index, tool) in self.tools.iter().enumerate() {
                if ui
                    .selectable_label(index == self.active, tool.label())
                    .clicked()
                {
                    selected = index;
                }
            }
        });
        self.active = selected;
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
        Box::new(|_creation| Ok(Box::new(app))),
    )
    .map_err(|error| anyhow::anyhow!("cannot open the window: {error}"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use crossbeam_channel::{TryRecvError, unbounded};
    use eframe::App;
    use egui::accesskit::{self, Toggled};

    use super::*;
    use crate::help::HelpSection;
    use crate::settings::Settings;
    use crate::status::Notice;
    use crate::tool::{AppPaths, CliError};

    /// How often the shell called a stub tool's `tick` and `view`, shared with the test.
    #[derive(Default)]
    struct Calls {
        ticks: AtomicUsize,
        views: AtomicUsize,
    }

    impl Calls {
        fn ticks(&self) -> usize {
            self.ticks.load(Ordering::SeqCst)
        }
        fn views(&self) -> usize {
            self.views.load(Ordering::SeqCst)
        }
    }

    struct StubTool {
        id: &'static str,
        label: &'static str,
        calls: Arc<Calls>,
    }

    impl StudioTool for StubTool {
        fn id(&self) -> &'static str {
            self.id
        }
        fn label(&self) -> &'static str {
            self.label
        }
        fn view(&mut self, _ui: &mut egui::Ui, _ctx: &ToolContext) {
            self.calls.views.fetch_add(1, Ordering::SeqCst);
        }
        fn tick(&mut self, _ctx: &ToolContext) {
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
    }

    /// A shell over the stub tools `a` (labeled `A`) and `b` (`B`), the context the tools send
    /// requests through, and each stub's call counts.
    struct Shell {
        app: StudioApp,
        ctx: ToolContext,
        a: Arc<Calls>,
        b: Arc<Calls>,
    }

    fn shell() -> Shell {
        let (events_tx, _events_rx) = unbounded();
        let (requests_tx, requests_rx) = unbounded();
        let ctx = ToolContext::new(
            Arc::new(Mutex::new(Settings::default())),
            AppPaths {
                exe_dir: PathBuf::from("exe"),
                data_dir: None,
            },
            events_tx,
            requests_tx,
        );
        let a = Arc::new(Calls::default());
        let b = Arc::new(Calls::default());
        let tools: Vec<Box<dyn StudioTool>> = vec![
            Box::new(StubTool {
                id: "a",
                label: "A",
                calls: Arc::clone(&a),
            }),
            Box::new(StubTool {
                id: "b",
                label: "B",
                calls: Arc::clone(&b),
            }),
        ];
        Shell {
            app: StudioApp::new(tools, ctx.clone(), requests_rx),
            ctx,
            a,
            b,
        }
    }

    /// One headless frame of the shell's `ui` on an 800 by 600 screen with `events` as its
    /// input; returns the frame's AccessKit nodes (the context must have AccessKit enabled).
    fn frame(
        ctx: &egui::Context,
        app: &mut StudioApp,
        events: Vec<egui::Event>,
    ) -> Vec<accesskit::Node> {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            ..Default::default()
        };
        // eframe's own constructor of a `Frame` for testing an `App` without a window.
        let mut output = ctx.run_ui(input, |ui| app.ui(ui, &mut eframe::Frame::_new_kittest()));
        let update = output
            .platform_output
            .accesskit_update
            .take()
            .expect("AccessKit is enabled on the context");
        // Nothing renders here, so the frame's texture deltas are dropped on purpose.
        output.drop_without_applying_deltas();
        update.nodes.into_iter().map(|(_, node)| node).collect()
    }

    /// The sidebar entry labeled `label`.
    fn entry<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| node.label() == Some(label))
            .unwrap_or_else(|| panic!("no widget labeled {label:?}"))
    }

    /// The primary button pressed (or released) at the centre of `node`'s bounds, with the
    /// pointer moved there first. egui reports the click on the frame of the release.
    fn pointer(node: &accesskit::Node, pressed: bool) -> Vec<egui::Event> {
        let bounds = node.bounds().expect("a widget has bounds");
        let pos = egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
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
        let Shell { mut app, ctx, a, b } = shell();
        ctx.switch_to_tool("b");
        // eframe's own constructor of a `Frame` for testing an `App` without a window.
        app.logic(
            &egui::Context::default(),
            &mut eframe::Frame::_new_kittest(),
        );
        assert_eq!((a.ticks(), b.ticks()), (1, 1));
        assert_eq!(app.active, 1);
    }

    #[test]
    fn the_sidebar_marks_the_active_tool_and_a_click_on_an_entry_switches_to_it() {
        let Shell { mut app, a, b, .. } = shell();
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        let nodes = frame(&egui_ctx, &mut app, Vec::new());
        assert_eq!((a.views(), b.views()), (1, 0), "only the active view runs");
        assert_eq!(entry(&nodes, "A").toggled(), Some(Toggled::True));
        assert_eq!(entry(&nodes, "B").toggled(), Some(Toggled::False));

        // egui hit-tests a click against the previous frame's widgets: the press comes a frame
        // after the layout, the release a frame after the press.
        let press = pointer(entry(&nodes, "B"), true);
        let release = pointer(entry(&nodes, "B"), false);
        frame(&egui_ctx, &mut app, press);
        frame(&egui_ctx, &mut app, release);
        assert_eq!(app.active, 1);
        assert_eq!(
            (a.views(), b.views()),
            (2, 1),
            "B's view ran on the click frame"
        );

        // The sidebar is drawn before the click is applied, so the entries show the switch on
        // the next frame.
        let nodes = frame(&egui_ctx, &mut app, Vec::new());
        assert_eq!(entry(&nodes, "A").toggled(), Some(Toggled::False));
        assert_eq!(entry(&nodes, "B").toggled(), Some(Toggled::True));
        assert_eq!((a.views(), b.views()), (2, 2));
    }
}
