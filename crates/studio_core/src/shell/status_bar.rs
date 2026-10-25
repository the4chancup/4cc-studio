//! The status bar (core plan, "Status bar"; draft `docs/prototypes/index.html`): one 24 px row
//! under the sidebar and the tool view, drawn in every tool. Three slots: the version and the
//! held `ShellCondition`s left, the latest `Notice` middle, the other tools' `ToolActivity`s
//! right. Tools cannot draw into the bar; they return an activity and publish notices
//! (`crate::status`).

use std::time::Duration;

use crate::status::{ActionEffect, Notice, ShellCondition};

/// How long a notice without an action stays in the middle slot (the draft's
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
            // Middle: the notice — text in its kind's colour, its action, the
            // dismiss X — centred on the bar as one group. egui centres an
            // allocated rect, not a row of widgets, so the group's width is
            // measured from the galleys and the buttons' padding and the row is
            // padded to it.
            if let Some(notice) = &self.notice {
                columns[1].with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        let color = crate::theme::notice_color(notice.kind, ui.visuals());
                        let spacing = ui.spacing().item_spacing.x;
                        let button = egui::TextStyle::Button.resolve(ui.style());
                        let body = egui::TextStyle::Body.resolve(ui.style());
                        let galley_width = |ui: &mut egui::Ui, text: &str, font: &egui::FontId| {
                            ui.fonts_mut(|fonts| {
                                fonts.layout_no_wrap(text.to_owned(), font.clone(), color)
                            })
                            .size()
                            .x
                        };
                        let dismiss_width = galley_width(ui, crate::icons::X, &button);
                        let action_width = notice.action.as_ref().map_or(0.0, |action| {
                            galley_width(ui, &action.label, &button)
                                + 2.0 * ui.spacing().button_padding.x
                        });
                        let gaps = if notice.action.is_some() {
                            2.0 * spacing
                        } else {
                            spacing
                        };
                        let text_width = galley_width(ui, &notice.text, &body)
                            .min(ui.available_width() - dismiss_width - action_width - gaps)
                            .max(0.0);
                        let group = text_width + action_width + dismiss_width + gaps;
                        ui.add_space(((ui.available_width() - group) / 2.0).max(0.0));
                        ui.add_sized(
                            [text_width, ui.spacing().interact_size.y],
                            egui::Label::new(egui::RichText::new(&notice.text).color(color))
                                .truncate(),
                        )
                        .on_hover_text(&notice.text);
                        if let Some(action) = &notice.action
                            && ui.button(&action.label).clicked()
                        {
                            dismissed = true;
                            effect = Some(action.effect.clone());
                        }
                        let x = ui.add(egui::Button::new(crate::icons::X).frame(false));
                        x.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::Role::Button, true, "Dismiss")
                        });
                        if x.on_hover_text("Dismiss").clicked() {
                            dismissed = true;
                        }
                    });
                });
            }
            // Right: the other tools' activities, packed against the right edge
            // and truncating to it.
            columns[2].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let texts = activities.join(" · ");
                if !texts.is_empty() {
                    ui.add(egui::Label::new(&texts).truncate())
                        .on_hover_text(&texts);
                }
            });
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
                            // The elided label's own "full text" tooltip would
                            // mask the entry's, which explains the condition.
                            .show_tooltip_when_elided(false)
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
    use crate::status::{ActionEffect, Notice, NoticeAction, NoticeKind};

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
        let egui_ctx = test_context();
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let version = crate::shell::version_label();
        assert_eq!(
            bottom_texts(&nodes),
            [version.as_str()],
            "the bar shows the version alone"
        );
    }

    /// The frame's `Shape::Text` carrying exactly `text` painted inside the
    /// bottom 24 px of the 800x600 screen (the status bar's row).
    fn bar_text_shape<'a>(out: &'a FrameOutput, text: &str) -> Option<&'a egui::epaint::TextShape> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(shape) => Some(shape),
                _ => None,
            })
            .find(|shape| shape.galley.text() == text && shape.pos.y >= 576.0)
    }

    // ST-BAR-02
    #[test]
    fn the_right_slot_shows_the_other_tools_activities_not_the_active_ones() {
        let Shell {
            mut app,
            ctx,
            calls,
            _dir,
            ..
        } = shell();
        let egui_ctx = test_context();

        calls[1].set_activity("B", Some((12, 48)), Duration::from_secs(72));
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let activity = nodes
            .iter()
            .find(|node| node.value() == Some("B · 12/48 · 01:12"))
            .expect("the second tool's activity is drawn");
        let bounds = activity.bounds().unwrap();
        assert!(
            bounds.x0 >= 1600.0 / 3.0,
            "the activity sits in the right slot: {bounds:?}"
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
        let egui_ctx = test_context();

        // A no-action notice is drawn at t = 1 s and gone at t = 6.1 s.
        ctx.notify(Notice {
            text: "first".to_owned(),
            kind: NoticeKind::Info,
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
            kind: NoticeKind::Info,
            action: None,
        });
        frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(7.0));
        ctx.notify(Notice {
            text: "third".to_owned(),
            kind: NoticeKind::Info,
            action: None,
        });
        let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(8.0)).nodes;
        assert!(has_text(&nodes, "third"), "the newer notice shows");
        assert!(!has_text(&nodes, "second"), "the earlier one is replaced");

        // A notice with an action stays past 5 s; clicking the action applies it and
        // empties the slot.
        ctx.notify(Notice {
            text: "ready".to_owned(),
            kind: NoticeKind::Info,
            action: Some(NoticeAction {
                label: "Go".to_owned(),
                effect: ActionEffect::SwitchTool("b".to_owned()),
            }),
        });
        let out = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(14.0));
        let nodes = &out.nodes;
        assert!(has_text(nodes, "ready"));
        entry(nodes, "Go");

        // The middle slot shows the group — text, action, dismiss — centred on
        // the bar: the text's left edge to the dismiss button's right edge
        // straddle the 800 px screen's middle.
        let text = bar_text_shape(&out, "ready").expect("the notice text is painted");
        assert!(
            text.pos.x >= 800.0 / 3.0 && text.pos.x + text.galley.rect.width() <= 1600.0 / 3.0,
            "the notice text sits in the middle slot: {:?}",
            text.pos
        );
        let dismiss = entry(nodes, "Dismiss").bounds().unwrap();
        let centre = (f64::from(text.pos.x) + dismiss.x1) / 2.0;
        assert!(
            (centre - 400.0).abs() <= 4.0,
            "the notice group centres on the bar, centre at {centre}"
        );
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
            kind: NoticeKind::Info,
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

    // ST-BAR-03
    #[test]
    fn a_notice_too_long_for_the_middle_slot_truncates_its_text_and_the_group_fills_the_slot() {
        let Shell {
            mut app, ctx, _dir, ..
        } = shell();
        let egui_ctx = test_context();
        ctx.notify(Notice {
            text: "Wrote settings.toml for Snake into C:/Data/Documents/4cc/exports/DBC7/co/Snake"
                .to_owned(),
            kind: NoticeKind::Success,
            action: Some(NoticeAction {
                label: "Go".to_owned(),
                effect: ActionEffect::SwitchTool("b".to_owned()),
            }),
        });
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;

        // The middle of three columns across the bar's 784 px (800 less its 8 px side
        // margins, `shell::mod`), the columns `item_spacing.x` apart.
        let spacing = egui_ctx.global_style().spacing.item_spacing.x;
        let column = (784.0 - 2.0 * spacing) / 3.0;
        let slot_left = f64::from(8.0 + column + spacing);
        let slot_right = slot_left + f64::from(column);
        let text = nodes
            .iter()
            .find(|node| {
                node.role() == Role::Label && node.value().is_some_and(|v| v.starts_with("Wrote"))
            })
            .expect("the notice's label")
            .bounds()
            .unwrap();
        let dismiss = entry(&nodes, "Dismiss").bounds().unwrap();
        // The text is cut to what the action and the X leave, so the group spans the slot
        // edge to edge, neither overflowing it nor leaving it half empty.
        assert!(
            (text.x0 - slot_left).abs() <= 1.0 && (dismiss.x1 - slot_right).abs() <= 1.0,
            "the group spans the slot {slot_left}..{slot_right}: text from {}, X to {}",
            text.x0,
            dismiss.x1
        );
    }

    // ST-BAR-04
    #[test]
    fn each_notice_kind_paints_its_text_in_its_own_colour() {
        let Shell {
            mut app, ctx, _dir, ..
        } = shell();
        let egui_ctx = test_context();
        let visuals = egui_ctx.global_style().visuals.clone();
        let cases = [
            (NoticeKind::Success, "did it", crate::theme::SUCCESS_FG),
            (NoticeKind::Info, "heads up", visuals.hyperlink_color),
            (NoticeKind::Warning, "careful", visuals.warn_fg_color),
        ];
        for (kind, text, expected) in cases {
            ctx.notify(Notice {
                text: text.to_owned(),
                kind,
                action: None,
            });
            let out = frame(&egui_ctx, &mut app, Vec::new(), true);
            let shape = bar_text_shape(&out, text).expect("the notice text is painted");
            let color = shape
                .galley
                .job
                .sections
                .first()
                .map(|section| section.format.color);
            assert_eq!(
                color,
                Some(expected),
                "{kind:?} paints its text in {expected:?}"
            );
        }
    }

    // ST-SET-03
    #[test]
    fn a_failed_settings_save_raises_the_read_only_condition_and_a_save_drops_it() {
        let egui_ctx = test_context();
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
