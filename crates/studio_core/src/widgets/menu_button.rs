//! The toolbars' menu button: egui's `menu_button`, its label ending in the
//! drafts' arrow.

/// A toolbar menu button: egui's `menu_button` with `label` followed by ` ⏷`
/// (U+23F7, the symbols font's), as the drafts draw every menu button. egui
/// draws an arrow only on a submenu's button, and the arrow is what tells a
/// menu from a button.
pub fn menu_button<R>(
    ui: &mut egui::Ui,
    label: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<Option<R>> {
    ui.menu_button(format!("{label} \u{23f7}"), add_contents)
}

#[cfg(test)]
mod tests {
    use egui::accesskit::{self, Role};

    use super::*;

    /// One frame of a toolbar holding the "Team" menu, whose popup holds a
    /// "Set edited flag" item, with `events` as its input; the frame's
    /// AccessKit nodes.
    fn frame(ctx: &egui::Context, events: Vec<egui::Event>) -> Vec<accesskit::Node> {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            menu_button(ui, "Team", |ui| ui.button("Set edited flag"));
        });
        let update = output
            .platform_output
            .accesskit_update
            .take()
            .expect("AccessKit is enabled on the context");
        // Nothing renders here, so the frame's texture deltas are dropped on
        // purpose.
        output.drop_without_applying_deltas();
        update.nodes.into_iter().map(|(_, node)| node).collect()
    }

    fn button<'a>(nodes: &'a [accesskit::Node], label: &str) -> Option<&'a accesskit::Node> {
        nodes
            .iter()
            .find(|node| node.role() == Role::Button && node.label() == Some(label))
    }

    /// The primary button pressed (or released) at `pos`, the pointer moved
    /// there first.
    fn pointer_at(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
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
    fn the_label_ends_in_the_arrow_and_a_click_opens_the_menu() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        crate::theme::install(&ctx);
        let nodes = frame(&ctx, Vec::new());
        let team = button(&nodes, "Team \u{23f7}").expect("the menu button reads \"Team ⏷\"");
        assert!(
            button(&nodes, "Set edited flag").is_none(),
            "the menu starts closed"
        );
        let bounds = team.bounds().expect("a drawn button has bounds");
        let pos = egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        frame(&ctx, pointer_at(pos, true));
        frame(&ctx, pointer_at(pos, false));
        let nodes = frame(&ctx, Vec::new());
        assert!(
            button(&nodes, "Set edited flag").is_some(),
            "the click opened the menu"
        );
    }
}
