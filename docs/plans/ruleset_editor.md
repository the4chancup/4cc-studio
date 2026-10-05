# 4cc Studio — Ruleset editor plan

The Ruleset editor (`crates/tools/ruleset_editor`, tool id `ruleset-editor`, sidebar label
"Ruleset editor") lets anyone who can fill in a form build an AATF ruleset for their own
invitational: tiers, stat targets, heights, cards and conditional special rules, saved as a rules
file the Save editor and the Team creator use like the official one. The ruleset need not start
from the official rules or resemble them. The file format, the schema and the interpreter are
`libs/aatf`'s, specified in the [AATF rules plan](aatf_rules.md); this tool is a form over that
schema. Platform context is in the [core plan](core/README.md).

---

## Design principles

1. **Forms, not code.** Every field of the ruleset schema has a widget; nothing in it is reachable
   only by editing text. The user never sees Rhai unless the file has custom checks, which are
   shown read-only.
2. **The tool owns the data, Studio owns the logic, the author owns the custom checks.** Saving
   rewrites the `RULESET` block whole, writes the current logic, and copies the custom-checks
   section byte for byte. Hand edits inside `RULESET` survive as values; their formatting and
   comments do not.
3. **An invalid ruleset is never written.** The load-time rules of "Loading and validation" run
   on every edit; Save stays available and, while the ruleset is invalid, lists what blocks it
   instead of writing.

---

## The editor

**Start page:** *New from template*, where the template is **Autumn 26** (the official ruleset),
**VGL26**, or **Blank** (one tier, no brackets, the universal rules on), and *Open file*. Recent
files below.

A left rail lists the sections; each is a form, and a section with a validation error shows a
mark in the rail.

- **General**: name, squad size, the rating stats and the at-most stats (stat checklists, with
  one line on what each list does).
- **Tiers**: a table, one row per tier, highest first, reordered by dragging, since the order is
  the recognition order and the quick-action buttons' order. Columns: label, count (a number or
  "the rest"), counts as, rate, form, injury resistance, weak foot, paid cards, free cards per
  group, A positions. Per row, an expander holds the per-stat targets (all ability stats, empty
  meaning "the rate") and the tier's conditions (the condition editor below).
- **Heights**: a "use height brackets" switch. On: the brackets in ascending order, each "up to
  N cm" or "exactly N, M cm"; the height systems in order, each "when any player is …" (the
  condition editor, limited to height and position) with a quota per bracket, the last one
  "otherwise". Off: heights are left to tier conditions and specials, as in VGL26.
- **Cards**: the card groups, each a name and a checkbox grid of the 41 skills (labels as the
  Save editor shows them); COM styles are the built-in `com` group.
- **Universal rules**: checkboxes and number fields for `universal`.
- **Specials**: a list of cards, reordered by dragging (order matters for replacing effects), each
  with a label, a **When** row and a **Then** row:
  - *When* is the condition editor: tiers, registered positions, height systems and brackets as
    multi-select chips; a height range; captain yes / no / either; "fielded only at"; and an
    "except when" sub-condition (`not`).
  - *Then* is a list of effects, added from a menu that names them in words ("Add to every stat",
    "Set one stat's target", "Extra paid cards", "Free cards", "Required card", "Extra A
    positions", "At most N A positions", "Weak-foot limits", "Height allowance", "Forbid"). A
    numeric effect has a "same for every tier / per tier" switch; per tier shows one field per tier.
    *One of* adds a choice: ordered options, each a list of effects, with the rule spelled out
    under it ("the first option the player needs is given").
  - A severity selector appears when the effects include Forbid or Required card.
- **Suggestions**: a checkbox per allowance (paid cards, A positions, weak foot, injury
  resistance, height, free cards, and one per card group, COM included).
- **Custom checks**: present only when the file has them; the code, read-only, with a line saying
  it runs after the ruleset and is kept as is.
- **Summary**: the ruleset as readable rules text ("Gold: 1 per team, every stat 99, form 8, …";
  each special as a sentence from its When and Then), generated from the data. **Copy** puts it
  on the clipboard as Markdown, for posting an invitational's rules; two summaries diffed in any
  text tool are the comparison between two rulesets.

Undo and redo work on whole-ruleset snapshots (a ruleset is a few kilobytes). Closing with unsaved
changes prompts.

## Opening and saving

**Open** runs `libs/aatf`'s validation, then reads the logic section's identity:

- *Current*: opens normally.
- *Older official version*: opens, with a banner offering **Update logic**, which migrates
  `RULESET` to the current schema if its version is older and replaces the logic section. The
  file keeps working unchanged until the user accepts.
- *Modified*: opens, with a banner saying the file's logic is not Studio's, so its specials may
  not behave as the forms describe; saving keeps that logic rather than replacing it.
- *Unreadable* (a parse or schema error): not opened in the forms; the findings are listed with
  their line or key path, for fixing in a text editor.

**Save** and **Save as** write the file: the header comment, the `RULESET` block from
`libs/aatf`'s writer, the logic section (current, or the file's own when modified), and the
custom-checks section as read. Before writing, the sample team is checked (validation step 3),
so a file that would fail in the Save editor is never written. The file is written to a
temporary sibling and renamed into place.

The Save editor and the Team creator pick a ruleset through their own settings (the rules path);
this tool does not write other tools' settings.

---

## CLI

```
4cc-studio ruleset-editor validate rules.rhai          # load-time validation; findings to stdout, exit 1 if any
4cc-studio ruleset-editor update-logic rules.rhai      # migrate RULESET and replace an older official logic
4cc-studio ruleset-editor summary rules.rhai [-o rules.md]
```

`update-logic` refuses a file whose logic is modified (it would discard someone's code) and leaves
a current file untouched. Both writing commands keep a `.bak` of the file they replace.

## Settings

- `last_folder`: where Open and Save as start.

---

## Crate layout

Standard tool-crate skeleton (core plan, "Tool crates"), elaborated below it:

```
crates/tools/ruleset_editor/src/
├── lib.rs              # Tool (StudioTool impl) — wiring only
├── settings.rs
├── cli.rs              # validate / update-logic / summary
├── messages.rs         # validation findings, open/save failures, logic-status banners
├── document.rs         # Document: path, Ruleset, custom-checks text, logic status, undo
│                       #   snapshots, dirty flag; egui-free, what the CLI drives
├── summary.rs          # Ruleset → rules text (Markdown)
└── view/
    ├── mod.rs          # start page, section rail, open/save, banners
    ├── condition.rs    # the condition editor (tier conditions, specials, systems)
    ├── effects.rs      # the effects editor: per-tier values, choices
    └── sections/       # one file per section
```

Placement rules: reading, validating, writing and migrating a rules file are `libs/aatf`'s
(`file.rs`, `ruleset.rs`, `writer.rs`); this crate never parses Rhai or formats a `RULESET` block.
`document.rs` and `summary.rs` are egui-free, so the CLI and the GUI share them.

---

## Development phase

Phase 20 in the core plan's Development Plan, after the first release: it needs `libs/aatf` with
the schema and the interpreter (Phase 5) and the shell (Phase 8), and its users are the organizers
of invitationals, who use 4ccEditor forks until then. Deliverables: `libs/aatf`'s writer and the
migration hook, the tool's sections, opening and saving with logic identity, the summary, the CLI,
the help chapter. The tool is file-picking only, so it joins Studio Web's first tier (Phase 18)
once both exist.

Verification: the writer round-trips both embedded rulesets (read, write, read gives an equal
`Ruleset`; writing twice gives identical bytes); VGL26's data rebuilt from Blank through the forms
alone (manual, recorded at converge) validates and passes VGL26's fixtures; a file with an older
logic hash updates and still passes its fixtures, a modified one is refused by `update-logic`; an
invalid ruleset is never written by Save or by the CLI.
