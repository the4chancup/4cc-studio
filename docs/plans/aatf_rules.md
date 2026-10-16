# 4cc Studio — AATF rules plan

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

What exists of this is in `spec/aatf_rules.md` "4cc Studio — AATF rules plan".

## The interpreter

What exists of this is in `spec/aatf_rules.md` "The interpreter".

Every surface that shows results (the Save editor's AATF panel, the Team creator's) has a "Show
suggestions" toggle, off by default, and lists them apart from errors and warnings, uncounted.

## Loading and validation

What exists of this is in `spec/aatf_rules.md` "Loading and validation".

A "save a copy" action writes the embedded file out, and a file is validated by the Save editor's
"validate rules" button and by the Ruleset editor before saving. A logic change that needs a new
data shape bumps `schema`; `libs/aatf` keeps a migration from every earlier schema, run by the
update.

## Crate layout

What exists of this is in `spec/aatf_rules.md` "Crate layout".

`src/writer.rs` (Phase 20): `Ruleset` → `RULESET` map literal, deterministic.

## Development phase and verification

What exists of this is in `spec/aatf_rules.md` "Development phase and verification".

The writer (`writer.rs`) and schema migrations arrive with the Ruleset editor in Phase 20.
