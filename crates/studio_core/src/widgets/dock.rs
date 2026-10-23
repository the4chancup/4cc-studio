//! A bottom dock's frame (core plan, "Log panel"): a resizable `Panel` docked at the
//! bottom of the `Ui` it is given, its header row carrying a collapse chevron and the
//! title ahead of whatever the caller draws in it. The frame holds no log-specific
//! state: the Save editor's Live AATF list docks in it too (`plans/save_editor.md`
//! "Configurable AATF rules").

/// The open dock's starting height in points; egui keeps the resized one across
/// frames for the session, never in settings.
const DEFAULT_HEIGHT: f32 = 220.0;

/// The open dock's smallest height in points, as the draft's divider drag floors at.
const MIN_HEIGHT: f32 = 60.0;

/// A bottom dock's frame: a bottom `Panel` whose header row holds the collapse
/// chevron and the title, resizable by its divider while open. Collapsed, it is a
/// non-resizable panel holding the header alone, under a second panel id so the open
/// panel's stored height survives the collapse (ST-LOG-04).
pub struct BottomDock {
    /// The open panel's egui id; the collapsed one's derives from it.
    id: egui::Id,
    /// The strong text after the chevron (`Log`), and the chevron tooltips' noun.
    title: &'static str,
    /// Whether the body is shown; the chevron flips it.
    expanded: bool,
}

impl BottomDock {
    /// A dock titled `title`, open.
    pub fn new(id_salt: impl Into<egui::Id>, title: &'static str) -> BottomDock {
        BottomDock {
            id: id_salt.into(),
            title,
            expanded: true,
        }
    }

    /// Draws the dock: `header` fills the header row after the title and runs in both
    /// states; `body` runs only while open.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        header: impl FnOnce(&mut egui::Ui),
        body: impl FnOnce(&mut egui::Ui),
    ) {
        let noun = self.title.to_lowercase();
        // Read before the closure borrows the field it flips.
        let expanded = self.expanded;
        let header_row = |ui: &mut egui::Ui| {
            ui.horizontal(|ui| {
                let (chevron, tooltip) = if self.expanded {
                    ("▾", format!("Collapse the {noun}"))
                } else {
                    ("▸", format!("Expand the {noun}"))
                };
                if ui
                    .add(egui::Button::new(chevron).frame(false))
                    .on_hover_text(tooltip)
                    .clicked()
                {
                    self.expanded = !self.expanded;
                }
                ui.strong(self.title);
                header(ui);
            });
        };
        if expanded {
            egui::Panel::bottom(self.id)
                .resizable(true)
                .default_size(DEFAULT_HEIGHT)
                .min_size(MIN_HEIGHT)
                .show(ui, |ui| {
                    header_row(ui);
                    body(ui);
                });
        } else {
            // The second id is what keeps the open panel's `PanelState` (its height)
            // for the return.
            egui::Panel::bottom(self.id.with("collapsed")).show(ui, header_row);
        }
    }

    /// The open and collapsed panels' egui ids, in that order — since egui 0.37 the
    /// salts a `Panel` resolves its persisted `PanelState` key and child scope from,
    /// against the parent `Ui`'s scope id; the widgets' tests measure them.
    #[cfg(test)]
    pub(crate) fn panel_ids(&self) -> (egui::Id, egui::Id) {
        (self.id, self.id.with("collapsed"))
    }
}
