//! The About popup the logo opens (core plan, "Sidebar" item 1; draft
//! `docs/prototypes/shell.html`): the detailed version block a member pastes into a bug
//! report, with a Copy button and links to the release page and the repository. The help
//! link joins when the help window lands (the GUI phase's later step).

use pes_version::PesVersion;

use crate::tool::AppPaths;

/// The repository, from the workspace manifest.
pub(crate) const REPOSITORY_URL: &str = env!("CARGO_PKG_REPOSITORY");
/// The release page: the repository's `releases` listing.
pub(crate) const RELEASE_PAGE_URL: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/releases");

/// The version block the popup shows and Copy puts on the clipboard, unchanged.
pub(crate) fn version_block(paths: &AppPaths, pes_version: PesVersion) -> String {
    let build = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let data = match &paths.data_dir {
        Some(dir) => dir.display().to_string(),
        // A GUI run with no settings file yet has no location until the first-run
        // dialog's choice lands; until then the block says so rather than guessing one.
        None => "not chosen yet".to_owned(),
    };
    [
        format!("4cc Studio v{}", env!("CARGO_PKG_VERSION")),
        format!("Build: {build}"),
        format!("Target: {}", env!("STUDIO_TARGET_TRIPLE")),
        format!("Data location: {data}"),
        format!("PES version: {pes_version}"),
    ]
    .join("\n")
}

/// Draws the popup while `open` holds and clears it when the user closes it (Close, the
/// backdrop or Escape). `Modal` carries no `ClosableTag`, so the Close button returns a flag
/// rather than calling `ui.close`.
pub(crate) fn show(
    ctx: &egui::Context,
    paths: &AppPaths,
    pes_version: PesVersion,
    open: &mut bool,
) {
    let block = version_block(paths, pes_version);
    let modal = egui::Modal::new(egui::Id::new("about-4cc-studio")).show(ctx, |ui| {
        ui.heading("About 4cc Studio");
        ui.label(egui::RichText::new(&block).monospace());
        ui.horizontal(|ui| {
            ui.hyperlink_to("Release page", RELEASE_PAGE_URL);
            ui.hyperlink_to("Repository", REPOSITORY_URL);
        });
        let mut close = false;
        ui.horizontal(|ui| {
            if ui.button("Copy").clicked() {
                ui.ctx().copy_text(block.clone());
            }
            close = ui.button("Close").clicked();
        });
        close
    });
    if modal.inner || modal.should_close() {
        *open = false;
    }
}
