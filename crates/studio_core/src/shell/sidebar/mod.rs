//! The sidebar, top to bottom (core plan, "Sidebar"; draft `docs/prototypes/shell.html`):
//! the logo doubling as the About click target, the PES version selector with its live exe
//! check, its blocker and the once-per-life auto-switch to a running PES, the tool list
//! with the planned tools grayed, the collapse button switching the sidebar between
//! labels and an icon rail, and the help `?` with the settings gear pinned at the bottom.

use std::path::{Path, PathBuf};
use std::time::Duration;

use pes_version::PesVersion;

use crate::help::HelpTarget;
use crate::pes_process::RunningPes;
use crate::settings::CommonSettings;
use crate::status::{ActionEffect, Notice, NoticeAction};
use crate::tool::{PlannedTool, ShellRequest, StudioTool, ToolContext};

#[cfg(test)]
mod tests;

use super::about;

/// The sidebar's two widths (draft: 160 px expanded, 56 px as the icon rail).
const EXPANDED_WIDTH: f32 = 160.0;
const COLLAPSED_WIDTH: f32 = 56.0;
/// The accent bar on the active entry's left edge (draft: 3 px).
const ACTIVE_BAR_WIDTH: f32 = 3.0;
/// Seconds between running-PES polls of the auto-switch (gui.md "Sidebar" item 2).
const PES_POLL_SECONDS: f64 = 5.0;

/// What a click in the sidebar asks the shell to do.
pub(super) enum SidebarClick {
    /// Make the tool at this index the active view.
    SwitchTool(usize),
    /// Open the settings menu.
    OpenSettings,
    /// Open the help window at the target (the `?` targets the active tool's chapter,
    /// the About popup's Help link `setting-up`).
    OpenHelp(HelpTarget),
}

/// The sidebar's state across frames: the collapse, the exe check's last verdict, the
/// window's last focus (the check re-runs when focus returns), the logo's About popup and
/// the once-per-life auto-switch to a running PES.
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
    /// The auto-switch is once per program life: spent by the switch itself or by a manual
    /// pick, never by a blocker (ST-SIDE-05).
    pub(super) auto_switch_used: bool,
    /// The running-PES probe: `pes_process::running_pes` in the app, replaced by the tests.
    /// A plain `fn`, not a trait: there is one real probe.
    pub(super) probe: fn() -> Vec<RunningPes>,
    /// egui time of the next poll; `None` until `poll_running_pes` first runs.
    next_poll: Option<f64>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Sidebar {
            collapsed: false,
            exe_check: None,
            exe_listings: 0,
            was_focused: false,
            about_open: false,
            auto_switch_used: false,
            probe: crate::pes_process::running_pes,
            next_poll: None,
        }
    }
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

    /// Draws the sidebar and returns the click's ask, if any. `tools` are in sidebar
    /// order, `active` the index of the one on screen, `cli_only` the registered tools'
    /// ids drawn as planned entries (gui.md "Sidebar" item 3).
    pub(super) fn show(
        &mut self,
        ui: &mut egui::Ui,
        tools: &[Box<dyn StudioTool>],
        active: usize,
        planned: &[PlannedTool],
        cli_only: &[&'static str],
        ctx: &ToolContext,
    ) -> Option<SidebarClick> {
        let clicked = if self.collapsed {
            // The rail centers its icons (draft: `.sidebar.collapsed`).
            ui.vertical_centered(|ui| self.contents(ui, tools, active, planned, cli_only, ctx))
                .inner
        } else {
            self.contents(ui, tools, active, planned, cli_only, ctx)
        };
        if self.about_open {
            let help = about::show(
                ui.ctx(),
                ctx.paths(),
                ctx.common().pes_version,
                &mut self.about_open,
            );
            if help {
                return Some(SidebarClick::OpenHelp(HelpTarget {
                    tool: "setting-up".to_owned(),
                    topic: None,
                    anchor: None,
                }));
            }
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
        cli_only: &[&'static str],
        ctx: &ToolContext,
    ) -> Option<SidebarClick> {
        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("4cc")
                        .color(ui.visuals().selection.stroke.color)
                        .size(15.0),
                )
                .frame(false),
            )
            .on_hover_text(format!("About {}", crate::APP_NAME))
            .clicked()
        {
            self.about_open = true;
        }
        self.version_selector(ui, tools, ctx);
        ui.separator();

        let mut clicked = None;
        for (index, tool) in tools.iter().enumerate() {
            // A registered tool marked CLI-only is no tool entry: it is drawn with
            // the planned entries below (ST-SIDE-08).
            if cli_only.contains(&tool.id()) {
                continue;
            }
            if self
                .tool_entry(ui, tool.icon(), tool.label(), index == active)
                .clicked()
            {
                clicked = Some(SidebarClick::SwitchTool(index));
            }
        }
        for entry in planned {
            // In the registry's order: a window tool's own entry is drawn above; a
            // CLI-only tool's is here, its tooltip saying it runs from the command
            // line in this version (ST-SIDE-08).
            if tools
                .iter()
                .any(|tool| tool.id() == entry.id && !cli_only.contains(&tool.id()))
            {
                continue;
            }
            let tooltip = if cli_only.contains(&entry.id) {
                format!(
                    "{} has no window in this version: it runs from the command line.",
                    entry.label
                )
            } else {
                format!("{} is not in this build", entry.label)
            };
            self.planned_entry(ui, entry, &tooltip);
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

        // The bottom row, pinned to the sidebar's foot (the draft centers the rail's
        // tail): the help `?`, then the settings gear.
        let align = if self.collapsed {
            egui::Align::Center
        } else {
            egui::Align::Min
        };
        ui.with_layout(egui::Layout::bottom_up(align), |ui| {
            ui.horizontal(|ui| {
                let help = ui.add(egui::Button::new(egui_phosphor::regular::QUESTION).frame(false));
                help.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Help")
                });
                if help.on_hover_text("Help").clicked()
                    && let Some(tool) = tools.get(active)
                {
                    clicked = Some(SidebarClick::OpenHelp(HelpTarget {
                        tool: tool.id().to_owned(),
                        topic: None,
                        anchor: None,
                    }));
                }
                let gear = ui.add(egui::Button::new(egui_phosphor::regular::GEAR).frame(false));
                gear.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Settings")
                });
                if gear.on_hover_text("Settings").clicked() {
                    clicked = Some(SidebarClick::OpenSettings);
                }
            });
        });
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
        let blocker = version_blocker(tools);
        let folder = common.pes_folder();
        let check = self.exe_check(&common, ui.input(|input| input.focused));
        let tooltip = exe_tooltip(check, &folder);

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

        self.pick_version(picked, ctx);
    }

    /// A manual pick of the PES version, shared by the sidebar's selector and the settings
    /// menu's row: a change writes the common setting, spends the once-per-life
    /// auto-switch so the running-PES poll never fights it (ST-SIDE-05), and asks the
    /// shell to save.
    pub(super) fn pick_version(&mut self, version: PesVersion, ctx: &ToolContext) {
        if version == ctx.common().pes_version {
            return;
        }
        ctx.settings().lock().unwrap().common.pes_version = version;
        self.auto_switch_used = true;
        ctx.request(ShellRequest::SettingsChanged);
    }

    /// The once-per-life auto-switch to a running PES (gui.md "Sidebar" item 2, ST-SIDE-05),
    /// called from the shell's `logic` every `PES_POLL_SECONDS` on egui's clock — and only
    /// while the switch is unspent, so a spent switch polls nothing. Each call asks for the
    /// repaint at the next poll so it runs while no input arrives. Of the running games the
    /// first detected wins ("Sidebar" item 2): the poll switches only when that game's
    /// version differs from the selection, and a `version_change_blocker` defers the
    /// switch without spending it. A switch writes `pes_version` and saves like a manual
    /// pick, and announces itself in the status bar, with a "Fix PES path" action when the
    /// running exe's folder exists on this system and differs from `pes_folder_path`
    /// resolved for the new version.
    pub(super) fn poll_running_pes(
        &mut self,
        egui_ctx: &egui::Context,
        tools: &[Box<dyn StudioTool>],
        ctx: &ToolContext,
    ) {
        if self.auto_switch_used {
            return;
        }
        let now = egui_ctx.input(|input| input.time);
        let next = self.next_poll.get_or_insert(now + PES_POLL_SECONDS);
        if now < *next {
            egui_ctx.request_repaint_after(Duration::from_secs_f64(*next - now));
            return;
        }
        *next = now + PES_POLL_SECONDS;
        egui_ctx.request_repaint_after(Duration::from_secs_f64(PES_POLL_SECONDS));
        // The blocker check is cheaper than the process snapshot, so it runs first.
        if version_blocker(tools).is_some() {
            return;
        }
        let Some(game) = (self.probe)().into_iter().next() else {
            return;
        };
        if game.version == ctx.common().pes_version {
            return;
        }
        ctx.settings().lock().unwrap().common.pes_version = game.version;
        ctx.request(ShellRequest::SettingsChanged);
        self.auto_switch_used = true;
        // The notice offers the path fix only when the running exe sits elsewhere than the
        // folder the setting resolves for the version just switched to, and that folder
        // exists on this system: under Wine or Proton the probe reports the game's Windows
        // path, which `Path::parent` cannot reduce to a usable folder here.
        let action = game
            .exe
            .as_deref()
            .and_then(Path::parent)
            .and_then(|folder| {
                (folder.is_dir() && folder != ctx.common().pes_folder()).then(|| NoticeAction {
                    label: "Fix PES path".to_owned(),
                    effect: ActionEffect::FixPesPath {
                        folder: folder.to_path_buf(),
                        version: game.version,
                    },
                })
            });
        ctx.notify(Notice {
            text: format!("Switched to PES {}", game.version.year()),
            action,
        });
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

    /// A tool the sidebar lists grayed and never clickable: one the build does not
    /// register (ST-SIDE-07) or a registered one marked CLI-only (ST-SIDE-08);
    /// `tooltip` says which.
    fn planned_entry(&self, ui: &mut egui::Ui, entry: &PlannedTool, tooltip: &str) {
        let text = if self.collapsed {
            entry.icon.to_owned()
        } else {
            format!("{}  {}", entry.icon, entry.label)
        };
        let response = ui.add_enabled(false, egui::Label::new(text).sense(egui::Sense::hover()));
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, false, entry.label));
        response.on_disabled_hover_text(tooltip);
    }

    /// The exe check's verdict, listing the resolved PES folder once per change of the
    /// version or the path and once per return of the window's focus, never per frame
    /// (ST-SIDE-03: "a cheap single directory listing"). The settings menu's version row
    /// shares it: the check is one cache for the shell.
    pub(super) fn exe_check(&mut self, common: &CommonSettings, focused: bool) -> ExeCheck {
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

/// The first `version_change_blocker` any registered tool reports, as the blocking tool's
/// label and its reason for the disabled tooltip (ST-SIDE-04); `None` while nothing
/// blocks.
pub(super) fn version_blocker(tools: &[Box<dyn StudioTool>]) -> Option<(&'static str, &str)> {
    tools.iter().find_map(|tool| {
        tool.version_change_blocker()
            .map(|reason| (tool.label(), reason))
    })
}

/// The version selector's enabled-state tooltip for an exe-check verdict (ST-SIDE-03):
/// what the last listing of the resolved PES folder found. The settings menu's version
/// row carries the same one (the draft's `pesSelectorTip`).
pub(super) fn exe_tooltip(check: ExeCheck, folder: &Path) -> String {
    match check {
        ExeCheck::Found => format!("{}\nThe PES version every tool targets", folder.display()),
        ExeCheck::OtherVersion(found) => format!(
            "Found PES{}.exe in {}: a different version than the selection",
            found.year(),
            folder.display()
        ),
        ExeCheck::NoExe => format!("No PES exe in {}", folder.display()),
        ExeCheck::NoFolder => format!("{} does not exist", folder.display()),
    }
}

/// The combo's item label: `PES 2021` expanded, `21` on the rail (draft).
pub(super) fn version_label(version: PesVersion, collapsed: bool) -> String {
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
        let Some(found) = entry
            .file_name()
            .to_str()
            .and_then(crate::pes_process::version_of_exe)
        else {
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
