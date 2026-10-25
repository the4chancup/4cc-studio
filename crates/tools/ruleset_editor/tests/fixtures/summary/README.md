# Summary goldens

The summary of each embedded ruleset (`libs/aatf`'s `rulesets/`), as the draft's own `summary()`
(`docs/prototypes/tools/ruleset_editor.js`, "Readable rules") writes it: the text the Summary
section shows and `ruleset-editor summary` prints (`plans/ruleset_editor.md` "The editor",
RE-SUM-01, RE-CLI-03). Lead-written: do not edit. A wording change is made in the draft first,
then these are regenerated.

Each ruleset was fed to the draft as `libs/aatf`'s writer writes it, so every map is in the Rust
type's order (a tier's stats in stat order, its free allowances by group id); a per-tier value
list is in the tiers' order. Regenerate with `scripts/provenance/fixtures/ruleset_summary_dump.rs`
then `ruleset_summary_goldens.py` (their headers say how). UTF-8, LF line ends, one newline at
the end; the weight rule's minus is U+2212.
