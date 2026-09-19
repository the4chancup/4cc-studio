# 4cc Studio — Library crates plan: teams_list

Part of the [Library crates plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## `libs/teams_list`

The 4cc team identity space and the file that records it, in one **leaf crate with no
dependencies beyond `thiserror`**. Three consumers need it and none is a natural owner: the Team
compiler resolves export names against it, the Save editor writes it from a savefile ("Export
teams list"), and the `studio` binary merges a new release's embedded list into the working copy
after an update. It used to be a module of `aesthetics_export`, which made the Save editor and the
updater depend on the whole export object model for one file format.

- `name.rs` — `TeamName`: the canonical `/xx/` form. `TeamName::new(token)` applies **the** fold
  (Unicode `str::to_lowercase`, wrap in slashes; empty token rejected) — the one place the rule
  lives, so the export-name side and the list side cannot drift. `is_referees()` / `is_balls()` /
  `is_reserved()`. Splitting an export display name into its first token is export-format
  knowledge and stays in `aesthetics_export::identity`, which calls `TeamName::new`.
- `id.rs` — `TeamId`: validated 701–920. Referees' fixed 999 is not a `TeamId` (see the Team
  compiler plan, "Export identity resolution").
- `file.rs` — `TeamsList`: parse / write `teams_list.txt` per the contract in the Team compiler
  plan ("Export identity resolution" and the "`teams_list.txt` contract" open question): tab-split,
  `ID` and `Name` columns located by header label in any order, every other cell carried verbatim
  and written back in place, Name folded through `TeamName::new` on load, rows that do not fold to
  a slash-wrapped name kept verbatim as inert placeholders whose numeric ids still count in the
  duplicate check; lookup by `TeamName`; the embedded upstream list as a `const`.
- `reconcile.rs` — the merge of an incoming list (embedded upstream, or a savefile's team table)
  into the working list: rows only in the incoming list added, rows only in the working list kept,
  conflicts (same name, different ID) taken from the incoming list, an incoming team whose ID a
  placeholder holds taking that slot, then uniqueness of IDs validated on the *final* mapping (so
  two teams swapping IDs merge cleanly) with every incoming change behind a remaining collision
  reverted; returns a `MergeSummary` (added / kept / overridden / unresolved) for the caller to
  show before writing. Never writes: callers own filesystem I/O and the read-only handling
  (`teams_list_read_only`).

Consumers: `aesthetics_export` (identity resolution), `team_compiler` (ID cell write),
`save_editor` (`export-teams-list`), `studio` (post-update merge). `wasm32`-clean. Tested against
Red's current `teams_list.txt` as a fixture (220 rows, `/umaJP/` mixed case, `/@/`, the `Backup N`
placeholders) plus reconcile cases for each summary bucket.

---
