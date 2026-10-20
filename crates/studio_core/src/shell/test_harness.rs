//! The headless shell harness: stub tools, a `StudioApp` over them and the helpers that drive
//! frames, read the AccessKit tree and click widgets (core plan, "GUI Design" "Acceptance":
//! what a scenario observes is what a frame draws).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossbeam_channel::{Receiver, unbounded};
use eframe::App;
use egui::accesskit::{self, Role};
use pes_version::PesVersion;

use super::StudioApp;
use crate::events::{PipelineEvent, PipelineEventEnvelope, RunId};
use crate::help::HelpSection;
use crate::settings::Settings;
use crate::status::ToolActivity;
use crate::tool::{AppPaths, CliError, PlannedTool, StudioTool, ToolContext};

/// What `activity()` reports while a stub works: `(tool_label, progress, elapsed)`.
pub(super) type Activity = (&'static str, Option<(usize, usize)>, Duration);

/// How often the shell called a stub tool's `tick` and `view`, what its `tick` read and the
/// knobs a test turns mid-run: the blocker reason, the reported activity, a run's event
/// receiver for `tick` to drain and the table `default_settings()` reports.
#[derive(Default)]
pub(super) struct Calls {
    pub(super) ticks: AtomicUsize,
    pub(super) views: AtomicUsize,
    pub(super) version_seen: Mutex<Option<PesVersion>>,
    /// The `version_change_blocker` reason while `Some`; a test clears it mid-run.
    pub(super) blocker: Mutex<Option<&'static str>>,
    /// What `activity()` reports: `(tool_label, progress, elapsed)`, `None` while idle.
    pub(super) activity: Mutex<Option<Activity>>,
    /// A run's event receiver: `tick` drains it and any envelope ends the activity, as a
    /// run's last event does.
    pub(super) events: Mutex<Option<Receiver<PipelineEventEnvelope>>>,
    /// The table `default_settings()` reports; `stub()` fills a non-empty one, a test
    /// clears it for a tool without settings.
    pub(super) default_settings: Mutex<toml::Table>,
}

impl Calls {
    pub(super) fn ticks(&self) -> usize {
        self.ticks.load(Ordering::SeqCst)
    }
    pub(super) fn views(&self) -> usize {
        self.views.load(Ordering::SeqCst)
    }
    pub(super) fn version_seen(&self) -> Option<PesVersion> {
        *self.version_seen.lock().unwrap()
    }
    /// What the stub reports from `activity()`.
    pub(super) fn set_activity(
        &self,
        tool_label: &'static str,
        progress: Option<(usize, usize)>,
        elapsed: Duration,
    ) {
        *self.activity.lock().unwrap() = Some((tool_label, progress, elapsed));
    }
    /// Ends the reported activity, as a finished run does.
    pub(super) fn clear_activity(&self) {
        *self.activity.lock().unwrap() = None;
    }
}

pub(super) struct StubTool {
    id: &'static str,
    label: &'static str,
    icon: &'static str,
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
        // A run's receiver, when the test gave the stub one: any envelope ends the
        // activity, as a run's last event does.
        let events = self.calls.events.lock().unwrap();
        if let Some(rx) = events.as_ref()
            && rx.try_recv().is_ok()
        {
            while rx.try_recv().is_ok() {}
            self.calls.clear_activity();
        }
    }
    fn settings_view(&mut self, ui: &mut egui::Ui, _ctx: &ToolContext) {
        ui.label(format!("{} settings body", self.label));
    }
    fn default_settings(&self) -> toml::Table {
        self.calls.default_settings.lock().unwrap().clone()
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
        *self.calls.blocker.lock().unwrap()
    }
    fn activity(&self) -> Option<ToolActivity> {
        let (tool_label, progress, elapsed) = *self.calls.activity.lock().unwrap().as_ref()?;
        Some(ToolActivity {
            tool_label,
            progress,
            elapsed,
        })
    }
}

/// A stub tool with its call counts shared with the test; `blocker` reports a
/// `version_change_blocker` while `Some`.
pub(super) fn stub(
    id: &'static str,
    label: &'static str,
    blocker: Option<&'static str>,
) -> (Box<dyn StudioTool>, Arc<Calls>) {
    let calls = Arc::new(Calls::default());
    *calls.blocker.lock().unwrap() = blocker;
    *calls.default_settings.lock().unwrap() =
        toml::Table::from_iter([("enabled".to_owned(), toml::Value::Boolean(true))]);
    (
        Box::new(StubTool {
            id,
            label,
            icon: "i",
            calls: Arc::clone(&calls),
        }),
        calls,
    )
}

/// A shell over its stub tools, the context the tools send requests through, the shared
/// settings and each stub's call counts.
pub(super) struct Shell {
    pub(super) app: StudioApp,
    pub(super) ctx: ToolContext,
    pub(super) settings: Arc<Mutex<Settings>>,
    pub(super) calls: Vec<Arc<Calls>>,
}

/// A shell over `tools` with `settings` (written through the shared handle), `planned`
/// as the sidebar's registry and `data_dir` as the settings file's folder. The sidebar's
/// running-PES probe is stubbed to find nothing: a poll over the developer's real process
/// list must never leak into a test.
pub(super) fn shell_with(
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
    let mut app = StudioApp::new(tools, planned, ctx.clone(), requests_rx);
    app.sidebar.probe = || Vec::new();
    Shell {
        app,
        ctx,
        settings,
        calls,
    }
}

/// A shell over the stub tools `a` (labeled `A`) and `b` (`B`).
pub(super) fn shell() -> Shell {
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
pub(super) fn test_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{name}_{}", std::process::id()))
}

/// The settings a shell test starts from: the defaults, except `pes_folder_path`
/// under `dir` so the exe check never lists the developer's real install
/// (CONTRIBUTING.md "Testing": a setting whose default finds something on the machine
/// is set to a path under the test's own folder).
pub(super) fn test_settings(dir: &Path) -> Settings {
    let mut settings = Settings::default();
    settings.common.pes_folder_path = dir.display().to_string();
    settings
}

/// A run's smallest envelope, for a stub run's emit.
pub(super) fn envelope() -> PipelineEventEnvelope {
    PipelineEventEnvelope {
        run_id: RunId(1),
        export_id: None,
        export_revision: None,
        event: PipelineEvent::UnmigratedContentFound,
    }
}

/// What one headless frame produced: the frame's AccessKit nodes, its platform output
/// (clipboard, opened links), the root viewport's commands (the window title) and the
/// repaint delay it asked for (`Duration::MAX` when nothing did).
pub(super) struct FrameOutput {
    pub(super) nodes: Vec<accesskit::Node>,
    pub(super) platform: egui::PlatformOutput,
    pub(super) commands: Vec<egui::ViewportCommand>,
    pub(super) repaint_delay: Duration,
}

/// One headless frame of the shell on an 800 by 600 screen with `events` as its input
/// and `focused` as the window's focus: `logic` first, then `ui`, as eframe calls them
/// inside one pass, so both see this frame's input. The context must have AccessKit
/// enabled.
pub(super) fn frame(
    ctx: &egui::Context,
    app: &mut StudioApp,
    events: Vec<egui::Event>,
    focused: bool,
) -> FrameOutput {
    frame_at(ctx, app, events, focused, None)
}

/// `frame` with the clock set: `time` is `RawInput::time`, in seconds.
pub(super) fn frame_at(
    ctx: &egui::Context,
    app: &mut StudioApp,
    events: Vec<egui::Event>,
    focused: bool,
    time: Option<f64>,
) -> FrameOutput {
    run_frame(ctx, app, raw_input(events, focused, time, None))
}

/// `frame` with the OS's theme in `RawInput::system_theme`: the shell must not follow
/// it — the `theme` setting decides.
pub(super) fn frame_theme(
    ctx: &egui::Context,
    app: &mut StudioApp,
    events: Vec<egui::Event>,
    focused: bool,
    system_theme: Option<egui::Theme>,
) -> FrameOutput {
    run_frame(ctx, app, raw_input(events, focused, None, system_theme))
}

/// The `RawInput` every harness frame shares: an 800 by 600 screen.
fn raw_input(
    events: Vec<egui::Event>,
    focused: bool,
    time: Option<f64>,
    system_theme: Option<egui::Theme>,
) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        events,
        focused,
        time,
        system_theme,
        ..Default::default()
    }
}

/// One frame of `input`.
fn run_frame(ctx: &egui::Context, app: &mut StudioApp, input: egui::RawInput) -> FrameOutput {
    let mut output = ctx.run_ui(input, |ui| {
        // eframe's own constructor of a `Frame` for testing an `App` without a window.
        app.logic(ui.ctx(), &mut eframe::Frame::_new_kittest());
        app.ui(ui, &mut eframe::Frame::_new_kittest());
    });
    let update = output
        .platform_output
        .accesskit_update
        .take()
        .expect("AccessKit is enabled on the context");
    let platform = std::mem::take(&mut output.platform_output);
    let viewport = output
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .expect("every frame produces the root viewport");
    let commands = viewport.commands.clone();
    let repaint_delay = viewport.repaint_delay;
    // Nothing renders here, so the frame's texture deltas are dropped on purpose.
    output.drop_without_applying_deltas();
    FrameOutput {
        nodes: update.nodes.into_iter().map(|(_, node)| node).collect(),
        platform,
        commands,
        repaint_delay,
    }
}

/// The title strings the frame's viewport commands carry, in order.
pub(super) fn titles(output: &FrameOutput) -> Vec<String> {
    output
        .commands
        .iter()
        .filter_map(|command| match command {
            egui::ViewportCommand::Title(title) => Some(title.clone()),
            _ => None,
        })
        .collect()
}

/// The widget labeled `label`.
pub(super) fn entry<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.label() == Some(label))
        .unwrap_or_else(|| panic!("no widget labeled {label:?}"))
}

/// The centre of `node`'s bounds.
pub(super) fn centre(node: &accesskit::Node) -> egui::Pos2 {
    let bounds = node.bounds().expect("a widget has bounds");
    egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

/// The primary button pressed (or released) at the centre of `node`'s bounds, with the
/// pointer moved there first. egui reports the click on the frame of the release.
pub(super) fn pointer(node: &accesskit::Node, pressed: bool) -> Vec<egui::Event> {
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
pub(super) fn hover(node: &accesskit::Node) -> Vec<egui::Event> {
    vec![egui::Event::PointerMoved(centre(node))]
}

/// A click (press frame, release frame) on `node`, returning the release frame: the one
/// `clicked()` lands in, so the one whose platform output carries the click's commands.
pub(super) fn click(
    ctx: &egui::Context,
    app: &mut StudioApp,
    node: &accesskit::Node,
) -> FrameOutput {
    let press = pointer(node, true);
    let release = pointer(node, false);
    frame(ctx, app, press, true);
    frame(ctx, app, release, true)
}

/// The PES version selector's node.
pub(super) fn combo(nodes: &[accesskit::Node]) -> &accesskit::Node {
    nodes
        .iter()
        .find(|node| node.role() == Role::ComboBox)
        .expect("the selector is drawn")
}

/// `n` frames with no input, returning the last one's output. egui's clock advances by
/// the predicted delta each frame; ten frames clear the post-click tooltip cooldown
/// (`clicked_more_recently_than_moved`, about 100 ms).
pub(super) fn idle(ctx: &egui::Context, app: &mut StudioApp, n: usize) -> FrameOutput {
    let mut out = frame(ctx, app, Vec::new(), true);
    for _ in 1..n {
        out = frame(ctx, app, Vec::new(), true);
    }
    out
}

/// A node carries `needle` as its text: label-role nodes (plain text, tooltips, the
/// planned entries) keep theirs in `value`.
pub(super) fn has_text(nodes: &[accesskit::Node], needle: &str) -> bool {
    nodes
        .iter()
        .any(|node| node.value().is_some_and(|text| text.contains(needle)))
}
