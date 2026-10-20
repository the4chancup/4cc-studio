//! The sidebar, top to bottom (core plan, "Sidebar"; draft `docs/prototypes/shell.html`):
//! the logo doubling as the About click target, the PES version selector with its live exe
//! check and its blocker, the tool list with the planned tools grayed, and the collapse
//! button switching the sidebar between labels and an icon rail. The `?` and gear buttons
//! that close the draft's column join with the windows they open.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use pes_version::PesVersion;

use crate::settings::CommonSettings;
use crate::tool::{PlannedTool, ShellRequest, StudioTool, ToolContext};

use super::about;

/// The sidebar's two widths (draft: 160 px expanded, 56 px as the icon rail).
const EXPANDED_WIDTH: f32 = 160.0;
const COLLAPSED_WIDTH: f32 = 56.0;
/// The accent bar on the active entry's left edge (draft: 3 px).
const ACTIVE_BAR_WIDTH: f32 = 3.0;

/// The sidebar's state across frames: the collapse, the exe check's last verdict, the
/// window's last focus (the check re-runs when focus returns) and the logo's About popup.
#[derive(Default)]
pub(super) struct Sidebar {
    /// Collapsed: the rail shows icons only and carries the labels as tooltips.
    pub(super) collapsed: bool,
    /// What the last exe check ran against and found: the version, the resolved PES folder
    /// and the verdict. `None` until the first listing.
    pub(super) exe_check: Option<(PesVersion, PathBuf, ExeCheck)>,
    /// How many directory listings the check has run; the tests read it (the check never
    /// re-lists per frame).
    pub(super) exe_listings: u64,
    /// The window's focus last frame, for the gained-focus re-check.
    was_focused: bool,
    /// The logo's About popup.
    pub(super) about_open: bool,
}

/// What the exe check's last directory listing found (ST-SIDE-03): the selector is red on
/// `OtherVersion`, yellow on `NoExe` and `NoFolder`, uncolored on `Found`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExeCheck {
    /// The resolved PES folder holds the selected version's exe.
    Found,
    /// It exists but holds a different version's exe (wrong selection or wrong path).
    OtherVersion(PesVersion),
    /// It exists but holds no `PES20NN.exe` at all.
    NoExe,
    /// It does not exist (or cannot be listed).
    NoFolder,
}

impl Sidebar {
    /// The panel width the current collapse asks for.
    pub(super) fn width(&self) -> f32 {
        if self.collapsed {
            COLLAPSED_WIDTH
        } else {
            EXPANDED_WIDTH
        }
    }

    /// Draws the sidebar and returns the registered tool the user clicked, if any. `tools`
    /// are in sidebar order, `active` the index of the one on screen.
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        tools: &[Box<dyn StudioTool>],
        active: usize,
        planned: &[PlannedTool],
        ctx: &ToolContext,
    ) -> Option<usize> {
        let clicked = if self.collapsed {
            // The rail centers its icons (draft: `.sidebar.collapsed`).
            ui.vertical_centered(|ui| self.contents(ui, tools, active, planned, ctx))
                .inner
        } else {
            self.contents(ui, tools, active, planned, ctx)
        };
        if self.about_open {
            about::show(
                ui.ctx(),
                ctx.paths(),
                ctx.common().pes_version,
                &mut self.about_open,
            );
        }
        clicked
    }

    /// The items, top to bottom as the draft lays them out.
    fn contents(
        &mut self,
        ui: &mut egui::Ui,
        tools: &[Box<dyn StudioTool>],
        active: usize,
        planned: &[PlannedTool],
        ctx: &ToolContext,
    ) -> Option<usize> {
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("4cc")
                        .color(ui.visuals().selection.stroke.color)
                        .size(15.0),
                )
                .frame(false),
            )
            .on_hover_text("About 4cc Studio")
            .clicked()
        {
            self.about_open = true;
        }
        self.version_selector(ui, tools, ctx);
        ui.separator();

        let mut clicked = None;
        for (index, tool) in tools.iter().enumerate() {
            if self
                .tool_entry(ui, tool.icon(), tool.label(), index == active)
                .clicked()
            {
                clicked = Some(index);
            }
        }
        for entry in planned {
            if tools.iter().any(|tool| tool.id() == entry.id) {
                continue;
            }
            self.planned_entry(ui, entry);
        }

        let (glyph, tip) = if self.collapsed {
            ("»", "Expand the sidebar")
        } else {
            ("«", "Collapse the sidebar")
        };
        if ui
            .add(egui::Button::new(glyph).frame(false))
            .on_hover_text(tip)
            .clicked()
        {
            self.collapsed = !self.collapsed;
        }
        clicked
    }

    /// The PES version selector: a compact combo bound to the common `pes_version`, disabled
    /// with the blocking tool's reason while a `version_change_blocker` reports one, colored
    /// by the live exe check. A change writes the common setting and asks the shell to save.
    fn version_selector(
        &mut self,
        ui: &mut egui::Ui,
        tools: &[Box<dyn StudioTool>],
        ctx: &ToolContext,
    ) {
        let common = ctx.common();
        let blocker = tools.iter().find_map(|tool| {
            tool.version_change_blocker()
                .map(|reason| (tool.label(), reason))
        });
        let folder = common.pes_folder();
        let check = self.exe_check(&common, ui.input(|input| input.focused));
        let tooltip = match check {
            ExeCheck::Found => format!("{}\nThe PES version every tool targets", folder.display()),
            ExeCheck::OtherVersion(found) => format!(
                "Found PES{}.exe in {}: a different version than the selection",
                found.year(),
                folder.display()
            ),
            ExeCheck::NoExe => format!("No PES exe in {}", folder.display()),
            ExeCheck::NoFolder => format!("{} does not exist", folder.display()),
        };

        let mut picked = common.pes_version;
        let combo_response = ui
            .add_enabled_ui(blocker.is_none(), |ui| {
                ui.scope(|ui| {
                    ui.visuals_mut().override_text_color = selector_color(check, ui.visuals());
                    // The combo spans the sidebar's width: the default `combo_width` (~100)
                    // overflows the rail and widens the layout region past its edge.
                    egui::ComboBox::from_id_salt("pes-version")
                        .width(ui.available_width())
                        .selected_text(version_label(picked, self.collapsed))
                        .show_ui(ui, |ui| {
                            for version in PesVersion::ALL {
                                ui.selectable_value(
                                    &mut picked,
                                    version,
                                    version_label(version, self.collapsed),
                                );
                            }
                        })
                        .response
                })
                .inner
            })
            .inner;
        // A disabled widget keeps no hover response of its own: the reason goes on the
        // selector's own response, which still reports the pointer over its rect (ST-SIDE-04).
        match blocker {
            Some((label, reason)) => {
                combo_response.on_disabled_hover_text(format!("{label}: {reason}"));
            }
            None => {
                combo_response.on_hover_text(tooltip);
            }
        }

        if picked != common.pes_version {
            ctx.settings().lock().unwrap().common.pes_version = picked;
            ctx.request(ShellRequest::SettingsChanged);
        }
    }

    /// One registered tool's entry: icon and label, the active one with the accent left bar
    /// and the filled background; collapsed, the icon alone with the label as the tooltip.
    fn tool_entry(
        &self,
        ui: &mut egui::Ui,
        icon: &str,
        label: &str,
        active: bool,
    ) -> egui::Response {
        let response = if self.collapsed {
            ui.add(egui::Button::selectable(active, icon))
        } else {
            let width = ui.available_width();
            ui.add_sized(
                [width, ui.spacing().interact_size.y],
                egui::Button::selectable(active, format!("{icon}  {label}")),
            )
        };
        // The rail draws the glyph alone; the entry's name stays the full label so tests and
        // screen readers find it unchanged in both modes.
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), active, label)
        });
        if active {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    response.rect.min,
                    egui::vec2(ACTIVE_BAR_WIDTH, response.rect.height()),
                ),
                0.0,
                ui.visuals().selection.stroke.color,
            );
        }
        if self.collapsed {
            response.on_hover_text(label)
        } else {
            response
        }
    }

    /// A planned tool the build does not register: grayed, never clickable, its tooltip
    /// saying it is not in this build (ST-SIDE-07).
    fn planned_entry(&self, ui: &mut egui::Ui, entry: &PlannedTool) {
        let text = if self.collapsed {
            entry.icon.to_owned()
        } else {
            format!("{}  {}", entry.icon, entry.label)
        };
        let response = ui.add_enabled(false, egui::Label::new(text).sense(egui::Sense::hover()));
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, false, entry.label));
        response.on_disabled_hover_text(format!("{} is not in this build", entry.label));
    }

    /// The exe check's verdict, listing the resolved PES folder once per change of the
    /// version or the path and once per return of the window's focus, never per frame
    /// (ST-SIDE-03: "a cheap single directory listing").
    fn exe_check(&mut self, common: &CommonSettings, focused: bool) -> ExeCheck {
        let folder = common.pes_folder();
        let stale = match &self.exe_check {
            Some((version, checked_folder, _)) => {
                *version != common.pes_version || *checked_folder != folder
            }
            None => true,
        };
        let gained_focus = focused && !self.was_focused;
        self.was_focused = focused;
        if stale || gained_focus {
            self.exe_listings += 1;
            let found = check_pes_folder(common.pes_version, &folder);
            self.exe_check = Some((common.pes_version, folder, found));
        }
        self.exe_check.as_ref().expect("a check ran above").2
    }
}

/// The combo's item label: `PES 2021` expanded, `21` on the rail (draft).
fn version_label(version: PesVersion, collapsed: bool) -> String {
    if collapsed {
        version.number().to_string()
    } else {
        version.to_string()
    }
}

/// The selector's colour for an exe-check verdict (ST-SIDE-03): red on a different
/// version's exe, yellow on no exe or no folder, none when the selection is installed.
pub(super) fn selector_color(check: ExeCheck, visuals: &egui::Visuals) -> Option<egui::Color32> {
    match check {
        ExeCheck::OtherVersion(_) => Some(visuals.error_fg_color),
        ExeCheck::NoExe | ExeCheck::NoFolder => Some(visuals.warn_fg_color),
        ExeCheck::Found => None,
    }
}

/// One directory listing of `folder` for `PES20NN.exe` names, in any letter case: the
/// selected version's own exe wins when several are present (ST-SIDE-03). Only files count:
/// the condition mirrors the compiler's preflight (`is_file` on the exe's path).
fn check_pes_folder(version: PesVersion, folder: &Path) -> ExeCheck {
    let entries = match std::fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(_) => return ExeCheck::NoFolder,
    };
    let mut other = None;
    for entry in entries.flatten() {
        // `Path::is_file` follows a link the same way the compiler's preflight does.
        if !entry.path().is_file() {
            continue;
        }
        let Some(found) = exe_version(&entry.file_name()) else {
            continue;
        };
        if found == version {
            return ExeCheck::Found;
        }
        if other.is_none() {
            other = Some(found);
        }
    }
    other.map_or(ExeCheck::NoExe, ExeCheck::OtherVersion)
}

/// `PES20NN.exe` in any letter case gives its version; anything else is not a PES exe.
fn exe_version(name: &OsStr) -> Option<PesVersion> {
    let lower = name.to_str()?.to_ascii_lowercase();
    let digits = lower.strip_prefix("pes20")?.strip_suffix(".exe")?;
    PesVersion::from_number(digits.parse::<u16>().ok()?)
}
