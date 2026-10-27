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
first run" and "Live AATF stays off by default"). P39 (a window closed maximized reopens
maximized) was accepted on 2026-10-27 (`DECISIONS.md` "A window closed maximized reopens
maximized").

## P40 — The Music export editor reorders its players

From the RigDJ audit (10.0h, 2026-10-27). The editor lists and writes the players in the
file's order, which is the order of Rigdio's buttons (`plans/music_export_editor.md` "What
the editor does", the slot list). 27 of the 50 RigDJ-written exports on the maintainer's
machine have their players out of sorted order, so managers reorder them by hand in a text
editor; RigDJ re-sorted them on every load.

Adds: up/down buttons (or a drag) on a player in the slot list, so a manager sets the button
order the streamer sees without leaving the editor. Costs: two buttons beside Rename and
Delete, and the drag the song rows already plan reused for the slot list; nothing new in the
format.
