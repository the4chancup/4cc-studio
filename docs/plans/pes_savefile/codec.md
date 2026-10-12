# 4cc Studio — Savefile plan: Save codec

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Schema-driven save codec

### Savefile discovery

What exists of this is in `spec/pes_savefile/codec.md` "Savefile discovery".

- **Account folders**: `auto` mode takes the newest and reports which one it chose
  (`savefile_autodetected`, Info, with the path) so a user with two accounts can pin `savefile_path`
  if the guess is wrong.
- **The version is verified after discovery**, not assumed from the folder: the candidate is loaded
  with the ordinary `EditFile::load` auto-detection and must match the requested version, otherwise
  it is reported as a mismatch rather than used (a stale save left by a previous game version in a
  reused folder must not be edited as if it were current).
- **Nothing found** → the consumer's own message (`savefile_missing` in the compiler; an empty
  quick-open list plus the ordinary file picker in the editor).
