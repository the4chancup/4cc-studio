# Proposals for the maintainer

Ideas nobody asked for that would make the suite easier to use: a button, a shortcut, a
default, a message that saves a member a step. Any agent adds one when it thinks of it while
working (`AGENTS.md` "When the plan has gaps"); the maintainer vetoes or accepts entries as
they come across them. A proposal is not built while it is unruled: plan text that already
describes one says so and points here, and the phase that would build it lists the entry in
its turn report. A vetoed entry leaves with a one-line `DECISIONS.md` entry, so it is not
proposed again; an accepted one leaves into its plan section and a `DECISIONS.md` entry.

Each entry: a heading, where it came from, what it adds and for whom, and what it costs (code,
surface, things a member must learn). Newest last.

The first idea of this kind, the Export upgrader's "Move to exports and compile" button, was
asked in `QUESTIONS.md` before this file existed and approved.

P16 to P38 were ruled on 2026-10-26 (`DECISIONS.md`'s entries "Usability proposals P17-P24
and P32-P38 accepted", "A folder per PES version", "Red's settings are not imported on
first run" and "Live AATF stays off by default").

## P39. A window closed maximized reopens maximized

From `plans/core/gui.md` "Window size classes" (worklog 0.0.1q, 2026-10-26): the window
reopens at the place it was closed at, and a window closed while maximized reopens at the
normal place it had before the maximize, since the record holds that place alone. **Adds**,
for a member who keeps the window maximized: it comes back maximized, as most desktop
programs do, instead of as a normal window they maximize again. **Costs**: a sixth value in
`window_main` (maximized or not), `ViewportBuilder::with_maximized` at creation, and the
check that a maximized window's record is the normal place under it, which eframe's own
persistence handles with a per-platform dance (Windows and macOS move the window first and
maximize after, X11 maximizes only once the window is shown); one more manual check.
