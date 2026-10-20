use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use egui::accesskit::{self, Toggled};
use pes_version::PesVersion;

use super::{ExeCheck, selector_color};
use crate::pes_process::RunningPes;
use crate::settings::Settings;
use crate::shell::StudioApp;
use crate::shell::about;
use crate::shell::test_harness::*;
use crate::tool::PlannedTool;

// TC-GUI-03
#[test]
fn the_sidebar_marks_the_active_tool_and_a_click_on_an_entry_switches_to_it() {
    let Shell { mut app, calls, .. } = shell();
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();

    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    assert_eq!(
        (calls[0].views(), calls[1].views()),
        (1, 0),
        "only the active view runs"
    );
    assert_eq!(entry(&nodes, "A").toggled(), Some(Toggled::True));
    assert_eq!(entry(&nodes, "B").toggled(), Some(Toggled::False));

    // egui hit-tests a click against the previous frame's widgets: the press comes a frame
    // after the layout, the release a frame after the press.
    let press = pointer(entry(&nodes, "B"), true);
    let release = pointer(entry(&nodes, "B"), false);
    frame(&egui_ctx, &mut app, press, true);
    frame(&egui_ctx, &mut app, release, true);
    assert_eq!(app.active, 1);
    assert_eq!(
        (calls[0].views(), calls[1].views()),
        (2, 1),
        "B's view ran on the click frame"
    );

    // The sidebar is drawn before the click is applied, so the entries show the switch on
    // the next frame.
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    assert_eq!(entry(&nodes, "A").toggled(), Some(Toggled::False));
    assert_eq!(entry(&nodes, "B").toggled(), Some(Toggled::True));
    assert_eq!((calls[0].views(), calls[1].views()), (2, 2));
}

// ST-SIDE-01
#[test]
fn the_gui_opens_on_the_tool_last_tool_names_and_lists_all_tools() {
    let (a, calls_a) = stub("a", "A", None);
    let (b, calls_b) = stub("b", "B", None);
    let (c, calls_c) = stub("c", "C", None);
    let mut settings = test_settings(&test_dir("studio_core_sidebar_last_named"));
    settings.common.last_tool = Some("b".to_owned());
    let Shell { mut app, calls, .. } = shell_with(
        settings,
        vec![a, b, c],
        &[],
        None,
        vec![calls_a, calls_b, calls_c],
    );
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();

    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    assert_eq!(app.active, 1, "last_tool names the second tool");
    assert_eq!(
        (calls[0].views(), calls[1].views(), calls[2].views()),
        (0, 1, 0),
        "only the second tool's view is drawn"
    );
    // The full labels, in registration order (drawn top to bottom).
    let top = |label: &str| entry(&nodes, label).bounds().unwrap().y0;
    assert!(top("A") < top("B") && top("B") < top("C"), "registry order");
    assert_eq!(entry(&nodes, "B").toggled(), Some(Toggled::True));
}

// ST-SIDE-02
#[test]
fn the_collapse_button_turns_the_sidebar_into_a_working_icon_rail() {
    let Shell { mut app, calls, .. } = shell();
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();
    egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

    // Expanded: the entries are full-width rows.
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    let expanded_width = entry(&nodes, "A").bounds().unwrap().width();
    assert!(expanded_width > 60.0, "expanded entries span the sidebar");

    click(&egui_ctx, &mut app, entry(&nodes, "«"));
    idle(&egui_ctx, &mut app, 10);
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    let rail_width = entry(&nodes, "A").bounds().unwrap().width();
    assert!(
        rail_width < expanded_width / 2.0,
        "collapsed entries shrink to the icon ({rail_width} < {})",
        expanded_width / 2.0
    );

    // The rail entry's tooltip is its label.
    frame(&egui_ctx, &mut app, hover(entry(&nodes, "B")), true);
    // `is_still` takes about 0.1 s of frames with no pointer movement.
    let nodes = idle(&egui_ctx, &mut app, 8).nodes;
    assert!(
        has_text(&nodes, "B"),
        "the rail entry's tooltip carries the label"
    );

    // A click on a rail entry still switches the tool.
    click(&egui_ctx, &mut app, entry(&nodes, "B"));
    assert_eq!(app.active, 1);
    assert!(calls[1].views() >= 1, "B's view runs");

    // A second click on the collapse button restores the labels.
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    click(&egui_ctx, &mut app, entry(&nodes, "»"));
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    assert_eq!(
        entry(&nodes, "A").bounds().unwrap().width(),
        expanded_width,
        "the labels are back"
    );
}

// ST-SIDE-03
#[test]
fn the_selector_is_colored_by_the_exe_found_in_the_pes_folder() {
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();
    egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

    // A shell at PES 21 with `data_dir` unset; `pes_folder_path` is the given folder.
    let with_pes_folder = |folder: &PathBuf| {
        let (a, calls_a) = stub("a", "A", None);
        let mut settings = Settings::default();
        settings.common.pes_version = PesVersion::Pes21;
        settings.common.pes_folder_path = folder.display().to_string();
        shell_with(settings, vec![a], &[], None, vec![calls_a])
    };
    let hover_selector = |app: &mut StudioApp| {
        let nodes = frame(&egui_ctx, app, Vec::new(), true).nodes;
        frame(&egui_ctx, app, hover(combo(&nodes)), true);
        // `is_still` takes about 0.1 s of frames with no pointer movement.
        idle(&egui_ctx, app, 8).nodes
    };
    let visuals = egui::Visuals::dark();
    let color_of =
        |app: &StudioApp| selector_color(app.sidebar.exe_check.as_ref().unwrap().2, &visuals);

    // Another version's exe: red, the tooltip names what was found.
    let dir = test_dir("studio_core_sidebar_pes19");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("PES2019.exe"), b"").unwrap();
    let Shell { mut app, .. } = with_pes_folder(&dir);
    frame(&egui_ctx, &mut app, Vec::new(), true);
    assert_eq!(
        app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
        Some(ExeCheck::OtherVersion(PesVersion::Pes19))
    );
    assert_eq!(color_of(&app), Some(visuals.error_fg_color), "red");
    let nodes = hover_selector(&mut app);
    let expected = format!("Found PES2019.exe in {}", dir.display());
    assert!(
        has_text(&nodes, &expected),
        "the tooltip names the exe found"
    );
    fs::remove_dir_all(&dir).unwrap();

    // No PES exe: yellow, the tooltip says none was found.
    let dir = test_dir("studio_core_sidebar_noexe");
    fs::create_dir_all(&dir).unwrap();
    let Shell { mut app, .. } = with_pes_folder(&dir);
    frame(&egui_ctx, &mut app, Vec::new(), true);
    assert_eq!(
        app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
        Some(ExeCheck::NoExe)
    );
    assert_eq!(color_of(&app), Some(visuals.warn_fg_color), "yellow");
    let nodes = hover_selector(&mut app);
    assert!(has_text(
        &nodes,
        &format!("No PES exe in {}", dir.display())
    ));
    fs::remove_dir_all(&dir).unwrap();

    // A name the games never ship is not an exe either: `PES20021.exe` is not PES 21's
    // (`PES2021.exe`), so the folder reads as no exe at all.
    let dir = test_dir("studio_core_sidebar_pes20021");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("PES20021.exe"), b"").unwrap();
    let Shell { mut app, .. } = with_pes_folder(&dir);
    frame(&egui_ctx, &mut app, Vec::new(), true);
    assert_eq!(
        app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
        Some(ExeCheck::NoExe)
    );
    assert_eq!(color_of(&app), Some(visuals.warn_fg_color), "yellow");
    fs::remove_dir_all(&dir).unwrap();

    // A directory named like the exe is not an exe (the compiler's preflight
    // counts files only): yellow like an empty folder.
    let dir = test_dir("studio_core_sidebar_dir_named_exe");
    fs::create_dir_all(dir.join("PES2021.exe")).unwrap();
    let Shell { mut app, .. } = with_pes_folder(&dir);
    frame(&egui_ctx, &mut app, Vec::new(), true);
    assert_eq!(
        app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
        Some(ExeCheck::NoExe)
    );
    assert_eq!(color_of(&app), Some(visuals.warn_fg_color), "yellow");
    fs::remove_dir_all(&dir).unwrap();

    // The folder missing: yellow too, the tooltip names it.
    let dir = test_dir("studio_core_sidebar_missing");
    let Shell { mut app, .. } = with_pes_folder(&dir);
    frame(&egui_ctx, &mut app, Vec::new(), true);
    assert_eq!(
        app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
        Some(ExeCheck::NoFolder)
    );
    assert_eq!(color_of(&app), Some(visuals.warn_fg_color), "yellow");
    let nodes = hover_selector(&mut app);
    assert!(has_text(
        &nodes,
        &format!("{} does not exist", dir.display())
    ));

    // The selected version's exe: no colour.
    let dir = test_dir("studio_core_sidebar_pes21");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("PES2021.exe"), b"").unwrap();
    let Shell { mut app, .. } = with_pes_folder(&dir);
    frame(&egui_ctx, &mut app, Vec::new(), true);
    assert_eq!(
        app.sidebar.exe_check.as_ref().map(|(_, _, check)| *check),
        Some(ExeCheck::Found)
    );
    assert_eq!(color_of(&app), None, "no colour");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_exe_check_relists_on_a_change_or_on_focus_never_per_frame() {
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();
    let dir = test_dir("studio_core_sidebar_recheck");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("PES2021.exe"), b"").unwrap();

    let (a, calls_a) = stub("a", "A", None);
    let mut settings = Settings::default();
    settings.common.pes_version = PesVersion::Pes21;
    settings.common.pes_folder_path = dir.display().to_string();
    let Shell {
        mut app, settings, ..
    } = shell_with(settings, vec![a], &[], None, vec![calls_a]);

    frame(&egui_ctx, &mut app, Vec::new(), true);
    frame(&egui_ctx, &mut app, Vec::new(), true);
    assert_eq!(app.sidebar.exe_listings, 1, "no re-list per frame");

    // The window's focus returning re-lists.
    frame(&egui_ctx, &mut app, Vec::new(), false);
    frame(&egui_ctx, &mut app, Vec::new(), true);
    assert_eq!(app.sidebar.exe_listings, 2, "focus regained re-lists");

    // A version or path change re-lists.
    settings.lock().unwrap().common.pes_version = PesVersion::Pes19;
    frame(&egui_ctx, &mut app, Vec::new(), true);
    assert_eq!(app.sidebar.exe_listings, 3, "the version changed");
    fs::remove_dir_all(&dir).unwrap();
}

// ST-SIDE-04
#[test]
fn a_version_change_blocker_disables_the_selector_and_a_pick_saves_the_version() {
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();
    egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

    // While a tool blocks, the selector is disabled and its tooltip is the reason.
    let (a, calls_a) = stub("a", "A", Some("a compile is running"));
    let (b, calls_b) = stub("b", "B", None);
    let dir = test_dir("studio_core_sidebar_blocked");
    fs::create_dir_all(&dir).unwrap();
    let Shell {
        mut app, settings, ..
    } = shell_with(
        test_settings(&dir),
        vec![a, b],
        &[],
        Some(dir.clone()),
        vec![calls_a, calls_b],
    );
    let version_before = settings.lock().unwrap().common.pes_version;
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    assert!(combo(&nodes).is_disabled(), "the selector is disabled");
    frame(&egui_ctx, &mut app, hover(combo(&nodes)), true);
    let nodes = idle(&egui_ctx, &mut app, 8).nodes;
    assert!(
        has_text(&nodes, "A: a compile is running"),
        "the tooltip is the blocking tool's reason"
    );
    assert_eq!(
        settings.lock().unwrap().common.pes_version,
        version_before,
        "the setting keeps its value"
    );

    // Unblocked: a pick writes the settings file and the tools read it next frame.
    let (a, calls_a) = stub("a", "A", None);
    let (b, calls_b) = stub("b", "B", None);
    let Shell {
        mut app,
        settings,
        calls,
        ..
    } = shell_with(
        test_settings(&dir),
        vec![a, b],
        &[],
        Some(dir.clone()),
        vec![calls_a, calls_b],
    );
    assert_ne!(
        Settings::default().common.pes_version,
        PesVersion::Pes21,
        "the test's pick really changes the setting"
    );
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    click(&egui_ctx, &mut app, combo(&nodes));
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    click(&egui_ctx, &mut app, entry(&nodes, "PES 2021"));
    assert_eq!(
        settings.lock().unwrap().common.pes_version,
        PesVersion::Pes21
    );
    // The save is applied at the next frame's logic.
    frame(&egui_ctx, &mut app, Vec::new(), true);
    let written = Settings::load(&dir.join(crate::settings::SETTINGS_FILE_NAME)).unwrap();
    assert_eq!(written.common.pes_version, PesVersion::Pes21);
    assert_eq!(calls[0].version_seen(), Some(PesVersion::Pes21));
    assert_eq!(calls[1].version_seen(), Some(PesVersion::Pes21));
    fs::remove_dir_all(&dir).unwrap();
}

// ST-SIDE-07
#[test]
fn the_planned_tools_follow_the_registered_ones_grayed_and_inert() {
    const PLANNED: &[PlannedTool] = &[
        PlannedTool {
            id: "a",
            label: "A",
            icon: "a",
        },
        PlannedTool {
            id: "planned-one",
            label: "Planned one",
            icon: "p",
        },
        PlannedTool {
            id: "planned-two",
            label: "Planned two",
            icon: "q",
        },
    ];
    let (a, calls_a) = stub("a", "A", None);
    let (b, calls_b) = stub("b", "B", None);
    let Shell { mut app, .. } = shell_with(
        test_settings(&test_dir("studio_core_sidebar_planned")),
        vec![a, b],
        PLANNED,
        None,
        vec![calls_a, calls_b],
    );
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();
    egui_ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);

    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    // The planned entries are label-role nodes: the text sits in `value`.
    let planned = |label: &str| {
        nodes
            .iter()
            .find(|node| node.value() == Some(label))
            .unwrap_or_else(|| panic!("no text node {label:?}"))
    };
    let top = |node: &accesskit::Node| node.bounds().unwrap().y0;
    assert!(
        top(entry(&nodes, "B")) < top(planned("Planned one"))
            && top(planned("Planned one")) < top(planned("Planned two")),
        "the planned entries follow the registered ones, in registry order"
    );
    assert!(
        planned("Planned one").is_disabled(),
        "the planned entry is grayed"
    );

    // Not clickable: a click changes nothing.
    click(&egui_ctx, &mut app, planned("Planned one"));
    assert_eq!(app.active, 0);

    // Its tooltip says the tool is not in this build.
    idle(&egui_ctx, &mut app, 10);
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    let node = nodes
        .iter()
        .find(|node| node.value() == Some("Planned one"))
        .unwrap();
    frame(&egui_ctx, &mut app, hover(node), true);
    let nodes = idle(&egui_ctx, &mut app, 8).nodes;
    assert!(
        has_text(&nodes, "Planned one is not in this build"),
        "the tooltip names the tool and the build"
    );
}

// The logo's About block and Copy (ST-SIDE-06's help link arrives with the help window).
#[test]
fn the_logo_opens_about_and_copy_puts_the_version_block_on_the_clipboard() {
    let dir = test_dir("studio_core_sidebar_about");
    fs::create_dir_all(&dir).unwrap();
    let (a, calls_a) = stub("a", "A", None);
    let mut settings = test_settings(&dir);
    settings.common.pes_version = PesVersion::Pes21;
    let Shell { mut app, .. } =
        shell_with(settings, vec![a], &[], Some(dir.clone()), vec![calls_a]);
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();

    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    click(&egui_ctx, &mut app, entry(&nodes, "4cc"));

    let out = frame(&egui_ctx, &mut app, Vec::new(), true);
    let block = format!(
        "4cc Studio v{}\nBuild: {}\nTarget: {}\nData location: {}\nPES version: PES 2021",
        env!("CARGO_PKG_VERSION"),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        env!("STUDIO_TARGET_TRIPLE"),
        dir.display(),
    );
    assert!(
        out.nodes
            .iter()
            .any(|node| node.value() == Some(block.as_str())),
        "the version block is shown"
    );
    entry(&out.nodes, "Release page");
    entry(&out.nodes, "Repository");

    // The links open their URLs.
    let out = click(&egui_ctx, &mut app, entry(&out.nodes, "Repository"));
    assert!(
        out.platform
            .commands
            .contains(&egui::OutputCommand::OpenUrl(egui::OpenUrl::same_tab(
                about::REPOSITORY_URL
            ))),
        "the repository link opens {}",
        env!("CARGO_PKG_REPOSITORY")
    );

    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    let out = click(&egui_ctx, &mut app, entry(&nodes, "Release page"));
    assert!(
        out.platform
            .commands
            .contains(&egui::OutputCommand::OpenUrl(egui::OpenUrl::same_tab(
                about::RELEASE_PAGE_URL
            ))),
        "the release-page link opens the releases listing"
    );

    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    let out = click(&egui_ctx, &mut app, entry(&nodes, "Copy"));
    assert!(
        out.platform
            .commands
            .contains(&egui::OutputCommand::CopyText(block)),
        "Copy puts exactly the version block on the clipboard"
    );

    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    click(&egui_ctx, &mut app, entry(&nodes, "Close"));
    assert!(!app.sidebar.about_open, "Close dismisses the popup");
    fs::remove_dir_all(&dir).unwrap();
}

// ST-SIDE-05 is manual: a real PES must run. These cover the poll's mechanics with a
// stubbed probe, so they cite no scenario.

/// The folder `<dir>/running/Pro Evolution Soccer 2019` and its `PES2019.exe` path; the
/// names the running game really uses.
fn running_pes_2019(dir: &std::path::Path) -> (PathBuf, PathBuf) {
    let folder = dir.join("running").join("Pro Evolution Soccer 2019");
    let exe = folder.join("PES2019.exe");
    (folder, exe)
}

#[test]
fn the_poll_switches_to_the_running_version_once_and_offers_fix_pes_path() {
    let dir = test_dir("studio_core_autoswitch");
    fs::create_dir_all(dir.join("data")).unwrap();
    let (running_folder, running_exe) = running_pes_2019(&dir);
    // The action is offered only for a folder that exists on this system: the running
    // install's own, exe included.
    fs::create_dir_all(&running_folder).unwrap();
    fs::write(&running_exe, b"").unwrap();

    // The probe is a `fn`, so the exe path is rebuilt inside it (a closure could not be
    // one): `test_dir` gives the same path.
    fn probe() -> Vec<RunningPes> {
        vec![RunningPes {
            version: PesVersion::Pes19,
            exe: Some(running_pes_2019(&test_dir("studio_core_autoswitch")).1),
        }]
    }

    let (a, calls_a) = stub("a", "A", None);
    let mut settings = test_settings(&dir);
    settings.common.pes_version = PesVersion::Pes21;
    settings.common.pes_folder_path = dir
        .join("installed")
        .join("Pro Evolution Soccer 20**")
        .display()
        .to_string();
    let Shell { mut app, ctx, .. } = shell_with(
        settings,
        vec![a],
        &[],
        Some(dir.join("data")),
        vec![calls_a],
    );
    app.sidebar.probe = probe;
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();
    assert_ne!(
        running_exe.parent(),
        Some(ctx.common().pes_folder().as_path()),
        "the running exe sits outside the resolved folder (the action's premise)"
    );

    // No poll before 5 s have passed.
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(0.0));
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(4.9));
    assert_eq!(
        ctx.common().pes_version,
        PesVersion::Pes21,
        "no poll before the interval"
    );

    // The switch's requests apply in the frame that switches: at t = 5 s the notice is
    // already drawn and the file already saved, no later frame needed.
    let out = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(5.0));
    assert_eq!(
        ctx.common().pes_version,
        PesVersion::Pes19,
        "the poll switched the selection"
    );
    assert!(app.sidebar.auto_switch_used, "the switch is spent");
    let file = dir.join("data").join(crate::settings::SETTINGS_FILE_NAME);
    assert!(
        file.is_file(),
        "the switching frame wrote the settings file"
    );
    let written = Settings::load(&file).unwrap();
    assert_eq!(
        written.common.pes_version,
        PesVersion::Pes19,
        "the switch saved in the switching frame"
    );
    assert!(
        has_text(&out.nodes, "Switched to PES 2019"),
        "the notice shows in the switching frame"
    );

    // The exe's folder differs from the resolved one, so the action is offered; clicking it
    // rewrites pes_folder_path from the running folder with the year back to `20**`.
    let fixed = running_folder.with_file_name("Pro Evolution Soccer 20**");
    click(&egui_ctx, &mut app, entry(&out.nodes, "Fix PES path"));
    assert_eq!(
        std::path::Path::new(&ctx.common().pes_folder_path),
        fixed.as_path(),
        "the ** placeholder is restored"
    );
    frame(&egui_ctx, &mut app, Vec::new(), true);
    let written =
        Settings::load(&dir.join("data").join(crate::settings::SETTINGS_FILE_NAME)).unwrap();
    assert_eq!(written.common.pes_folder_path, fixed.display().to_string());
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    assert!(
        !has_text(&nodes, "Switched to PES 2019"),
        "the click dismissed the notice"
    );

    // A manual pick back to PES 2021 is not fought at the next polls.
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    click(&egui_ctx, &mut app, combo(&nodes));
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    click(&egui_ctx, &mut app, entry(&nodes, "PES 2021"));
    assert_eq!(ctx.common().pes_version, PesVersion::Pes21);
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(15.0));
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(25.0));
    assert_eq!(
        ctx.common().pes_version,
        PesVersion::Pes21,
        "the spent switch never fires again"
    );
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_version_change_blocker_defers_the_auto_switch_without_spending_it() {
    fn probe() -> Vec<RunningPes> {
        vec![RunningPes {
            version: PesVersion::Pes19,
            exe: None,
        }]
    }
    let (a, calls_a) = stub("a", "A", Some("a compile is running"));
    let mut settings = test_settings(&test_dir("studio_core_autoswitch_blocked"));
    settings.common.pes_version = PesVersion::Pes21;
    let Shell {
        mut app,
        ctx,
        calls,
        ..
    } = shell_with(settings, vec![a], &[], None, vec![calls_a]);
    app.sidebar.probe = probe;
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();

    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(0.0));
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(5.0));
    assert_eq!(
        ctx.common().pes_version,
        PesVersion::Pes21,
        "the blocker deferred the switch"
    );
    assert!(
        !app.sidebar.auto_switch_used,
        "a deferred switch is not spent"
    );

    *calls[0].blocker.lock().unwrap() = None;
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(10.0));
    assert_eq!(
        ctx.common().pes_version,
        PesVersion::Pes19,
        "the switch happens once the blocker clears"
    );
}

#[test]
fn the_switch_notice_offers_fix_pes_path_only_when_the_exe_sits_elsewhere() {
    let dir = test_dir("studio_core_autoswitch_notice");
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();

    // The probe found no exe path: the notice has no action.
    fn no_exe() -> Vec<RunningPes> {
        vec![RunningPes {
            version: PesVersion::Pes19,
            exe: None,
        }]
    }
    let (a, calls_a) = stub("a", "A", None);
    let mut settings = test_settings(&dir);
    settings.common.pes_version = PesVersion::Pes21;
    let Shell { mut app, .. } = shell_with(settings, vec![a], &[], None, vec![calls_a]);
    app.sidebar.probe = no_exe;
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(0.0));
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(5.0));
    let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(6.0)).nodes;
    assert!(has_text(&nodes, "Switched to PES 2019"));
    assert!(
        nodes
            .iter()
            .all(|node| node.label() != Some("Fix PES path")),
        "no action without an exe path"
    );

    // The exe's folder is already the resolved one: the notice has no action either.
    fn matching() -> Vec<RunningPes> {
        vec![RunningPes {
            version: PesVersion::Pes19,
            exe: Some(
                test_dir("studio_core_autoswitch_notice")
                    .join("installed")
                    .join("Pro Evolution Soccer 2019")
                    .join("PES2019.exe"),
            ),
        }]
    }
    let installed = dir.join("installed").join("Pro Evolution Soccer 2019");
    fs::create_dir_all(&installed).unwrap();
    fs::write(installed.join("PES2019.exe"), b"").unwrap();
    let (a, calls_a) = stub("a", "A", None);
    let mut settings = test_settings(&dir);
    settings.common.pes_version = PesVersion::Pes21;
    settings.common.pes_folder_path = dir
        .join("installed")
        .join("Pro Evolution Soccer 20**")
        .display()
        .to_string();
    let Shell { mut app, .. } = shell_with(settings, vec![a], &[], None, vec![calls_a]);
    app.sidebar.probe = matching;
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(0.0));
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(5.0));
    let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(6.0)).nodes;
    assert!(has_text(&nodes, "Switched to PES 2019"));
    assert!(
        nodes
            .iter()
            .all(|node| node.label() != Some("Fix PES path")),
        "no action when the folder already matches"
    );

    // An exe whose folder does not exist on this system (a Windows path the probe read
    // under Wine or Proton): the notice carries no action either.
    fn missing_folder() -> Vec<RunningPes> {
        vec![RunningPes {
            version: PesVersion::Pes19,
            exe: Some(
                test_dir("studio_core_autoswitch_notice")
                    .join("running-missing")
                    .join("Pro Evolution Soccer 2019")
                    .join("PES2019.exe"),
            ),
        }]
    }
    let (a, calls_a) = stub("a", "A", None);
    let mut settings = test_settings(&dir);
    settings.common.pes_version = PesVersion::Pes21;
    let Shell { mut app, .. } = shell_with(settings, vec![a], &[], None, vec![calls_a]);
    app.sidebar.probe = missing_folder;
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(0.0));
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(5.0));
    let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(6.0)).nodes;
    assert!(has_text(&nodes, "Switched to PES 2019"));
    assert!(
        nodes
            .iter()
            .all(|node| node.label() != Some("Fix PES path")),
        "no action for a folder that does not exist"
    );
}

#[test]
fn the_fix_pes_path_click_rewrites_the_year_the_notice_was_raised_for() {
    let dir = test_dir("studio_core_autoswitch_fixpick");
    fs::create_dir_all(dir.join("data")).unwrap();
    let (running_folder, running_exe) = running_pes_2019(&dir);
    fs::create_dir_all(&running_folder).unwrap();
    fs::write(&running_exe, b"").unwrap();

    fn probe() -> Vec<RunningPes> {
        vec![RunningPes {
            version: PesVersion::Pes19,
            exe: Some(running_pes_2019(&test_dir("studio_core_autoswitch_fixpick")).1),
        }]
    }

    let (a, calls_a) = stub("a", "A", None);
    let mut settings = test_settings(&dir);
    settings.common.pes_version = PesVersion::Pes21;
    settings.common.pes_folder_path = dir
        .join("installed")
        .join("Pro Evolution Soccer 20**")
        .display()
        .to_string();
    let Shell { mut app, ctx, .. } = shell_with(
        settings,
        vec![a],
        &[],
        Some(dir.join("data")),
        vec![calls_a],
    );
    app.sidebar.probe = probe;
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();

    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(0.0));
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(5.0));
    let nodes = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(6.0)).nodes;
    assert!(has_text(&nodes, "Switched to PES 2019"));
    entry(&nodes, "Fix PES path");

    // A manual pick back to PES 2021 leaves the notice up: the click must still rewrite the
    // running game's year, not the year the selection holds now.
    click(&egui_ctx, &mut app, combo(&nodes));
    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    click(&egui_ctx, &mut app, entry(&nodes, "PES 2021"));
    assert_eq!(ctx.common().pes_version, PesVersion::Pes21);

    let nodes = frame(&egui_ctx, &mut app, Vec::new(), true).nodes;
    click(&egui_ctx, &mut app, entry(&nodes, "Fix PES path"));
    assert!(
        std::path::Path::new(&ctx.common().pes_folder_path).ends_with("Pro Evolution Soccer 20**"),
        "the running game's year is rewritten, not the selection's: {}",
        ctx.common().pes_folder_path
    );
    fs::remove_dir_all(&dir).unwrap();
}

// "If several PES versions are running, the first detected wins" (gui.md "Sidebar" item 2):
// the probe's first game is the one that matters, not the first one that differs.
#[test]
fn the_poll_takes_the_first_detected_game_and_stays_put_when_it_is_the_selection() {
    fn probe() -> Vec<RunningPes> {
        vec![
            RunningPes {
                version: PesVersion::Pes19,
                exe: None,
            },
            RunningPes {
                version: PesVersion::Pes21,
                exe: None,
            },
        ]
    }
    let (a, calls_a) = stub("a", "A", None);
    let mut settings = test_settings(&test_dir("studio_core_autoswitch_first"));
    settings.common.pes_version = PesVersion::Pes19;
    let Shell { mut app, ctx, .. } = shell_with(settings, vec![a], &[], None, vec![calls_a]);
    app.sidebar.probe = probe;
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();

    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(0.0));
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(5.0));
    assert_eq!(
        ctx.common().pes_version,
        PesVersion::Pes19,
        "the first detected game is the selection: nothing switches"
    );
    assert!(
        !app.sidebar.auto_switch_used,
        "no switch happened: the switch stays unspent"
    );
}

#[test]
fn the_poll_calls_the_probe_every_five_seconds_and_schedules_its_repaint() {
    static PROBE_CALLS: AtomicUsize = AtomicUsize::new(0);
    // The probe reports the selected version itself (the default), so nothing switches and
    // the polling goes on.
    fn probe() -> Vec<RunningPes> {
        PROBE_CALLS.fetch_add(1, Ordering::SeqCst);
        vec![RunningPes {
            version: PesVersion::Pes19,
            exe: None,
        }]
    }

    let (a, calls_a) = stub("a", "A", None);
    let Shell { mut app, .. } = shell_with(
        test_settings(&test_dir("studio_core_autoswitch_poll")),
        vec![a],
        &[],
        None,
        vec![calls_a],
    );
    app.sidebar.probe = probe;
    let egui_ctx = egui::Context::default();
    egui_ctx.enable_accesskit();

    // The t = 0 frame arms the first poll for t = 5 s.
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(0.0));
    // Between polls the frame asks for a repaint at the next one: three seconds left (egui
    // trims the request by the predicted frame time, so the bound is a hair under). The
    // t = 1 s frame is not read: the first frame's own request (the window's theme sync) is
    // still settling there.
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(1.0));
    let out = frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(2.0));
    let delay = out.repaint_delay.as_secs_f64();
    assert!(
        (2.9..=3.0).contains(&delay),
        "the poll asks for a repaint at the next poll, got {delay} s"
    );

    for t in [5.0, 6.0, 9.9] {
        frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(t));
    }
    assert_eq!(
        PROBE_CALLS.load(Ordering::SeqCst),
        1,
        "the probe ran once, at t = 5"
    );
    frame_at(&egui_ctx, &mut app, Vec::new(), true, Some(10.0));
    assert_eq!(
        PROBE_CALLS.load(Ordering::SeqCst),
        2,
        "the next poll ran at t = 10"
    );
}
