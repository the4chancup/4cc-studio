//! The tool plugin interface. Tools are compile-time plugins: each tool crate implements
//! `StudioTool`, and the `studio` binary registers them in a static list (core plan, "Tool plugin
//! interface"). `ToolContext` is the tool's one handle on the platform: settings, the event sink,
//! and requests to the shell (switch tool, notify).

use std::sync::{Arc, Mutex};

use crossbeam_channel::Sender;
use log::debug;

use crate::events::PipelineEventEnvelope;
use crate::help::HelpSection;
use crate::settings::{CommonSettings, Settings};
use crate::status::{Notice, ToolActivity};

/// A tool of the suite. Implemented once per tool crate, by its `Tool` type.
pub trait StudioTool {
    /// Stable identifier, used for CLI dispatch and as the settings key (`team-compiler`).
    fn id(&self) -> &'static str;

    /// Sidebar label (`Team compiler`).
    fn label(&self) -> &'static str;

    /// The tool's main view, rendered in the panel right of the tool selector.
    fn view(&mut self, ui: &mut egui::Ui, ctx: &ToolContext);

    /// Per-frame background work, called for every registered tool each frame, active view or
    /// not: drain event channels, check timer deadlines. Default no-op.
    fn tick(&mut self, ctx: &ToolContext) {
        let _ = ctx;
    }

    /// The tool's settings section, injected into the settings menu.
    fn settings_view(&mut self, ui: &mut egui::Ui);

    /// Defaults merged into the tool's settings table for every key it lacks, on first run and
    /// when a new version adds a key.
    fn default_settings(&self) -> toml::Table;

    /// The tool's chapter of the help window: prose topics only; the window appends the message
    /// catalog as a generated topic.
    fn help(&self) -> HelpSection;

    /// The tool's CLI subcommand. Its name must equal `id()`; the shell enforces it.
    fn cli_command(&self) -> clap::Command;

    /// Runs the already-parsed CLI subcommand headless.
    fn cli_run(&self, matches: &clap::ArgMatches, ctx: &ToolContext) -> anyhow::Result<()>;

    /// GUI autorun for `studio --gui <tool-id> <command>`: performs the parsed command inside the
    /// GUI as if the user had pressed the corresponding button. Called once after the tool's view
    /// exists; the tool may queue the action until its own readiness condition holds. Default: no
    /// command is GUI-runnable, so the launch fails with a message instead of silently opening.
    fn gui_run(&mut self, matches: &clap::ArgMatches, ctx: &ToolContext) -> anyhow::Result<()> {
        let _ = (matches, ctx);
        anyhow::bail!("{} has no GUI-runnable commands", self.id())
    }

    /// A short reason the global PES version cannot change right now (the match tracker holding
    /// live memory addresses), shown as the selector's disabled-state tooltip. Default: none.
    fn version_change_blocker(&self) -> Option<&str> {
        None
    }

    /// What the tool is doing in the background, for the status bar and the window title.
    /// Default: nothing.
    fn activity(&self) -> Option<ToolActivity> {
        None
    }
}

/// Something a tool asks the shell to do at the end of the frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellRequest {
    /// Make the named tool the active view.
    SwitchTool(String),
    /// Show a notice in the status bar.
    Notify(Notice),
    /// A tool changed its settings section; save the file (best-effort).
    SettingsChanged,
}

/// The tool's handle on the platform. Cheap to clone; every clone shares the same settings and
/// channels.
#[derive(Debug, Clone)]
pub struct ToolContext {
    settings: Arc<Mutex<Settings>>,
    events: Sender<PipelineEventEnvelope>,
    requests: Sender<ShellRequest>,
}

impl ToolContext {
    /// Builds a context over the shell's settings and its two receiving channels.
    pub fn new(
        settings: Arc<Mutex<Settings>>,
        events: Sender<PipelineEventEnvelope>,
        requests: Sender<ShellRequest>,
    ) -> Self {
        ToolContext {
            settings,
            events,
            requests,
        }
    }

    /// A copy of the common settings as they are now.
    pub fn common(&self) -> CommonSettings {
        self.settings.lock().unwrap().common.clone()
    }

    /// A copy of the tool's own settings table; empty if the file has no section for it yet.
    pub fn tool_settings(&self, tool_id: &str) -> toml::Table {
        self.settings
            .lock()
            .unwrap()
            .tool(tool_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Replaces the tool's settings table and asks the shell to save.
    pub fn set_tool_settings(&self, tool_id: &str, table: toml::Table) {
        self.settings.lock().unwrap().set_tool(tool_id, table);
        self.request(ShellRequest::SettingsChanged);
    }

    /// Sends a pipeline event. A receiver that has gone away (the CLI printer finished, the GUI
    /// closed) is not an error for the sender, so the failure is logged here, once, and dropped.
    pub fn emit(&self, envelope: PipelineEventEnvelope) {
        if self.events.send(envelope).is_err() {
            debug!("pipeline event dropped: no receiver");
        }
    }

    /// Asks the shell to make the named tool the active view at the end of the frame. Id-based,
    /// so tools can link to each other without depending on each other's crates.
    pub fn switch_to_tool(&self, tool_id: &str) {
        self.request(ShellRequest::SwitchTool(tool_id.to_owned()));
    }

    /// Publishes a notice to the status bar: the one way a tool puts something there.
    pub fn notify(&self, notice: Notice) {
        self.request(ShellRequest::Notify(notice));
    }

    fn request(&self, request: ShellRequest) {
        if self.requests.send(request).is_err() {
            debug!("shell request dropped: no receiver");
        }
    }
}
