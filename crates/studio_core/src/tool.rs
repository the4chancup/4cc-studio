//! The tool plugin interface. Tools are compile-time plugins: each tool crate implements
//! `StudioTool`, and the `studio` binary registers them in a static list (core plan, "Tool plugin
//! interface"). `ToolContext` is the tool's one handle on the platform: settings, the event sink,
//! and requests to the shell (switch tool, notify).

use std::path::PathBuf;
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
    fn tick(&mut self, _ctx: &ToolContext) {}

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

    /// Runs the already-parsed CLI subcommand headless. `Ok` carries the process exit code of a
    /// command that reached a verdict (its findings went out as events); `Err` is a command
    /// refused or failed before one, printed by the binary as `error: …`.
    fn cli_run(&self, matches: &clap::ArgMatches, ctx: &ToolContext) -> Result<u8, CliError>;

    /// GUI autorun for `4cc-studio --gui <tool-id> <command>`: performs the parsed command inside the
    /// GUI as if the user had pressed the corresponding button. Called once after the tool's view
    /// exists; the tool may queue the action until its own readiness condition holds. Default: no
    /// command is GUI-runnable, so the launch fails with a message instead of silently opening.
    fn gui_run(&mut self, _matches: &clap::ArgMatches, _ctx: &ToolContext) -> anyhow::Result<()> {
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

/// A CLI command refused or failed before it reached a verdict.
#[derive(Debug, thiserror::Error)]
#[error("{error:#}")]
pub struct CliError {
    /// The process exit code: the tool's own mapping (the Team compiler's 2 or 3).
    pub exit_code: u8,
    /// What went wrong, printed as `error: …`.
    pub error: anyhow::Error,
}

impl CliError {
    /// A refusal with the tool's exit code for it.
    pub fn new(exit_code: u8, error: impl Into<anyhow::Error>) -> CliError {
        CliError {
            exit_code,
            error: error.into(),
        }
    }
}

/// The base folders relative path settings resolve against (`team_compiler/settings.md`
/// "Path resolution").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// The folder holding the executable.
    pub exe_dir: PathBuf,
    /// The selected data directory; `None` when no settings file exists yet.
    pub data_dir: Option<PathBuf>,
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
    paths: AppPaths,
    events: Sender<PipelineEventEnvelope>,
    requests: Sender<ShellRequest>,
}

impl ToolContext {
    /// Builds a context over the shell's settings, the app's base folders and its two receiving
    /// channels.
    pub fn new(
        settings: Arc<Mutex<Settings>>,
        paths: AppPaths,
        events: Sender<PipelineEventEnvelope>,
        requests: Sender<ShellRequest>,
    ) -> Self {
        ToolContext {
            settings,
            paths,
            events,
            requests,
        }
    }

    /// The same handle (settings, paths, shell requests) with its events sent to `events`. A
    /// tool whose view shows a run's events gives the run its own sink this way and drains the
    /// receiver in `tick`: an envelope names no tool, so events routed through the shell's
    /// receiver could not be told apart by tool.
    pub fn with_events(&self, events: Sender<PipelineEventEnvelope>) -> ToolContext {
        ToolContext {
            events,
            ..self.clone()
        }
    }

    /// The two base folders relative path settings resolve against: the executable's folder and
    /// the data directory (`None` before a settings file exists, as in a CLI run ahead of the
    /// first GUI start). Tools never see the settings file itself.
    pub fn paths(&self) -> &AppPaths {
        &self.paths
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

#[cfg(test)]
mod tests {
    use crossbeam_channel::{Receiver, unbounded};

    use super::*;
    use crate::events::{PipelineEvent, RunId};

    /// A context over fresh channels, with the receivers it sends to.
    fn context() -> (
        ToolContext,
        Receiver<PipelineEventEnvelope>,
        Receiver<ShellRequest>,
    ) {
        let (events_tx, events_rx) = unbounded();
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
        (ctx, events_rx, requests_rx)
    }

    fn envelope() -> PipelineEventEnvelope {
        PipelineEventEnvelope {
            run_id: RunId(1),
            export_id: None,
            export_revision: None,
            event: PipelineEvent::UnmigratedContentFound,
        }
    }

    #[test]
    fn common_returns_the_settings_common_section_as_it_is() {
        let mut settings = Settings::default();
        settings.common.thread_count = 4;
        let (events_tx, _events_rx) = unbounded();
        let (requests_tx, _requests_rx) = unbounded();
        let ctx = ToolContext::new(
            Arc::new(Mutex::new(settings)),
            AppPaths {
                exe_dir: PathBuf::from("exe"),
                data_dir: None,
            },
            events_tx,
            requests_tx,
        );
        assert_eq!(ctx.common().thread_count, 4);
    }

    #[test]
    fn with_events_sends_events_to_the_new_receiver_only() {
        let (ctx, events_rx, _requests_rx) = context();
        let (own_tx, own_rx) = unbounded();
        ctx.with_events(own_tx).emit(envelope());
        assert_eq!(own_rx.try_recv().unwrap(), envelope());
        assert!(
            events_rx.try_recv().is_err(),
            "the original receiver got nothing"
        );
    }

    #[test]
    fn with_events_keeps_the_shell_requests_and_the_settings() {
        let (ctx, _events_rx, requests_rx) = context();
        let (own_tx, _own_rx) = unbounded();
        let run_ctx = ctx.with_events(own_tx);
        run_ctx.switch_to_tool("stub");
        assert_eq!(
            requests_rx.try_recv().unwrap(),
            ShellRequest::SwitchTool("stub".to_owned())
        );
        let mut table = toml::Table::new();
        table.insert("key".to_owned(), toml::Value::Integer(1));
        run_ctx.set_tool_settings("stub", table.clone());
        assert_eq!(ctx.tool_settings("stub"), table);
        assert_eq!(
            requests_rx.try_recv().unwrap(),
            ShellRequest::SettingsChanged
        );
    }
}
