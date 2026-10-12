# 4cc Studio — Savefile plan: Verification

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Verification

What exists of this is in `spec/pes_savefile/verification.md` "Verification".

- **Patch equivalence**: compiling a fixture export against a save, and compiling it without a save
  then applying the produced patch to a copy of that save, yield byte-identical savefiles.
- **Texport write**: import the generated file in the actual game (manual, per version; a `new`
  file and an edited round-tripped one).
