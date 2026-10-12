# 4cc Studio — Savefile plan: Data model

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Player settings model (settings.toml)

What exists of this is in `spec/pes_savefile/model.md` "Player settings model (settings.toml)".

**Every field is optional**: applying a `PlayerSettings` only touches the
savefile fields it actually specifies (`Option<T>` throughout; `None` = leave
the savefile value alone). This includes the player name — name writing is
opt-in (`name = true` derives it from the folder, a string sets it explicitly,
absent means untouched); the derivation rules are the Team compiler's, and this
crate just applies whatever the resolved `PlayerSettings` carries.

**Planned for Phase 5** (step 4.0's result, decision entry "aesthetics travel in the database
tables"; the description above and below is the Phase 2 code until then):

- `Height` and `Weight` leave the key table (classified `Gameplay`); Team TOML carries them as
  player-level keys of `[players.NN]` instead of inside its appearance table.
  `shirt_name: Option<String>` joins `name` at the top level, string-only, checked against
  the version's shirt-name field (its full length, `spec/pes_savefile/codec.md` "Text fields hold `len` bytes").
- **Absent = default.** Applying a `PlayerSettings` writes every appearance field, a `None` taking
  the value the template shows (`settings_toml.md`'s template block; a `default` column of
  `KeySpec`); only `name` and `shirt_name` keep `None` = not written. `boots_id`/`gloves_id`
  are not appearance fields: a `None` there is the compiler's to resolve (the folder's models,
  the FPC marker), so `apply` leaves the stored ID alone. The template's "unset ones as
  commented lines" stays, now meaning "at its default", each line showing that default. A key
  the target version has no field for (`dribbling` before PES 20) is skipped, not
  `NotInThisVersion`: an export written for a newer version compiles for an older one as it is
  (`settings_toml.md`). Team TOML's apply, a whole-team transfer, keeps both "absent =
  untouched" and the error.
- **Unknown bits become authorable as raw values.** The ownership classification gains the
  complement of the decoded bits inside the appearance block (`Unknown`), all of them in the
  ingame-face run on every version once the appearance id below is modeled, and
  `AppearanceSettings` gains `[appearance.unknown]`'s one key, `face`: the run's bytes with
  the named bits zeroed (format in `settings_toml.md`). `from_player` emits it; applying
  writes its undecoded bits as far as the target's run reaches (absent = 0), so `apply`
  gains a `PesVersion` parameter, since `PlayerEntry` is version-independent and a PES 20
  entry looks like a 21 one. The completeness test then asserts, per version, that every bit
  of the block is named, unknown, or compiler-owned, exactly once, and that no unknown bit
  lies outside the run. Team TOML's `ingame_face` hex goes: `face` carries the same bits,
  across versions as the hex's prefix copy does, so each bit has one carrier: its import
  writes them only as far as both runs reach, and a longer target run keeps its tail (Team
  TOML's absent = untouched; a `settings.toml`'s apply writes it 0, absent = default).
- **The Fox database row.** A `PlayerSettings` (with its defaults) also produces the
  `PlayerAppearance.bin` row: the player id, then the record's 56 appearance bytes (everything
  after its id, edit flags, boots/gloves and base-copy fields; record bytes 128 to 184 on
  PES 18/19, 252 to 308 on PES 20/21). The row is one layout on every version: the games' own
  tables hold the same bytes for most players from PES 17 to 21, and PES 17's, whose record
  offset in-game Test 4 confirmed, sits at the same place in the same 72-byte block, whose
  fields lie at the same bits on PES 17 to 21 (measured 2026-10-12, `.tmp/5_1/measure/`). The
  run's last 4 bytes (the block's last 4) are in no row, so their undecoded bits do not reach
  a stripped player. A pre-Fox version has no row: PES 17 reads one, but its compile writes
  the savefile instead (`pipeline.md` output table). Boots and gloves go to the
  `BootsList.bin`/`GloveList.bin` pairs.
- The appearance block's player id becomes a modeled field of the entry, outside
  `PlayerAppearance`, read and preserved, so a transplant's whole-block copy leaves the target's;
  only the Save editor's strip operation writes -1 (worklog issue "player id versus appearance id").
  Generation's callers skip a record at -1 (`settings_toml.md`, the end of "Player settings in
  exports"); `from_player` itself reads whatever record it is given.

```rust
/// `settings_toml.rs`
pub enum NameSetting { FromFolder, Explicit(String) }

/// One TOML key of the table; the enum is the table's row set and the `spec` function its
/// columns (table path, key name, kind, widest range, range comment). Physique keys convert
/// between the game number and the stored value (+7); 1-based motion keys between the game
/// number and the stored value (-1); string-enum keys between the listed labels and the stored
/// index.
pub enum SettingKey { SkinColor, IrisColor, Height, /* … every key of the block, in its order */ LowerLipType }

/// Every leaf is `Option`; `None` = leave the savefile value alone. The struct mirrors the
/// TOML tables (top level, `appearance`, `appearance.physique`, `.strip`, `.motion`, `.face`)
/// with the model's storage types (`u8`/`bool`), not the TOML surface types: the string labels
/// and the signed physique numbers exist only at the parse/emit boundary. `boots_id`/`gloves_id`
/// are top-level keys (0 to 100, the stock band, or `""` = `None` = default: the compiler
/// resolves it from the folder's content and FPC marker), outside
/// `AppearanceSettings`, because Team TOML embeds `AppearanceSettings` beside its own full-range
/// player-level IDs. `to_toml` writes a `None` ID as `""`, uncommented.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerSettings { pub name: Option<NameSetting>, pub boots_id: Option<u8>,
                            pub gloves_id: Option<u8>, pub appearance: AppearanceSettings }

impl PlayerSettings {
    /// Parses a file: unknown keys and tables, wrong types, out-of-range values and unknown
    /// labels are `SettingsError`s naming the key; the compiler-owned keys are unknown keys.
    /// An explicit name containing a NUL is `WrongType` (reading a save field stops at the
    /// first NUL, so it would silently truncate on reload).
    pub fn parse(text: &str) -> Result<Self, SettingsError>;
    /// Every key `Some` from the player; `name` is `Explicit(raw name)`, colour codes included
    /// (the Export upgrader then writes `FromFolder` when that name equals the folder's name
    /// part; the function sees no folder).
    /// `boots_id`/`gloves_id` are the exception: `Some` only for a stored ID from 1 to 100 (0 is
    /// the game's default, an ID above 100 is custom content a folder owns), so they are never
    /// `OutOfRange`. Any other stored value the key table cannot express (a 2-bit sleeves field
    /// holding 3, which the reference scripts label "broken") is `OutOfRange` naming the key.
    pub fn from_player(player: &PlayerEntry) -> Result<Self, SettingsError>;
    /// Sets a stored value; `OutOfRange` when the key's kind cannot represent it, so a
    /// `PlayerSettings` built through `set` is always one the emitters can write.
    pub fn set(&mut self, key: SettingKey, value: u8) -> Result<(), SettingsError>;
    /// Writes the `Some` fields onto the player, all or nothing: on `Err` the player is
    /// unchanged. `NameSetting::FromFolder` is not applied: the caller (the Team compiler)
    /// resolves it to `Explicit` first, since the folder rules are its. A `Some` on a field this
    /// player's version lacks (`dribbling` before PES 20) is `SettingsError::NotInThisVersion`.
    pub fn apply(&self, player: &mut PlayerEntry) -> Result<(), SettingsError>;
    /// The complete template: every key in table order, set ones as values, unset ones
    /// commented, each with its range comment. `OutOfRange` for a value written directly into
    /// a public field that the key's kind cannot represent (the fields are plain data).
    pub fn to_toml(&self) -> Result<String, SettingsError>;
    /// Edits an existing document in place (`toml_edit`, comments preserved), setting the
    /// `Some` keys and leaving everything else as the user wrote it. `WrongType` when a
    /// table position holds a non-table (never a panic on a user's file).
    pub fn update_toml(&self, document: &mut toml_edit::DocumentMut) -> Result<(), SettingsError>;
}
```
