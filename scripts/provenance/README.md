# Provenance scripts

One-shot scripts and one harness that produced the fixtures and measurements
cited from `docs/plans/`, `docs/DECISIONS.md` and the `tests/fixtures/README.md`
files. Each ran once on the writing machine and hardcodes that machine's paths;
most need the legacy tool sources (`4cc-aet-converter-19to16`,
`pes-stadium-compiler`) and PIL/numpy. Nothing here is called by a `just` recipe
or a gate. They are kept as evidence of method, so a number in a plan can be
traced back to how it was measured, not as maintained tooling.

- `fixtures/` — generated the committed test fixtures for `pes_savefile`,
  `archives`, `fox2` and `dds_convert`, cited by their fixture READMEs.
- `pes_model/` — the `.model` layout census behind the section table in
  `docs/plans/libs/format_crates.md` and the count note in `docs/DECISIONS.md`.
- `fmdl_bone_matrix/` — the bind-pose noise census, gap and histogram scripts
  behind the `1e-4` merge threshold in `docs/DECISIONS.md` (`libs/format_crates.md` carries
  the numbers).
- `kit_uv/` — the kit UV island measurement and diff images behind the Fox to
  pre-Fox layout notes in `docs/plans/team_compiler/pipeline.md` and `docs/DECISIONS.md`.
- `calib/` — a standalone Rust crate (own `[workspace]`, so it stays out of the
  root one) that measured kit color calibration for `docs/plans/libs/color_tools.md`.
  Written against the `dds_convert`/`kit_config` API of its day; it currently
  compiles (`cargo check` clean as of the move).
