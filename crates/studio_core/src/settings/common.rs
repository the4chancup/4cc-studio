//! The common settings: what every tool shares (core plan, "Settings menu"; Team compiler plan,
//! "Common settings").

use std::path::PathBuf;

use pes_version::PesVersion;
use serde::{Deserialize, Serialize};

/// The default PES folder, with `**` standing for the two-digit version.
#[cfg(windows)]
const DEFAULT_PES_FOLDER: &str = r"C:\Program Files (x86)\Pro Evolution Soccer 20**";
/// No sensible default exists off Windows; the user sets it.
#[cfg(not(windows))]
const DEFAULT_PES_FOLDER: &str = "";

/// Settings shared by the shell and every tool, stored under `[common]` in the settings file.
/// Every field has a default, so a missing key loads as its default (this is how a new version's
/// settings appear in an old file).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CommonSettings {
    /// The PES version the suite targets; the sidebar selector.
    pub pes_version: PesVersion,
    /// The PES installation folder, with `**` standing for the two-digit version (`20**`).
    pub pes_folder_path: String,
    /// Where exports are picked up. A relative path resolves beside the executable in both
    /// data-location modes, so it sits next to `quick_compile.bat`.
    pub exports_folder_path: PathBuf,
    /// Worker threads for pipelines; `0` means automatic (logical cores minus one).
    pub thread_count: usize,
    /// Memory budget cap for pipelines, as a percentage of physical memory.
    pub memory_cap_percent: f32,
    /// GUI color theme.
    pub theme: Theme,
    /// Whether the shell looks for new releases.
    pub check_for_updates: bool,
    /// Hours between release checks.
    pub check_interval_hours: u32,
    /// When the last release check ran, in seconds since the Unix epoch.
    pub last_update_check: Option<u64>,
    /// A release the user chose to skip; the updater stays quiet about it.
    pub skipped_version: Option<String>,
    /// The tool active when the GUI last closed; `studio` with no arguments reopens it.
    pub last_tool: Option<String>,
    /// The main window class's last place (gui.md "Window size classes"), written when the
    /// window closes; `None` until the first close, which opens the window fitted and
    /// centered.
    pub window_main: Option<WindowPlace>,
    /// The boots ID the FPC hide preset writes (the `fpc` crate's `HideIds`), an ID the
    /// game has no boots model for.
    pub fpc_hide_boots_id: u16,
    /// The gloves ID it writes; gloves 1 to 10 are the goalkeepers' real gloves.
    pub fpc_hide_gloves_id: u16,
}

impl Default for CommonSettings {
    fn default() -> Self {
        CommonSettings {
            pes_version: PesVersion::Pes19,
            pes_folder_path: DEFAULT_PES_FOLDER.to_owned(),
            exports_folder_path: PathBuf::from("exports"),
            thread_count: 0,
            memory_cap_percent: 80.0,
            theme: Theme::Dark,
            check_for_updates: true,
            check_interval_hours: 24,
            last_update_check: None,
            skipped_version: None,
            last_tool: None,
            window_main: None,
            fpc_hide_boots_id: fpc::HideIds::CONVENTION.boots(),
            fpc_hide_gloves_id: fpc::HideIds::CONVENTION.gloves(),
        }
    }
}

impl CommonSettings {
    /// The PES installation folder: `pes_folder_path` with every `**` replaced by the two-digit
    /// number of `pes_version` (`21` for PES 2021).
    pub fn pes_folder(&self) -> PathBuf {
        let version = self.pes_version.number().to_string();
        PathBuf::from(self.pes_folder_path.replace("**", &version))
    }

    /// The FPC hide preset's boots and gloves IDs the settings give, or the
    /// `HideIdsError` naming the one outside its band: a value a member set by
    /// hand is not clamped (the settings menu's drag range bounds only what it
    /// enters), so an out-of-band pair is an error the consumers surface.
    pub fn fpc_hide_ids(&self) -> Result<fpc::HideIds, fpc::HideIdsError> {
        fpc::HideIds::new(self.fpc_hide_boots_id, self.fpc_hide_gloves_id)
    }
}

/// A window's remembered place: its outer position in the screen's physical pixels, its
/// inner size in points at zoom 1 (the pixels over the monitor's scale), and the scale
/// factor of the monitor it was on, written in the settings file as
/// `[x, y, width, height, scale]` (gui.md "Window size classes" says why each unit).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f32; 5]", into = "[f32; 5]")]
pub struct WindowPlace {
    /// The outer rectangle's left edge, in physical pixels.
    pub x: f32,
    /// The outer rectangle's top edge, in physical pixels.
    pub y: f32,
    /// The inner width in points at zoom 1.
    pub width: f32,
    /// The inner height in points at zoom 1.
    pub height: f32,
    /// The monitor's scale factor (native pixels per point) when the place was recorded.
    pub scale: f32,
}

impl From<[f32; 5]> for WindowPlace {
    fn from([x, y, width, height, scale]: [f32; 5]) -> WindowPlace {
        WindowPlace {
            x,
            y,
            width,
            height,
            scale,
        }
    }
}

impl From<WindowPlace> for [f32; 5] {
    fn from(place: WindowPlace) -> [f32; 5] {
        [place.x, place.y, place.width, place.height, place.scale]
    }
}

impl WindowPlace {
    /// The outer position, in physical pixels.
    pub fn position(&self) -> egui::Pos2 {
        egui::pos2(self.x, self.y)
    }

    /// The inner size, in points at zoom 1.
    pub fn size(&self) -> egui::Vec2 {
        egui::vec2(self.width, self.height)
    }
}

/// GUI color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Dark background (egui's default).
    Dark,
    /// Light background.
    Light,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(version: PesVersion, path: &str) -> CommonSettings {
        CommonSettings {
            pes_version: version,
            pes_folder_path: path.to_owned(),
            ..CommonSettings::default()
        }
    }

    #[cfg(windows)]
    #[test]
    fn the_default_pes_folder_names_the_default_version() {
        assert_eq!(
            CommonSettings::default().pes_folder(),
            PathBuf::from(r"C:\Program Files (x86)\Pro Evolution Soccer 2019")
        );
    }

    #[test]
    fn the_placeholder_is_replaced_by_the_two_digit_version() {
        assert_eq!(
            settings(PesVersion::Pes21, "/games/Pro Evolution Soccer 20**").pes_folder(),
            PathBuf::from("/games/Pro Evolution Soccer 2021")
        );
    }

    #[test]
    fn a_path_with_no_placeholder_is_unchanged() {
        assert_eq!(
            settings(PesVersion::Pes21, "/games/PES").pes_folder(),
            PathBuf::from("/games/PES")
        );
    }

    // The defaults the spec pins: `spec/team_compiler/settings.md`'s "Common
    // settings (studio_core)" table (version 19, the `20**` folder, `exports/`,
    // 0 auto threads, 80.0, the update check) and `spec/core/distribution.md`'s
    // "Update state" table (`check_for_updates` true, `check_interval_hours` 24,
    // `last_update_check` and `skipped_version` empty). `theme` is the dark
    // preset, the default the GUI phase plans, `last_tool` empty on a first
    // run, `window_main` empty until the first close. The hide IDs are the FPC
    // convention's 55 and 11 (`fpc::HideIds::CONVENTION`).
    #[test]
    fn the_defaults_match_the_settings_tables() {
        // `pes_folder_path` is the shipped pattern on Windows and empty
        // elsewhere: no sensible default exists off Windows.
        #[cfg(windows)]
        let pes_folder_path = r"C:\Program Files (x86)\Pro Evolution Soccer 20**";
        #[cfg(not(windows))]
        let pes_folder_path = "";
        assert_eq!(
            CommonSettings::default(),
            CommonSettings {
                pes_version: PesVersion::Pes19,
                pes_folder_path: pes_folder_path.to_owned(),
                exports_folder_path: PathBuf::from("exports"),
                thread_count: 0,
                memory_cap_percent: 80.0,
                theme: Theme::Dark,
                check_for_updates: true,
                check_interval_hours: 24,
                last_update_check: None,
                skipped_version: None,
                last_tool: None,
                window_main: None,
                fpc_hide_boots_id: 55,
                fpc_hide_gloves_id: 11,
            }
        );
    }

    // A settings file whose `[common]` lacks both keys loads the
    // convention's 55 and 11 (the struct's `serde(default)`); a key set by
    // hand loads as written, clamped nowhere, and `fpc_hide_ids` reports
    // the band the value left.
    #[test]
    fn the_hide_ids_default_to_the_convention_and_load_as_written() {
        let settings = crate::settings::Settings::parse("[common]\npes_version = 21\n")
            .expect("parses")
            .common;
        assert_eq!(settings.fpc_hide_boots_id, 55);
        assert_eq!(settings.fpc_hide_gloves_id, 11);
        assert_eq!(
            settings.fpc_hide_ids(),
            Ok(fpc::HideIds::CONVENTION),
            "the defaults resolve to the convention"
        );

        let settings = crate::settings::Settings::parse(
            "[common]\nfpc_hide_boots_id = 627\nfpc_hide_gloves_id = 12\n",
        )
        .expect("parses")
        .common;
        assert_eq!(settings.fpc_hide_boots_id, 627, "as written, no clamp");
        assert_eq!(settings.fpc_hide_gloves_id, 12);
        assert_eq!(settings.fpc_hide_ids(), Err(fpc::HideIdsError::Boots(627)));
    }

    // The window's place reads as the five numbers a member might write and
    // writes back the same way, its key inside `[common]`.
    #[test]
    fn a_window_place_loads_from_five_numbers_and_saves_as_five() {
        let settings =
            crate::settings::Settings::parse("[common]\nwindow_main = [100, 50, 1100, 700, 1]\n")
                .expect("parses");
        let place = WindowPlace {
            x: 100.0,
            y: 50.0,
            width: 1100.0,
            height: 700.0,
            scale: 1.0,
        };
        assert_eq!(settings.common.window_main, Some(place));

        let dir = std::env::temp_dir().join(format!(
            "studio_core_common_window_place_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.toml");
        let mut settings = crate::settings::Settings::default();
        settings.common.window_main = Some(place);
        settings.save(&path).unwrap();
        let loaded = crate::settings::Settings::load(&path).unwrap();
        assert_eq!(loaded.common.window_main, Some(place));

        let text = std::fs::read_to_string(&path).unwrap();
        let common_at = text.find("[common]").expect("a [common] section");
        let key_at = text.find("window_main").expect("the place is written");
        assert!(key_at > common_at, "the key sits inside [common]");
        if let Some(next) = text[common_at..].find("\n[").map(|i| common_at + i) {
            assert!(key_at < next, "the key sits before the next section");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
