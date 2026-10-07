//! The Team compiler's window, render-only (`team_compiler/README.md` "Crate layout"): it reads
//! the tool's state and reports the user's intent (Compile clicked); it never calls into the
//! pipeline itself. In Phase 3: the effective settings read-only, the Compile button and the
//! run log; the grid and the toolbar arrive with the GUI phase.

use std::time::Duration;

use studio_core::ToolContext;

use crate::gui_run::RunLog;
use crate::messages::TOOL_ID;

/// Shows the window; `true` when Compile was clicked this frame.
pub(crate) fn show(ui: &mut egui::Ui, ctx: &ToolContext, log: &RunLog) -> bool {
    ui.collapsing("Settings", |ui| show_settings(ui, ctx));
    let clicked = ui
        .add_enabled(!log.is_running(), egui::Button::new("Compile"))
        .clicked();
    if log.is_running() {
        // egui repaints only on input, and the run's events arrive on their own: without this
        // the log would advance only when the mouse moves.
        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in log.lines() {
                ui.monospace(line);
            }
        });
    clicked
}

/// The effective settings, as the settings file spells them: the common ones the compiler
/// reads, then the tool's own section.
fn show_settings(ui: &mut egui::Ui, ctx: &ToolContext) {
    let common = ctx.common();
    ui.monospace(format!("pes_version = {}", common.pes_version.number()));
    ui.monospace(format!("pes_folder_path = {}", common.pes_folder_path));
    ui.monospace(format!(
        "exports_folder_path = {}",
        common.exports_folder_path.display()
    ));
    for (key, value) in &ctx.tool_settings(TOOL_ID) {
        ui.monospace(format!("{key} = {value}"));
    }
    ui.label("Edit them in settings.toml; the settings menu comes later.");
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crossbeam_channel::unbounded;
    use egui::accesskit;
    use studio_core::StudioTool;

    use super::*;
    use crate::Tool;
    use crate::testing::{poll_until_done, sandbox, tool_context};

    /// A headless window: one egui context with AccessKit on and animations off (so a
    /// collapsing header is fully open on the frame after its click), run a frame at a time.
    struct Headless {
        ctx: egui::Context,
    }

    impl Headless {
        fn new() -> Headless {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            ctx.global_style_mut(|style| style.animation_time = 0.0);
            Headless { ctx }
        }

        /// Runs one frame of `contents` on an 800 by 600 screen with `events` as its input;
        /// returns what `contents` returned and the frame's AccessKit nodes.
        fn frame<R>(
            &mut self,
            events: Vec<egui::Event>,
            mut contents: impl FnMut(&mut egui::Ui) -> R,
        ) -> (R, Vec<accesskit::Node>) {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                events,
                ..Default::default()
            };
            let mut result = None;
            let mut output = self.ctx.run_ui(input, |ui| result = Some(contents(ui)));
            let update = output
                .platform_output
                .accesskit_update
                .take()
                .expect("AccessKit is enabled on the context");
            // Nothing renders here, so the frame's texture deltas are dropped on purpose.
            output.drop_without_applying_deltas();
            let nodes = update.nodes.into_iter().map(|(_, node)| node).collect();
            (result.expect("run_ui calls the closure"), nodes)
        }
    }

    /// The node of the button (or collapsing header) labeled `label`.
    fn button<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| node.label() == Some(label))
            .unwrap_or_else(|| panic!("no widget labeled {label:?}"))
    }

    /// Whether some label widget shows exactly `text`. egui puts a label's text in its
    /// AccessKit node's value, not its label.
    fn shows_text(nodes: &[accesskit::Node], text: &str) -> bool {
        nodes.iter().any(|node| node.value() == Some(text))
    }

    /// The centre of a node's bounds, where a click lands.
    fn centre(node: &accesskit::Node) -> egui::Pos2 {
        let bounds = node.bounds().expect("a widget has bounds");
        egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        )
    }

    /// The pointer moved to `pos` and the primary button went down there.
    fn press(pos: egui::Pos2) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            },
        ]
    }

    /// The primary button came up at `pos`: egui reports the click on this frame.
    fn release(pos: egui::Pos2) -> Vec<egui::Event> {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        }]
    }

    /// A context with no sandbox behind it, for frames that start no run.
    fn settings_only_context() -> ToolContext {
        tool_context(Path::new("exe"), "")
    }

    #[test]
    fn without_a_click_nothing_is_reported_and_compile_is_enabled() {
        let ctx = settings_only_context();
        let log = RunLog::default();
        let (clicked, nodes) = Headless::new().frame(Vec::new(), |ui| show(ui, &ctx, &log));
        assert!(!clicked);
        assert!(!button(&nodes, "Compile").is_disabled());
    }

    #[test]
    fn a_click_on_compile_is_reported_on_the_frame_of_the_release() {
        let ctx = settings_only_context();
        let log = RunLog::default();
        let mut window = Headless::new();
        // The first frame lays the button out; egui hit-tests a click against the previous
        // frame's widgets, so the press comes on the second frame and the release on the third.
        let (_, nodes) = window.frame(Vec::new(), |ui| show(ui, &ctx, &log));
        let target = centre(button(&nodes, "Compile"));
        let (clicked, _) = window.frame(press(target), |ui| show(ui, &ctx, &log));
        assert!(!clicked, "not before the release");
        let (clicked, _) = window.frame(release(target), |ui| show(ui, &ctx, &log));
        assert!(clicked);
    }

    #[test]
    fn compile_is_disabled_while_a_run_is_in_flight() {
        let ctx = settings_only_context();
        let (release_run, until) = unbounded();
        let mut log = RunLog::in_flight(until);
        let (_, nodes) = Headless::new().frame(Vec::new(), |ui| show(ui, &ctx, &log));
        assert!(button(&nodes, "Compile").is_disabled());
        release_run.send(()).unwrap();
        poll_until_done(|| {
            log.poll();
            log.is_running()
        });
        assert_eq!(log.lines(), ["Run finished: exit code 0"]);
    }

    #[test]
    fn the_settings_section_opens_on_the_effective_settings() {
        let ctx = settings_only_context();
        let log = RunLog::default();
        let mut window = Headless::new();
        let (_, nodes) = window.frame(Vec::new(), |ui| show(ui, &ctx, &log));
        assert!(
            !shows_text(&nodes, "pes_version = 21"),
            "collapsed at first"
        );
        let header = centre(button(&nodes, "Settings"));
        window.frame(press(header), |ui| show(ui, &ctx, &log));
        window.frame(release(header), |ui| show(ui, &ctx, &log));
        let (_, nodes) = window.frame(Vec::new(), |ui| show(ui, &ctx, &log));
        assert!(shows_text(&nodes, "pes_version = 21"));
        assert!(shows_text(&nodes, "cpk_name = \"4cc_99_test\""));
    }

    #[test]
    fn a_compile_click_through_the_tool_starts_a_run() {
        let temp = sandbox("view_compile_click");
        let root = temp.path();
        let ctx = tool_context(root, "");
        let mut tool = Tool::new();
        let mut window = Headless::new();
        let (_, nodes) = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
        let target = centre(button(&nodes, "Compile"));
        window.frame(press(target), |ui| tool.view(ui, &ctx));
        window.frame(release(target), |ui| tool.view(ui, &ctx));
        assert!(tool.run_log.is_running());
        poll_until_done(|| {
            tool.tick(&ctx);
            tool.run_log.is_running()
        });
        assert_eq!(
            tool.run_log.lines(),
            [
                "- Info bin_source (bin=TeamColor.bin, cpk=bundled)",
                "- Info bin_source (bin=UniColor.bin, cpk=bundled)",
                "- Info bin_source (bin=UniformParameter.bin, cpk=bundled)",
                "- egg Midcup Tracer: Info fmdl_weights_not_normalized at Players/05 - The Chad Stormworks Player (file=boots.fmdl, count=1662)",
                "- egg Midcup Tracer: Info fmdl_weights_not_normalized at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)",
                "- egg Midcup Tracer: Info fmdl_weights_not_normalized at Players/05 - The Chad Stormworks Player (file=glove_l.fmdl, count=2)",
                "- egg Midcup Tracer: Info export_identified (team=/egg/, id=792)",
                "- Warning player_table_missing (table=BootsList.bin, rows=1)",
                "- Warning player_table_missing (table=GloveList.bin, rows=1)",
                "Run finished: exit code 0",
            ]
        );
    }
}
