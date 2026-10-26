//! The window's diagnostic sink: `studio.log` in the data location, the previous run's
//! renamed to `studio.old.log` at start, every line flushed as it is written and the
//! user's home folder written as `~`, because the file is what a member posts with a bug
//! report (core plan, "Diagnostic logging"; ST-RUN-07). A panic is written to it with a
//! backtrace by the panic hook, and a panic that ends the window raises a dialog naming
//! the file before the process exits with the code an unguarded panic would (ST-RUN-08).
//! The CLI and help modes keep `env_logger`: a log file a member is asked to post is a
//! GUI-mode need.

use std::any::Any;
use std::backtrace::Backtrace;
use std::fs::{self, File};
use std::io::{self, Write};
use std::panic::{self, AssertUnwindSafe, PanicHookInfo};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Mutex, MutexGuard, PoisonError};

use directories::BaseDirs;
use log::{Level, LevelFilter, Metadata, Record};
use studio_core::version_label;

/// `studio.log`'s name in the data location.
const LOG_FILE_NAME: &str = "studio.log";
/// What the previous run's log is renamed to, so a crash followed by a relaunch keeps
/// the crash's log.
const OLD_LOG_FILE_NAME: &str = "studio.old.log";

/// `line` with the user's home folder written as `~`, because findings carry absolute
/// paths and the log is posted where the cup's members can read it (ST-RUN-07). No home
/// known (`None`) and an empty one both leave the line as it is.
fn redact_home(line: &str, home: Option<&str>) -> String {
    match home {
        Some(home) if !home.is_empty() => line.replace(home, "~"),
        _ => line.to_string(),
    }
}

/// Creates `data_dir/studio.log` with `first_line` as its first line, the previous run's
/// renamed to `studio.old.log` first: `fs::rename` replaces the older file on Windows
/// and Linux alike, so a crash followed by a relaunch keeps the crash's log.
fn open_log(data_dir: &Path, first_line: &str) -> io::Result<File> {
    let path = data_dir.join(LOG_FILE_NAME);
    if path.exists() {
        fs::rename(&path, data_dir.join(OLD_LOG_FILE_NAME))?;
    }
    let mut file = File::create(&path)?;
    writeln!(file, "{first_line}")?;
    Ok(file)
}

/// The `log::Log` writing `studio.log`, at most `Info` (the plan's level: a double-click
/// passes no `-v` or `RUST_LOG`). A struct, not a closure, so the tests can drive the
/// `log` method — a process holds one logger and `main`'s tests already install the
/// CLI's.
struct FileLog {
    file: Mutex<File>,
    home: Option<String>,
}

impl FileLog {
    /// The lock survives a panic mid-write: a panic's own line must not be lost to the
    /// poisoning it caused.
    fn locked(&self) -> MutexGuard<'_, File> {
        self.file.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl log::Log for FileLog {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = redact_home(
            &format!(
                "{:<5} {}: {}\n",
                record.level(),
                record.target(),
                record.args()
            ),
            self.home.as_deref(),
        );
        let mut file = self.locked();
        // Each line is flushed at once, so a crash keeps everything before it. The log
        // is where failures would be reported, so its own failed write has nowhere to
        // report itself: it is dropped.
        #[expect(
            clippy::let_underscore_must_use,
            reason = "a failed write to the log has nowhere to be reported"
        )]
        let _ = file.write_all(line.as_bytes()).and_then(|()| file.flush());
    }

    fn flush(&self) {
        #[expect(
            clippy::let_underscore_must_use,
            reason = "a failed flush of the log has nowhere to be reported"
        )]
        let _ = self.locked().flush();
    }
}

/// Installs `studio.log` as the `log` sink at level `Info` and returns its path. `None`
/// means no log this run: no data location yet (a first run, whose dialog has not chosen
/// one — a log written elsewhere would leave a folder behind that the portable choice
/// promised not to), or the file cannot be opened (a read-only location the status bar
/// already reports) (ST-RUN-07).
pub(crate) fn install_gui_logger(data_dir: Option<&Path>) -> Option<PathBuf> {
    let data_dir = data_dir?;
    let first_line = format!("{} {}", studio_core::app_name!(), version_label());
    let file = open_log(data_dir, &first_line).ok()?;
    let home = BaseDirs::new().map(|dirs| dirs.home_dir().display().to_string());
    log::set_boxed_logger(Box::new(FileLog {
        file: Mutex::new(file),
        home,
    }))
    .ok()?;
    log::set_max_level(LevelFilter::Info);
    Some(data_dir.join(LOG_FILE_NAME))
}

/// The hook that writes every panic to the log — the `PanicHookInfo`'s `Display` (its
/// message and location), then a backtrace — chained before the hook that was installed
/// when it was made, so a terminal still prints the panic as before.
fn panic_hook(logger: &'static dyn log::Log) -> Box<dyn Fn(&PanicHookInfo<'_>) + Send + Sync> {
    let previous = panic::take_hook();
    Box::new(move |info| {
        let backtrace = Backtrace::force_capture();
        logger.log(
            &Record::builder()
                .level(Level::Error)
                .target("panic")
                .args(format_args!("{info}\n{backtrace}"))
                .build(),
        );
        logger.flush();
        previous(info);
    })
}

/// Installs the panic hook over the `log` sink installed by `install_gui_logger`; with
/// none (a first run) `log::logger()` is a nop and only the chained hook still prints.
pub(crate) fn install_panic_hook() {
    panic::set_hook(panic_hook(log::logger()));
}

/// The crash dialog's text, written for a member: what happened, and the file — or the
/// message, when no log exists — to post where problems are reported. No code
/// vocabulary reaches it (CONTRIBUTING.md "Code style").
fn crash_text(message: &str, log: Option<&Path>) -> String {
    let what = format!(
        "{} stopped because of an error in its own code: {message}",
        studio_core::app_name!()
    );
    match log {
        Some(path) => format!(
            "{what}\n\nPlease post this file where you report problems: {}. It holds what the program was doing; your home folder is written in it as ~.",
            path.display()
        ),
        None => format!("{what}\n\nPlease post this message where you report problems."),
    }
}

/// What a panic's payload says: the `&str` or `String` `panic!` carried, or a fallback
/// for a payload of anything else.
fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "an error without a message".to_string()
}

/// Runs the window inside a panic guard: a clean return is the window's own code, while
/// a panic is told to the member through `show` and exits 101 — the code an unguarded
/// panic exits with, so launchers see the same code as before (ST-RUN-08). A panic on
/// another thread is logged by the hook but raises no dialog here: the hook cannot tell
/// a panic that will be recovered from one that ends the process.
pub(crate) fn guard_gui(
    run: impl FnOnce() -> ExitCode,
    log: Option<&Path>,
    show: impl FnOnce(&str),
) -> ExitCode {
    match panic::catch_unwind(AssertUnwindSafe(run)) {
        Ok(code) => code,
        Err(payload) => {
            show(&crash_text(&panic_message(payload.as_ref()), log));
            ExitCode::from(101)
        }
    }
}

/// The blocking dialog raised for a failure a double-clicked window cannot print: a
/// startup error, a window that could not open, or the panic that ended it. Titled with
/// the suite's name, level Error, one OK button.
pub(crate) fn show_error_dialog(text: &str) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title(studio_core::app_name!())
        .set_description(text)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicBool, Ordering};

    use log::Log;

    use super::*;
    use crate::test_dir::TestDir;

    // ST-RUN-07
    #[test]
    fn the_previous_runs_log_is_kept_as_the_old_log() {
        let dir = TestDir::new("rotate");
        fs::write(dir.path().join(LOG_FILE_NAME), "second run\n").unwrap();
        fs::write(dir.path().join(OLD_LOG_FILE_NAME), "first run\n").unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join(OLD_LOG_FILE_NAME)).unwrap(),
            "first run\n"
        );
        open_log(dir.path(), "Unlimited Tool Works v0.0.1 pre-alpha").unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join(OLD_LOG_FILE_NAME)).unwrap(),
            "second run\n"
        );
        assert_eq!(
            fs::read_to_string(dir.path().join(LOG_FILE_NAME)).unwrap(),
            "Unlimited Tool Works v0.0.1 pre-alpha\n"
        );
    }

    // ST-RUN-07
    #[test]
    fn a_first_run_leaves_no_old_log() {
        let dir = TestDir::new("fresh");
        open_log(dir.path(), "Unlimited Tool Works v0.0.1 pre-alpha").unwrap();
        assert!(!dir.path().join(OLD_LOG_FILE_NAME).exists());
    }

    // ST-RUN-07
    #[test]
    fn a_logged_line_writes_the_home_folder_as_a_tilde() {
        let dir = TestDir::new("redact");
        let path = dir.path().join(LOG_FILE_NAME);
        let logger = FileLog {
            file: Mutex::new(File::create(&path).unwrap()),
            home: Some("C:\\Users\\member".to_string()),
        };
        logger.log(
            &Record::builder()
                .level(Level::Info)
                .target("studio::gui_log")
                .args(format_args!(
                    "opened C:\\Users\\member\\Documents\\KONAMI\\a.bin and C:\\Users\\member\\b"
                ))
                .build(),
        );
        let written = fs::read_to_string(&path).unwrap();
        assert_eq!(
            written,
            "INFO  studio::gui_log: opened ~\\Documents\\KONAMI\\a.bin and ~\\b\n"
        );
        logger.log(
            &Record::builder()
                .level(Level::Debug)
                .target("studio::gui_log")
                .args(format_args!("hidden"))
                .build(),
        );
        assert!(!logger.enabled(&Metadata::builder().level(Level::Debug).build()));
        assert_eq!(fs::read_to_string(&path).unwrap(), written);
    }

    #[test]
    fn redact_home_without_a_home_leaves_the_line_as_it_is() {
        let line = "opened C:\\Users\\member\\a.bin";
        assert_eq!(redact_home(line, None), line);
        assert_eq!(redact_home(line, Some("")), line);
    }

    /// The logger `panic_hook` is tested over: a `&'static` test double collecting each
    /// record's level, target and text plus whether `flush` ran.
    #[derive(Default)]
    struct CollectLog {
        records: Mutex<Vec<String>>,
        flushed: AtomicBool,
    }

    impl log::Log for CollectLog {
        fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
            true
        }

        fn log(&self, record: &Record<'_>) {
            self.records.lock().unwrap().push(format!(
                "{:?}|{}|{}",
                record.level(),
                record.target(),
                record.args()
            ));
        }

        fn flush(&self) {
            self.flushed.store(true, Ordering::Relaxed);
        }
    }

    #[test]
    fn the_panic_hook_writes_the_panic_and_its_backtrace_to_the_log() {
        let logger: &'static CollectLog = Box::leak(Box::default());
        // The harness's hook is parked aside and restored before any assertion, so a
        // failing assertion panics through it; meanwhile the hook under test chains the
        // default hook `take_hook` registered in its place.
        let saved = panic::take_hook();
        panic::set_hook(panic_hook(logger));
        let result = panic::catch_unwind(|| panic!("probe 7"));
        panic::set_hook(saved);
        assert!(result.is_err());
        let records = logger.records.lock().unwrap();
        let record = records
            .iter()
            .find(|record| record.contains("probe 7"))
            .unwrap_or_else(|| panic!("no panic record; got {records:?}"));
        assert!(record.starts_with("Error|panic|"), "{record}");
        assert!(record.contains("gui_log.rs"), "{record}");
        assert!(record.contains("gui_log::tests"), "{record}");
        assert!(logger.flushed.load(Ordering::Relaxed));
    }

    #[test]
    fn a_clean_run_returns_its_own_code_and_shows_no_dialog() {
        let shown = RefCell::new(Vec::<String>::new());
        let code = guard_gui(
            || ExitCode::from(2),
            None,
            |text| shown.borrow_mut().push(text.to_string()),
        );
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::from(2)));
        assert!(shown.borrow().is_empty());
    }

    #[test]
    fn a_panic_shows_the_dialog_text_and_exits_with_101() {
        let shown = RefCell::new(Vec::<String>::new());
        let code = guard_gui(
            || panic!("probe 8"),
            Some(Path::new("C:\\data\\studio.log")),
            |text| shown.borrow_mut().push(text.to_string()),
        );
        assert_eq!(format!("{code:?}"), format!("{:?}", ExitCode::from(101)));
        let texts = shown.borrow();
        assert_eq!(texts.len(), 1);
        assert!(texts[0].contains("probe 8"), "{}", texts[0]);
        assert!(texts[0].contains("studio.log"), "{}", texts[0]);
    }

    #[test]
    fn crash_text_without_a_log_path_asks_for_the_message() {
        let text = crash_text("the window froze", None);
        assert!(text.contains("the window froze"), "{text}");
        assert!(
            text.contains("Please post this message where you report problems."),
            "{text}"
        );
    }
}
