//! The status bar (core plan, "Status bar"; draft `docs/prototypes/shell.html`): one 24 px row
//! under the sidebar and the tool view, drawn in every tool. Three slots: the version and the
//! held `ShellCondition`s left, the other tools' `ToolActivity`s middle, the latest `Notice`
//! right. Tools cannot draw into the bar; they return an activity and publish notices
//! (`crate::status`).

use std::time::Duration;

use crate::status::{ActionEffect, Notice, ShellCondition};

/// How long a notice without an action stays in the right slot (the draft's
/// `noticeTimeoutS`), measured on egui's clock from the frame it is first drawn. A notice
/// with an action stays until the action or the dismiss is clicked.
const NOTICE_SECONDS: f64 = 5.0;

/// The bar's state across frames: the conditions currently held and the latest notice.
#[derive(Default)]
pub(super) struct StatusBar {
    /// The held conditions, in the order they were raised; each leaves when the shell drops
    /// it, not by timeout.
    conditions: Vec<ShellCondition>,
    /// The latest notice; a new one replaces it (older notices are not kept).
    notice: Option<Notice>,
    /// egui time of the frame a no-action notice was first drawn.
    notice_shown_at: Option<f64>,
}

impl StatusBar {
    /// Raises `condition` while `holds`, drops it when it stops. A held condition is shown
    /// once even if raised twice. Only the settings save raises one for now, and the browser
    /// build has no settings file.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn set_condition(&mut self, condition: ShellCondition, holds: bool) {
        if holds {
            if !self.conditions.contains(&condition) {
                self.conditions.push(condition);
            }
        } else {
            self.conditions.retain(|held| *held != condition);
        }
    }

    /// The latest notice; it replaces the one before it.
    pub(super) fn set_notice(&mut self, notice: Notice) {
        self.notice = Some(notice);
        self.notice_shown_at = None;
    }

    /// Drops the shown notice, if any.
    fn dismiss(&mut self) {
        self.notice = None;
        self.notice_shown_at = None;
    }

    /// Drops a no-action notice once `NOTICE_SECONDS` have passed since the frame it was
    /// first drawn, and asks for the repaint that removes it while no input arrives.
    fn expire(&mut self, ui: &egui::Ui) {
        let now = ui.input(|input| input.time);
        let expired = match &self.notice {
            Some(notice) if notice.action.is_none() => {
                let shown_at = *self.notice_shown_at.get_or_insert(now);
                let remaining = NOTICE_SECONDS - (now - shown_at);
                if remaining > 0.0 {
                    ui.ctx()
                        .request_repaint_after(Duration::from_secs_f64(remaining));
                }
                remaining <= 0.0
            }
            _ => false,
        };
        if expired {
            self.dismiss();
        }
    }

    /// Draws the row's three slots. `activities` are the not-active tools' activity texts in
    /// sidebar order (the active tool's run strip already shows its own). Returns the notice
    /// action the user clicked, for the shell to apply; clicking it also dismisses the
    /// notice.
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        activities: &[String],
    ) -> Option<ActionEffect> {
        self.expire(ui);
        let mut effect = None;
        let mut dismissed = false;
        ui.columns(3, |columns| {
            self.left_slot(&mut columns[0]);
            // Middle: the other tools' activities, centered as the draft lays it out.
            columns[1].with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let texts = activities.join(" · ");
                if !texts.is_empty() {
                    ui.add(egui::Label::new(&texts).truncate())
                        .on_hover_text(&texts);
                }
            });
            // Right: notice text, its action, the dismiss X — packed against the right edge,
            // the text taking what is left and truncating to it.
            if let Some(notice) = &self.notice {
                columns[2].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let x = ui.add(egui::Button::new(crate::icons::X).frame(false));
                    x.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::Button, true, "Dismiss")
                    });
                    if x.on_hover_text("Dismiss").clicked() {
                        dismissed = true;
                    }
                    if let Some(action) = &notice.action
                        && ui.button(&action.label).clicked()
                    {
                        dismissed = true;
                        effect = Some(action.effect.clone());
                    }
                    ui.add(egui::Label::new(&notice.text).truncate())
                        .on_hover_text(&notice.text);
                });
            }
        });
        if effect.is_some() || dismissed {
            self.dismiss();
        }
        effect
    }

    /// The left slot: the version, always, then one icon-and-text entry per held condition,
    /// each carrying its tooltip.
    fn left_slot(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(crate::shell::version_label())
                    .small()
                    .weak(),
            );
            for condition in &self.conditions {
                // The labels are not selectable: a selectable label senses click+drag and
                // would shadow the entry's own `hovered`, keeping the tooltip shut.
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(condition.icon()).selectable(false));
                    ui.add(
                        egui::Label::new(condition.text())
                            .truncate()
                            .selectable(false),
                    );
                })
                .response
                .on_hover_text(condition.tooltip());
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use egui::accesskit::Role;
    use pes_version::PesVersion;

    use crate::shell::test_harness::*;
    use crate::status::{ActionEffect, Notice, NoticeAction};

    /// The `Label` nodes' texts drawn inside the bottom 24 px of the 800x600 screen (a
    /// `RichText` label also leaves a `TextRun` child, so `Label` alone is counted).
    fn bottom_texts(nodes: &[egui::accesskit::Node]) -> Vec<&str> {
        nodes
            .iter()
            .filter(|node| node.role() == Role::Label)
            .filter(|node| node.bounds().is_some_and(|bounds| bounds.y0 >= 576.0))
            .filter_map(|node| node.value())
            .collect()
    }

    // ST-BAR-01
    #[test]
    fn an_empty_bar_shows_the_version_and_nothing_else() {
        let Shell { mut app, _dir, .. } = shell();
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let version = crate::shell::version_label();
        assert_eq!(
            bottom_texts(&nodes),
            [version.as_str()],
            "the bar shows the version alone"
        );
    }

    // ST-BAR-02
    #[test]
    fn the_middle_slot_shows_the_other_tools_activities_not_the_active_ones() {
        let Shell {
            mut app,
            ctx,
            calls,
            _dir,
            ..
        } = shell();
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        calls[1].set_activity("B", Some((12, 48)), Duration::from_secs(72));
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let activity = nodes
            .iter()
            .find(|node| node.value() == Some("B · 12/48 · 01:12"))
            .expect("the second tool's activity is drawn");
        let bounds = activity.bounds().unwrap();
        assert!(
            bounds.x0 >= 800.0 / 3.0 && bounds.x1 <= 1600.0 / 3.0,
            "the activity sits in the middle slot: {bounds:?}"
        );

        // With the second tool active its activity leaves the slot.
        ctx.switch_to_tool("b");
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert!(
            !has_text(&nodes, "B · 12/48 · 01:12"),
            "the active tool's activity is its own view's, not the bar's"
        );
    }

    // ST-BAR-03
    #[test]
    fn a_notice_with_an_action_stays_a_plain_one_leaves_and_a_newer_replaces() {
        let Shell {
            mut app, ctx, _dir, ..
        } = shell();
        // The sidebar's poll would ask for its own repaint to the same deadline and mask the
        // notice's; spending the once-per-life auto-switch leaves the expiry the only
        // requester the delay assertion sees.
        app.sidebar.auto_switch_used = true;
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        // A no-action notice is drawn at t = 1 s and gone at t = 6.1 s.
        ctx.notify(Notice {
            text: "first".to_owned(),
            action: None,
        });
        let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(1.0)).nodes;
        assert!(has_text(&nodes, "first"), "the notice is drawn");
        // While the notice is up the expiry asks for the repaint that removes it: at t = 3 s
        // three of its five seconds are left (egui trims the request by the predicted frame
        // time, so the bound is a hair under). The t = 2 s frame is not read: the first
        // frame's own request (the window's theme sync) is still settling there.
        frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(2.0));
        let out = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(3.0));
        let delay = out.repaint_delay.as_secs_f64();
        assert!(
            (2.9..=3.0).contains(&delay),
            "the expiry asks for a repaint at the notice's deadline, got {delay} s"
        );
        let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(5.9)).nodes;
        assert!(
            has_text(&nodes, "first"),
            "still drawn 0.1 s short of the deadline"
        );
        let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(6.1)).nodes;
        assert!(
            !has_text(&nodes, "first"),
            "a notice without an action leaves after 5 s"
        );

        // A later notice replaces the earlier one.
        ctx.notify(Notice {
            text: "second".to_owned(),
            action: None,
        });
        frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(7.0));
        ctx.notify(Notice {
            text: "third".to_owned(),
            action: None,
        });
        let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(8.0)).nodes;
        assert!(has_text(&nodes, "third"), "the newer notice shows");
        assert!(!has_text(&nodes, "second"), "the earlier one is replaced");

        // A notice with an action stays past 5 s; clicking the action applies it and
        // empties the slot.
        ctx.notify(Notice {
            text: "ready".to_owned(),
            action: Some(NoticeAction {
                label: "Go".to_owned(),
                effect: ActionEffect::SwitchTool("b".to_owned()),
            }),
        });
        let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(14.0)).nodes;
        assert!(has_text(&nodes, "ready"));
        entry(&nodes, "Go");
        let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(20.0)).nodes;
        assert!(
            has_text(&nodes, "ready"),
            "an action notice does not expire"
        );
        click(&egui_ctx, &mut app, entry(&nodes, "Go"));
        assert_eq!(app.active, 1, "the SwitchTool effect applied");
        let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(21.0)).nodes;
        assert!(!has_text(&nodes, "ready"), "the click emptied the slot");

        // The dismiss button empties the slot without applying anything.
        ctx.notify(Notice {
            text: "dismiss me".to_owned(),
            action: Some(NoticeAction {
                label: "Go".to_owned(),
                effect: ActionEffect::SwitchTool("a".to_owned()),
            }),
        });
        let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(22.0)).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "Dismiss"));
        assert_eq!(app.active, 1, "dismiss applies nothing");
        let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(23.0)).nodes;
        assert!(!has_text(&nodes, "dismiss me"), "dismissed");
    }

    // ST-SET-03
    #[test]
    fn a_failed_settings_save_raises_the_read_only_condition_and_a_save_drops_it() {
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();
        egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

        // A `data_dir` that is a regular file fails the save on every platform.
        let dir = test_dir("studio_core_status_bar_read_only");
        fs::create_dir_all(&dir).unwrap();
        let data_dir = dir.join("data");
        fs::write(&data_dir, b"").unwrap();

        let (a, calls_a) = stub("a", "A", None);
        let mut settings = test_settings(&dir);
        settings.common.pes_version = PesVersion::Pes19;
        let Shell {
            mut app,
            ctx,
            settings,
            ..
        } = shell_with(
            settings,
            vec![a],
            &[],
            Some(data_dir.clone()),
            vec![calls_a],
        );

        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, combo(&nodes));
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "PES 2021"));

        // The change applies for the session even though the file cannot be written.
        assert_eq!(ctx.common().pes_version, PesVersion::Pes21);
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert_eq!(
            settings.lock().unwrap().common.pes_version,
            PesVersion::Pes21,
            "the in-memory setting kept the pick"
        );
        assert!(
            bottom_texts(&nodes)
                .iter()
                .any(|text| text.contains("Data folder is read-only")),
            "the condition is in the left slot"
        );
        assert!(
            bottom_texts(&nodes).contains(&crate::icons::WARNING),
            "the entry draws the warning glyph"
        );

        // Hovering the entry shows the tooltip. A hover that follows a click within 0.1 s is
        // suppressed, so the pick's click ages out first; `is_still` then takes about 0.1 s
        // of idle frames.
        let nodes = idle(&egui_ctx, &mut app, 8).nodes;
        let node = nodes
            .iter()
            .find(|node| node.value() == Some("Data folder is read-only"))
            .expect("the condition's text label");
        frame(&egui_ctx, &mut app, hover(node), true);
        let nodes = idle(&egui_ctx, &mut app, 8).nodes;
        assert!(
            has_text(
                &nodes,
                "Settings and the teams list cannot be saved; changes last for this session"
            ),
            "the entry's tooltip explains the session-only setting"
        );

        assert_eq!(
            nodes
                .iter()
                .filter(|node| node.role() == Role::Window)
                .count(),
            1,
            "only the frame's root window node: no dialog opened"
        );

        // The folder made writable, the next save succeeds and the condition drops.
        fs::remove_file(&data_dir).unwrap();
        fs::create_dir(&data_dir).unwrap();
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, combo(&nodes));
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        click(&egui_ctx, &mut app, entry(&nodes, "PES 2020"));
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        assert!(
            !bottom_texts(&nodes)
                .iter()
                .any(|text| text.contains("Data folder is read-only")),
            "a save that succeeds drops the condition"
        );
        assert_eq!(ctx.common().pes_version, PesVersion::Pes20);
        fs::remove_dir_all(&dir).unwrap();
    }
}
