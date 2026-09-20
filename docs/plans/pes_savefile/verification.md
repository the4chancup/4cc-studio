# 4cc Studio — Savefile plan: Verification

Part of the [Savefile plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Verification

- **Container roundtrips**: decrypt → re-encrypt → decrypt again must reproduce
  the payload byte-for-byte for every version (encryption uses a random salt, so
  ciphertexts differ; plaintexts must not). Known-answer tests for the MT19937
  keystream against the Python reference.
- **Codec roundtrips**: read → write → byte-compare for every player and team
  block in real saves of all seven versions (`Midcupping/EDIT.bin` and friends
  are test fixtures; the user's KONAMI `eFootball PES 2021 SEASON UPDATE` save
  is the 20/21 fixture — sanity anchors already verified there: player count
  u16 at 0x60, and every record's appearance playerID at +240 equals its player
  ID at +0).
- **Name fields**: names read as null-terminated UTF-8 (the `♂` in a real PES 19
  name is the bytes `E2 99 82`, colour codes are the ASCII bytes `11 63` + hex)
  from real PES 19 (`PRO EVOLUTION SOCCER 2019` KONAMI save) and PES 21 cup
  saves through the AET compiler Red's `savefile.py` reader, which shares this
  plan's offsets; `display_name()` must strip every colour code in those saves'
  ~30 decorated names and leave every undecorated name unchanged.
- **Export settings versus save interchange**: generated `settings.toml` omits boots/gloves IDs,
  and authored ID keys are rejected; full Team TOML still round-trips those IDs as player-record
  data. Compiler ID assignment remains independent of the authorable settings serializer.
- **Settings completeness**: for every version, each appearance field in the schema table is
  marked `settings` or `compiler_owned`, and the `settings` set equals `PlayerSettings`' fields;
  the generated template contains every `PlayerSettings` key (a commented line for each unset one).
- **Patch equivalence**: compiling a fixture export against a save, and compiling it without a save
  then applying the produced patch to a copy of that save, yield byte-identical savefiles; applying
  a second patch covering a subset of teams changes only those teams' players and only the fields
  it carries; a patch with a different `pes_version` or an older `allocation_scheme_version` is
  refused without touching the save.
- **Cross-implementation parity**: field values must match 4ccEditor's display
  for the same save; aesthetics fingerprints must match `compare-saves-*.py`
  output; transplant output must byte-match `transplant-aesthetics-*.py`.
- **Cross-version conversion**: compare against the converters' verified behavior on real team
  data, excluding their documented stale PES20/21 offsets. Assert corrected name and appearance
  fields independently against the target schema; reproducing the known converter corruption is
  not parity.
- **Interchange formats**: Team TOML `from_team` → `to_toml` → `parse` → `apply` onto a copy of
  the same team reproduces the team and its players field for field on every fixture save, and
  `to_toml` of the parsed document is the emitted text again; a partial file touches only the
  fields it carries (a one-key file leaves everything else byte-identical after `write_player`);
  `.4ccs` decodes the real PES 19 fixture file's 23 players against the ctypes offsets held as
  literals in the golden test; the `.4cct` fixture's tactics equal the schema codec's for the same
  team; Texport `from_bytes` → `to_bytes` is byte-identical on the real 17, 18, 19 and 21 fixtures
  and every record decodes to the same values the fixture save's codec produces for the same
  layout; the 18/19/21 key indices and layouts are measured, 15/16/20's are the reference's.
- **Texport write**: import the generated file in the actual game (manual, per version; a `new`
  file and an edited round-tripped one).
