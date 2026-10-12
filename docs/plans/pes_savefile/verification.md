# 4cc Studio — Savefile plan: Verification

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Verification

What exists of this is in `spec/pes_savefile/verification.md` "Verification".

- **Settings completeness**: Phase 5 adds the `unknown`
  class and takes the assertion to bit level, every bit of the block named, unknown, or
  compiler-owned ("Player settings model" in `model.md`, "Planned for Phase 5").
- **Patch equivalence**: compiling a fixture export against a save, and compiling it without a save
  then applying the produced patch to a copy of that save, yield byte-identical savefiles; applying
  a second patch covering a subset of teams changes only those teams' players and only the fields
  it carries; a patch with a different `pes_version` or a different `allocation_scheme_version` is
  refused without touching the save.
- **Texport write**: import the generated file in the actual game (manual, per version; a `new`
  file and an edited round-tripped one).
