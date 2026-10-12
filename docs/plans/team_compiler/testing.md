# 4cc Studio — Team compiler plan: Testing

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Testing: parity against Red (adapted from Blue)

What exists of this is in `spec/team_compiler/testing.md` "Testing: parity against Red (adapted from Blue)".

Since the Rust compiler only accepts the Studio export format, the parity test becomes a two-step flow:
**upgrade the old reference exports with the Export upgrader, compile the result, and compare the
game-facing output** (nested CPK contents, bins, packed structures) against Red's reference output.

The extracted trees are several GB of binary data and are **not committed
to git**; instead, a **hash manifest** (content hash per file, keyed by normalized relative path)
is committed as the test fixture. The test regenerates the reference tree locally by running Red +
extraction when the manifest is stale or the local tree is missing, and verifies the regenerated
tree against the committed manifest before using it. This keeps the repo small while making the
test reproducible without a pre-shared binary blob.

The versioned test matrix also covers savefile success/failure and cancellation, and these key
behavior areas:

- **Dispositions and GUI**: pass-through and `DoneWithErrors`; an Error-level `DropFile` rendering
  red at check time and finishing `DoneWithErrors`; `DropFolder` outcomes; an effective `DropExport`
  blocking every row cell; an invalid `ParsedAestheticsExport` still producing stable GUI rows and scoped
  messages; stale validation event rejection; disabled exports producing a log entry but no row; a
  failed kit showing base-bin colors; catalog completeness (every emitted code has a template,
  context placeholders are satisfied, code+scope grouping is stable, CLI and GUI render the same
  underlying message).
- **Deployment and environment**: the installed ID kept for players whose boots/gloves task failed
  (pre-Fox: his savefile ID) while independent settings
  still apply; with either FPC marker, successful standalone assets
  overriding preset IDs, failed/dropped requested assets preserving existing IDs, and absent
  standalone assets using preset IDs (including pre-Fox face-XML-local parts); no FPC
  markers on an already-FPC team (pre-Fox: an already-FPC savefile; Fox: already-FPC installed
  rows), configs untouched; a mixed `fpc_on`/`fpc_off` team applying
  per-player presets; atomic `players.txt` replacement
  never exposing a half-written roster to the
  watcher.

GPU mode may produce different valid encoded texture bytes
across devices/drivers, and containers containing those textures may consequently differ too.
That exception does not relax model/bin output, asset identities, collision decisions, or ordering;
GPU checks compare decoded texture semantics and quality rather than demanding CPU byte equality.
Savefiles use fresh randomized encryption salts, so compare them after decryption and semantic
normalization, or inject a deterministic RNG in tests.
