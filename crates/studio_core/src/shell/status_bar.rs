//! The status bar, as the draft `docs/prototypes/index.html` draws it: one 24 px row under the
//! sidebar and the tool view, drawn in every tool. Three slots: the version and the
//! held `ShellCondition`s left at their content's width, the other tools' `ToolActivity`s
//! right at their text's width capped at a third of the bar, and the latest `Notice` middle
//! in the space they leave. Tools cannot draw into the bar; they return an activity and
//! publish notices (`crate::status`).

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

    /// Draws the row's slots, the middle taking what the sides leave: the left and right
    /// slots are as wide as what they hold and the notice's slot is the rest.
    /// `activities` are the not-active tools' activity texts in
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
        let bar = ui.max_rect();
        let spacing = ui.spacing().item_spacing.x;
        let left_right = self.left_slot(ui);
        // Right: the other tools' activities, at their text's width capped at a third
        // of the bar (an activity line is short; the cap keeps a long one from taking
        // the notice's room), packed against the right edge and truncating past the
        // cap. No activity, no width.
        let texts = activities.join(" · ");
        let right_width = if texts.is_empty() {
            0.0
        } else {
            ui.fonts_mut(|fonts| {
                fonts.layout_no_wrap(
                    texts.clone(),
                    egui::TextStyle::Body.resolve(ui.style()),
                    ui.visuals().text_color(),
                )
            })
            .size()
            .x
            .min(bar.width() / 3.0)
        };
        if right_width > 0.0 {
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(egui::Rect::from_min_max(
                        egui::pos2(bar.right() - right_width, bar.top()),
                        bar.right_bottom(),
                    ))
                    .layout(egui::Layout::right_to_left(egui::Align::Center)),
                |ui| {
                    ui.add(egui::Label::new(&texts).truncate())
                        .on_hover_text(&texts);
                },
            );
        }
        // Middle: the notice — text in its kind's colour, its action, the
        // dismiss X — centred in the space the sides leave as one group. egui
        // centres an allocated rect, not a row of widgets, so the group's width
        // is measured from the galleys and the buttons' padding and the row is
        // padded to it.
        let middle_left = left_right + spacing;
        let middle_right = if right_width > 0.0 {
            bar.right() - right_width - spacing
        } else {
            bar.right()
        }
        .max(middle_left);
        // The scope's id is pinned: the right slot's scope exists only while an
        // activity is up, and a salted id would change the notice's widget ids
        // the frame one appears.
        ui.scope_builder(
            egui::UiBuilder::new()
                .scope_id(ui.make_persistent_id("notice"))
                .max_rect(egui::Rect::from_min_max(
                    egui::pos2(middle_left, bar.top()),
                    egui::pos2(middle_right, bar.bottom()),
                )),
            |ui| {
                if let Some(notice) = &self.notice {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.horizontal(|ui| {
                            let color = crate::theme::notice_color(notice.kind, ui.visuals());
                            let spacing = ui.spacing().item_spacing.x;
                            let button = egui::TextStyle::Button.resolve(ui.style());
                            let body = egui::TextStyle::Body.resolve(ui.style());
                            let galley_width =
                                |ui: &mut egui::Ui, text: &str, font: &egui::FontId| {
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
            },
        );
        if effect.is_some() || dismissed {
            self.dismiss();
        }
        effect
    }

    /// The left slot: the version, always, then one icon-and-text entry per held condition,
    /// each carrying its tooltip. Returns the content's right edge, the middle slot's
    /// left bound.
    fn left_slot(&self, ui: &mut egui::Ui) -> f32 {
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
            ui.min_rect().right()
        })
        .inner
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

    /// The left slot's right edge: the version label's right side (no condition is
    /// raised in these tests, so the version is the slot's last entry).
    fn left_slot_right(nodes: &[egui::accesskit::Node]) -> f64 {
        let version = crate::shell::version_label();
        nodes
            .iter()
            .find(|node| node.value() == Some(version.as_str()))
            .expect("the version label is drawn")
            .bounds()
            .unwrap()
            .x1
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

        // The middle slot shows the group — text, action, dismiss — centred in
        // the space the sides leave: the left slot's right edge plus item
        // spacing to the bar's right margin, with no activity up.
        let text = bar_text_shape(&out, "ready").expect("the notice text is painted");
        let spacing = f64::from(egui_ctx.global_style().spacing.item_spacing.x);
        let slot_left = left_slot_right(nodes) + spacing;
        let slot_right = 800.0 - 8.0;
        let dismiss = entry(nodes, "Dismiss").bounds().unwrap();
        let centre = (f64::from(text.pos.x) + dismiss.x1) / 2.0;
        assert!(
            (centre - (slot_left + slot_right) / 2.0).abs() <= 4.0,
            "the notice group centres in the space the sides leave {slot_left}..{slot_right}, centre at {centre}"
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
            text: "Although the sidebar is set to PES 2019, this save is from PES 2021, so the editor will keep treating it as a PES 2021 save."
                .to_owned(),
            kind: NoticeKind::Warning,
            action: Some(NoticeAction {
                label: "Go".to_owned(),
                effect: ActionEffect::SwitchTool("b".to_owned()),
            }),
        });
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;

        // The middle slot is what the sides leave: the left slot's right edge
        // plus item spacing to the bar's right margin (800 less its 8 px side
        // margin, `shell::mod`), with no activity up.
        let spacing = f64::from(egui_ctx.global_style().spacing.item_spacing.x);
        let slot_left = left_slot_right(&nodes) + spacing;
        let slot_right = 800.0 - 8.0;
        let text = nodes
            .iter()
            .find(|node| {
                node.role() == Role::Label
                    && node.value().is_some_and(|v| v.starts_with("Although"))
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

    // ST-BAR-03
    #[test]
    fn a_hundred_character_notice_is_drawn_whole_on_a_1280_point_window() {
        let Shell {
            mut app, ctx, _dir, ..
        } = shell();
        let egui_ctx = test_context();
        const NOTICE_TEXT: &str = "Although the sidebar is set to PES 2019, this save is from PES 2021, so the editor will keep treating it as a PES 2021 save.";
        ctx.notify(Notice {
            text: NOTICE_TEXT.to_owned(),
            kind: NoticeKind::Warning,
            action: Some(NoticeAction {
                label: "Switch to PES 2021".to_owned(),
                effect: ActionEffect::SwitchTool("b".to_owned()),
            }),
        });
        let mut input = raw_input(Vec::new(), true, None, None);
        input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 800.0),
        ));
        let out = run_frame(&egui_ctx, &mut app, input);

        assert!(
            out.nodes
                .iter()
                .any(|node| { node.role() == Role::Label && node.value() == Some(NOTICE_TEXT) }),
            "the notice's label carries the whole text"
        );
        let shape = bar_text_shape(&out, NOTICE_TEXT).expect("the notice text is painted");
        assert!(
            !shape.galley.elided,
            "the painted galley is the notice's whole text, no ellipsis"
        );
    }

    // ST-BAR-03
    #[test]
    fn the_activity_ends_at_the_right_margin_and_the_notice_stops_before_it() {
        let Shell {
            mut app,
            ctx,
            calls,
            _dir,
            ..
        } = shell();
        let egui_ctx = test_context();
        calls[1].set_activity("B", Some((12, 48)), Duration::from_secs(72));
        ctx.notify(Notice {
            text: "Although the sidebar is set to PES 2019, this save is from PES 2021, so the editor will keep treating it as a PES 2021 save.".to_owned(),
            kind: NoticeKind::Warning,
            action: Some(NoticeAction {
                label: "Switch to PES 2021".to_owned(),
                effect: ActionEffect::SwitchTool("b".to_owned()),
            }),
        });
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;

        let activity = nodes
            .iter()
            .find(|node| node.value() == Some("B · 12/48 · 01:12"))
            .expect("the second tool's activity is drawn")
            .bounds()
            .unwrap();
        assert!(
            (activity.x1 - (800.0 - 8.0)).abs() <= 1.0,
            "the activity's text ends at the bar's right margin: {activity:?}"
        );
        let dismiss = entry(&nodes, "Dismiss").bounds().unwrap();
        assert!(
            dismiss.x1 <= activity.x0,
            "the notice group stops before the activity {activity:?}: {dismiss:?}"
        );
    }

    // ST-BAR-03's right-slot cap: an activity text wider than a third of
    // the bar is drawn in a slot a third of the bar wide, its right edge
    // on the bar's (the 800 window's margin holds the bar to 8..792).
    #[test]
    fn a_wide_activity_is_capped_at_a_third_of_the_bar() {
        let Shell {
            mut app,
            calls,
            _dir,
            ..
        } = shell();
        let egui_ctx = test_context();
        calls[1].set_activity(
            "B and its much longer tool name for the slot cap",
            Some((12, 48)),
            Duration::from_secs(72),
        );
        let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
        let bounds = nodes
            .iter()
            .find(|node| {
                node.value()
                    .is_some_and(|value| value.starts_with("B and its much longer"))
            })
            .expect("the activity label")
            .bounds()
            .unwrap();
        assert!(
            bounds.x1 - bounds.x0 <= 784.0 / 3.0 + 1.0,
            "the slot is a third of the bar at most ({bounds:?})"
        );
        assert!(
            (bounds.x1 - (800.0 - 8.0)).abs() <= 1.0,
            "its right edge is the bar's right edge ({bounds:?})"
        );
    }

    // ST-BAR-03's middle slot, beside an activity: the notice's group is
    // centred in the space the left and right slots leave (the left
    // slot's right edge plus spacing, the right slot's left edge minus
    // it).
    #[test]
    fn the_notice_group_centres_between_the_slots() {
        let Shell {
            mut app,
            ctx,
            calls,
            _dir,
            ..
        } = shell();
        let egui_ctx = test_context();
        calls[1].set_activity("B", Some((12, 48)), Duration::from_secs(72));
        ctx.notify(Notice {
            text: "ready".to_owned(),
            kind: NoticeKind::Info,
            action: Some(NoticeAction {
                label: "Go".to_owned(),
                effect: ActionEffect::SwitchTool("b".to_owned()),
            }),
        });
        let out = frame(&egui_ctx, &mut app, Vec::new(), true);
        let nodes = &out.nodes;

        let activity = nodes
            .iter()
            .find(|node| node.value() == Some("B · 12/48 · 01:12"))
            .expect("the second tool's activity is drawn")
            .bounds()
            .unwrap();
        let text = bar_text_shape(&out, "ready").expect("the notice text is painted");
        let dismiss = entry(nodes, "Dismiss").bounds().unwrap();
        let spacing = f64::from(egui_ctx.global_style().spacing.item_spacing.x);
        let slot_left = left_slot_right(nodes) + spacing;
        let slot_right = activity.x0 - spacing;
        let centre = (f64::from(text.pos.x) + dismiss.x1) / 2.0;
        assert!(
            (centre - (slot_left + slot_right) / 2.0).abs() <= 1.0,
            "the notice group centres in {slot_left}..{slot_right}, centre at {centre}"
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
                "Settings and the teams list cannot be saved, so changes last for this \
                 session only"
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
