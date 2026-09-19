# 4cc Studio — Core plan: Distribution and updates

Part of the [Core plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Distribution and updates

### License

**`MIT OR Apache-2.0`** — the Rust ecosystem's dual license, declared once as
`license = "MIT OR Apache-2.0"` in the workspace `Cargo.toml` and inherited by every crate;
`LICENSE-MIT` and `LICENSE-APACHE` at the repository root; contributions are accepted under the
same terms (the README says so). Red stays GPL-3.0; Studio is an independent reimplementation of
its ideas, not a derivative of its code (see "Relationship to legacy tools"), and Red's author is
this project's author, so nothing in Red's license binds Studio.

Permissive rather than copyleft because the suite is *designed as libraries with tools on top*,
for a community that writes its own tools in whatever language the author knows: a GPL
`pes_savefile` or `cpk` could go into a GPL Blender add-on and nowhere else, which would cancel
half the reason the crate boundaries exist. The copyleft threat — a closed fork — has never
materialized in this community's tooling and would do it no harm if it did; the open version
stays. Apache alongside MIT for the explicit patent grant and because it is what every dependency
uses. The door MIT closes (absorbing GPL code) leads nowhere: Red is the author's own, 4ccEditor
is zlib, and the other legacy tools carry no license at all — unusable verbatim under any license.

What the license does **not** cover: game-derived data kept for interoperability — the skeletons
in `resources/skeletons/`, `DpFileList.bin` and the other embedded `.bin` templates, the kit-icon
reference sheet. They are Konami's; the README states it. `just deps-check` enforces the
dependency side with a `cargo deny` license allowlist (`MIT`, `Apache-2.0`, `BSD-2-Clause`,
`BSD-3-Clause`, `ISC`, `Zlib`, `Unicode-*`, `MPL-2.0` for file-level copyleft crates only, plus
the font licenses `OFL-1.1` and `Ubuntu-font-1.0` that egui's bundled default fonts carry), so a
copyleft crate cannot enter the dependency graph unnoticed. The list lives in `deny.toml`.

### Distribution: portable .7z bundle

There is no installer. Each release is a `.7z` archive containing:

- `studio.exe` (or the Linux binary) — the single self-contained binary
- `quick_compile.bat` (`quick_compile.sh` on Linux) — the double-click launcher: opens the GUI on
  the Team compiler and starts a compile (see `architecture.md` "Launch modes" under Architecture)
- a basic readme (pointing into the app: the full help and the changelog live in the help window
  — see `gui.md` "Help window" under GUI Design — and `studio help --export` produces the same text as
  files for the release page), with the two license files beside it

The current teams list is not a bundle file: it is embedded in the binary and materialized as
`data/teams_list.txt` on first run (see the [Team compiler plan](../team_compiler/README.md), "Resolved
decisions", "Teams list").

The user extracts the archive anywhere and runs the binary — or, for the common case, drops an
export into `exports/` (shipped empty beside the binary) and double-clicks
`quick_compile.bat`, which is the whole Red workflow carried over. Where the suite's persisted data goes
(the settings file, state like the skipped-update version, logs) is the user's choice, asked once on
first run — see "Data location" below. This is the same folder model Red uses, minus the ~80MB
embedded Python runtime that made Red's updates a folder-replacement operation.

### Data location: asked on first run

Some users want a fully portable install, others want their data to survive replacing or moving the
program folder. Following EGG-Translator's first-start wizard, the first run asks where to store
settings and data, with two options (each with a one-line hint, like EGG's buttons):

- **Next to the executable** — a `data/` subfolder beside the binary. Fully portable: the folder
  carries the program and everything it persists; deleting it removes the program completely, moving
  it moves everything.
- **In the user config directory** — `%APPDATA%\4cc-studio` on Windows, `~/.config/4cc-studio` on
  Linux (via the `directories` crate). Survives deleting or replacing the program folder.

Resolution at startup is **presence-based, no marker files** (EGG's mechanism): if
`data/settings.toml` exists next to the binary → portable mode; otherwise if the config-dir settings
file exists → config-dir mode; if neither → first run, show the dialog, and write the settings file
to the chosen location — the file's existence is what makes the choice stick. Since the portable
location wins when both exist, choosing the config dir while a portable settings file is present
renames that file to `.bak` so it can't shadow the choice on the next start (EGG's exact fix for
this edge).

`data/teams_list.txt` *is* data-dir cargo (the grid writes into it; see the [Team compiler
plan](../team_compiler/pipeline.md), "Resolved decisions", "Teams list"), and writes to it are best-effort:
an unwritable data directory makes the list read-only, never triggers elevation. The following
things are *not* data-dir cargo:

- The `exports/` folder (the default `exports_folder_path`) lives **next to the binary in both
  modes**, beside `quick_compile.bat`. Copying exports into `%APPDATA%` is impractical and would
  break the "drop it in the folder, double-click" habit Red established; users who want it elsewhere
  point the setting at another path.
- The `output/` folder (the default `output_folder_path`) likewise lives next to the binary — it is
  Red's `patches_output/`, where users look for a CPK when a deployment could not happen or
  `--no-deploy` was given; it also hosts the run-scoped staging folder (see "Post-processing" in
  the Team compiler plan).
- The `old/` rollback binary is tied to the program folder, not the data.

The question is desktop-only: the browser build has no exe directory and persists to browser
storage.

### Self-update: in-place binary swap

Red updates by downloading the new release's 7z, extracting it as a **sibling folder**, and
migrating user data into it (settings via a transfer table, `teams_list.txt` via an interactive
diff, `exports/` contents, state files) — necessary because Red's install folder is a pile of
engine files that must be replaced wholesale, with user data interleaved. 4cc Studio's folder is
a few files, so the update inverts: **the folder stays, the binary is swapped in place**, and there
is almost nothing to migrate.

The desktop flow is coordinated by `studio` (the binary). `studio_core::updater` provides generic
release checking, download/verification, and binary replacement; `studio` separately calls
`teams_list`'s parse/reconcile logic for teams-list reconciliation. Core knows neither
teams-list/team-ID semantics nor team-format types. No plugin hook or new updater framework is
needed. This is desktop-only (`#[cfg(not(target_arch = "wasm32"))]`); the browser build updates
server-side:

1. **Check** (startup, non-blocking background task): if `check_for_updates` is on and the last
   check is older than the check interval, query the GitHub Releases API (`releases/latest`),
   compare the `tag_name` semver against the built-in `CARGO_PKG_VERSION`. The release `body`
   (release notes) comes back in the same response.
2. **Offer**: if a newer version exists and it isn't the `skipped_version`, show a dialog with the
   release notes inline and the options *Update now* / *Skip this version* / *Remind me later*; a
   *Disable update checks* option lives in settings (kept out of the dialog — Red's `fuckoff`). The
   CLI equivalent is a one-line notice plus a `studio update` subcommand that performs the
   check+apply non-interactively.
3. **Download + verify**: fetch the release's `.7z` asset (the same artifact users download manually
   — one artifact serves both paths) into a temp folder, verify its SHA256 against a checksum asset,
   and extract it with the already-in-workspace `sevenz-rust`. The download **streams** the
   response body (`ureq`'s `into_reader()`) into the temp file, hashing and counting bytes as it
   goes — that is where the dialog's progress comes from, and it sidesteps `ureq`'s 10 MB default
   in-memory body limit, which the convenience readers (`read_to_vec`/`read_to_string`/`read_json`)
   enforce and a release asset exceeds (`into_reader()` is unbounded; `into_with_config().limit()`
   changes either). Both HTTP calls run on a plain background thread.
4. **Swap**: first stage and verify the new binary on the executable's volume. Move the running
   binary into `old/`, then put the new binary in place; if replacement fails, restore the original.
   This is a recoverable sequence, not one atomic swap; handle file locks and permissions explicitly.
   If the folder isn't writable (e.g. Program Files), use the `elevation` lib's elevated-relaunch
   path or download-and-instruct. Restart only after active work is stopped and unsaved edits are
   handled by the ordinary close prompts.
5. **Teams list reconciliation** (on the first start of the new binary, not during the swap): the
   new binary carries the new upstream list embedded; the working `data/teams_list.txt` may contain
   user-added ID assignments (the grid's inline ID-cell writes — see the [Team compiler
   plan](../team_compiler/README.md)), so it can't just be overwritten. The `studio` coordinator calls
   `teams_list`'s reconciliation, not the generic core updater, to **merge** instead
   of showing Red's pick-one diff: rows only in the embedded list are added, rows only in the
   working list (user additions) are kept, and conflicting rows (same team, different ID) take the
   embedded list's value. The result is presented as a reviewable summary (added / kept /
   overridden) before being written. Validate uniqueness of both team names and IDs before
   publication: a new team's ID may collide with a retained local team's ID even when their keys
   differ. Unresolved conflicts leave the list unchanged and are reported for review (the
   noninteractive CLI does not guess). An unwritable data directory skips the merge with
   `teams_list_read_only`; the embedded list is not used in place of the working one, so the
   user's assignments keep winning until they move the install. The readme and
   `quick_compile.bat` are overwritten without ceremony (not user-edited).
6. **Restart**: offer to relaunch. The settings file needs no transfer — it stays where it is, and
   the existing merge of `default_settings()` for missing keys (see `gui.md` "Settings menu") absorbs
   new-version settings changes, which is what retired Red's transfer-table mechanism.

### Rollback

`old/studio.exe` is kept (one version deep — each update replaces it), and the settings menu gets a
**"Roll back to previous version"** button that swaps the two binaries back and relaunches. This
covers the "new version is broken" case that Red handles by preserving the entire old folder.

### Update state

Stored with the settings (common settings, `studio_core`):

| Key | Replaces (Red) | Default |
|-----|----------------|---------|
| `check_for_updates` | `updates_check` ini key | `true` |
| `check_interval_hours` | hardcoded 60 minutes | `24` (a resident GUI app, not a per-compile CLI run) |
| `last_update_check` | `state/update_check_last.txt` | — |
| `skipped_version` | `state/update_skip_last.txt` | — |

This closes out Red's entire `Engines/state/` folder — the remaining five files either became
settings or lost their reason to exist:

| Red state file | Fate |
|---|---|
| `dt00_write_allowed.txt` | The `dt00_overwrite_allow` setting (team compiler) — standing consent as a setting, not a marker |
| `first_run_done.txt` | Presence-based data-location resolution: the settings file's existence *is* the first-run marker (see "Data location") |
| `admin_warned.txt` | Dropped — the one-time console explanation before Red's UAC relaunch becomes the `elevation` lib's GUI prompt, which explains itself every time it appears |
| `sideload_warned.txt` | Dropped — the one-time sideload explanation becomes the per-run `sideload_active` Info message (team compiler catalog) |
| `ver_mismatch_warned.txt` | Dropped — the suppressible console notice becomes the plain per-run `pes_version_mismatch` warning (team compiler catalog) plus the version selector's live red/yellow exe check (see `gui.md` "Sidebar") |

### Versioning: one workspace version, one exception

The suite uses a **single workspace version** for every crate that ships in the `studio` binary. Set
once in the root `Cargo.toml`:

```toml
[workspace.package]
version = "1.4.0"
```

Every tool and lib crate inherits it via `version.workspace = true`. Bumping is one line, applied to
all crates at once.

Rationale:

- **The binary is the release unit.** Users download `studio.exe`; the self-updater compares
  `CARGO_PKG_VERSION` against a GitHub Release tag. The version they see *is* the suite version.
  Independent crate versions would be invisible to every actual consumer.
- **Nothing is published to crates.io.** Per-crate semver exists to signal breaking changes to
  downstream *crate* consumers; there are none; every internal crate's only consumer is `studio` (or
  another internal crate), all compiled together in one binary.
- **Tools are compile-time plugins**, statically registered in `studio`. A "team_compiler 2.0 vs.
  studio 1.5" compatibility question is impossible by construction — a version mismatch can't ship.
- **Changelog and rollback are per-Studio-release.** Release notes are "4cc Studio 1.4 adds the
  Balls compiler"; the rollback button swaps two binaries in `old/`. Both only make sense on one
  version axis.

A release is cut (tag, GitHub Release, `.7z` asset) when there's something to distribute; not every
commit is a release. The workspace version is the next-release version; the tag is what the updater
compares against.

**One exception: `python_bindings`.** It is built via maturin into a wheel that Blender users
install independently of the Studio binary, and its consumers are not Studio users. Its version is
meaningful to them (which wheel matches their Blender / which `fmdl`+`pes_model` API), so it keeps
its own `version = "..."` and its own release cadence — shipping on demand when the
`fmdl`/`pes_model` API surface it exposes changes. This is also why the plan builds it outside the
default `cargo build` path.

### Changelog and version display

**One changelog, three readers.** `CHANGELOG.md` at the repository root, in the Keep a Changelog
shape: `## [Unreleased]` on top, then one `## [1.4.0] — 2026-10-02` section per release; inside a
section, bullets grouped by tool (`**Team compiler** — …`, `**Save editor** — …`, `**Studio** —
…`), because that is how a member reads it even though the version axis is one. It is written in
the step that changes the behavior, like the help topic (see `gui.md` "Help window"). Nothing else
describes releases; the three places a user meets release notes all read this file:

1. **The help window's "What's new" chapter** — the file embedded at build time, newest first.
2. **The GitHub release body** — the release process copies the released version's section
   verbatim into the release notes. The updater dialog shows that body inline (see "Distribution
   and updates" step 2), so the dialog, the release page and the in-app chapter are the same text
   by construction rather than by discipline.
3. **The post-update notice** — `Updated to v1.4.0 · [What's new]` in the status bar on the first
   start of a new version, opening reader 1 (see `gui.md` "Status bar").

**One version string, shown in one place.** `v{CARGO_PKG_VERSION}`, with ` (debug)` appended
when `cfg!(debug_assertions)` — enough to tell a local build from a release, which is the only
distinction that has ever mattered in a bug report. No git hash: it would need a `build.rs` or
`vergen` for information the tag already carries, since releases are built from tags. Where it
appears:

- **Status bar empty state** — the one always-visible place, and the line to paste into a report.
- **About** (logo click) — the detailed block with Copy (see `gui.md` "Sidebar" item 1).
- **`studio --version`** — the same one-liner on stdout (CLI equivalence).
- **Not the title bar**: the title is what the taskbar and alt-tab show — the surface for what
  the Studio is *doing* while the window is not in front (see `gui.md` "Window title"), and it is outside
  every screenshot a user crops to the window content. **Not beside the logo**: duplicate of the
  status bar, and homeless when the sidebar collapses to the icon rail.

---
