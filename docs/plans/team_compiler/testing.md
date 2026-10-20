# 4cc Studio — Team compiler plan: Testing

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Testing: parity against Red (adapted from Blue)

What exists of this is in `spec/team_compiler/testing.md` "Testing: parity against Red (adapted from Blue)".

Since the Rust compiler only accepts the Studio export format, the parity test becomes a two-step flow:
**upgrade the old reference exports with the Export upgrader, compile the result, and compare the
game-facing output** (nested CPK contents, bins, packed structures) against Red's reference output.

Beyond the tracers, whose small trees are committed, the parity over the maintainer's whole
library of old exports is a **measurement rerun at each phase close** that changes the Team
compiler or the Export upgrader, not a committed test: every input the upgrade census upgrades
clean is upgraded, compiled, compiled by Red from a fresh copy, and compared by `red_parity`
(`team_compiler/examples/red_parity.rs`, one classified line per Red entry), and every line is
attributed to an intentional class or a defect, a new class diagnosed before the phase closes.
We do not commit a hash manifest of Red's trees as a fixture, because the inputs are members'
exports on the maintainer's machine alone (several hundred, several GB), so no such test could
run in CI; and because our output differs from Red's by design in 28 classes (6.12c), so a
manifest of either output would be a golden that every intentional change rewrites, while the
attribution, which needs judgement on each new class, is what the run is for. One run takes
about 20 minutes of Red and 17 of ours over 250 exports (6.12c), sequentially. The scripts are
kept under `scripts/library_parity/`.

- **Kit-dependent path magic**: The comparison
  normalizes Red's `u0???p<d>` to `kit<d>` (`p0` → `kitN`) in file names and in decoded FMDL/MTL/XML
  paths before diffing.

The versioned test matrix also covers these key behavior areas:

- **Rosters and identity**: a refs export without `players.txt` blocked by the compiler yet
  repairable in the Refs arranger.
- **Models and textures**: a glTF image shared by two model parts loaded and emitted exactly once.
- **Infrastructure**: deterministic converted output across fresh processes with randomized hash
  seeds.
- **Dispositions and GUI**: pass-through and `DoneWithErrors`; an Error-level `DropFile` rendering
  red at check time and finishing `DoneWithErrors`; `DropFolder` outcomes; an effective `DropExport`
  blocking every row cell; an invalid `ParsedAestheticsExport` still producing stable GUI rows and scoped
  messages; stale validation event rejection; disabled exports producing a log entry but no row; a
  failed kit showing base-bin colors; catalog completeness (every emitted code has a template,
  context placeholders are satisfied, code+scope grouping is stable, CLI and GUI render the same
  underlying message).
- **Deployment and environment**: no FPC
  markers on an already-FPC team (pre-Fox: an already-FPC savefile; Fox: already-FPC installed
  rows), configs untouched; atomic `players.txt` replacement
  never exposing a half-written roster to the
  watcher.

GPU mode may produce different valid encoded texture bytes
across devices/drivers, and containers containing those textures may consequently differ too.
That exception does not relax model/bin output, asset identities, collision decisions, or ordering;
GPU checks compare decoded texture semantics and quality rather than demanding CPU byte equality.
