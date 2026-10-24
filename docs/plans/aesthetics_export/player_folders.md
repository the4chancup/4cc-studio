# 4cc Studio — Aesthetics export plan: Player folders

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Player folders (the Studio export format)

What exists of this is in `spec/aesthetics_export/player_folders.md` "Player folders (the Studio export format)".

The Export upgrader generates all the root files from the old Team Note txt and `Logo/` folder when
migrating.

At compile time, after the spec's steps 0 to 3, the pipeline:

4. **Plans the settings side** for assigned IDs and independent accepted settings — conditional
   savefile mutations on pre-Fox, the compiled player's `PlayerAppearance.bin` and boots/gloves
   rows in the output CPK on Fox. Producer commits activate mutations that reference compiled
   assets; after all outcomes are known, only activated asset mutations plus eligible
   independent settings are serialized (`pes_savefile`'s savefile write on pre-Fox, the resolved
   rows in the CPK's tables on Fox). This
   makes the player wear successfully emitted boots/gloves without pointing at failed content — the
   coordination currently done manually with 4ccEditor.

### `ingame_face.toml` (new; step 0.0.1h)

The `ingame_face` marker (spec "`ingame_face` marker") becomes a file, **`ingame_face.toml`**,
holding the in-game face, and `settings.toml` loses the keys it takes:

- **What moves.** The template's `[appearance.face]` table, `[appearance.hair]` table and
  `iris_color` (spec `settings_toml.md`, the template), 90 of its 132 keys: in the file they are
  the top-level `iris_color` and the tables `[face]` and `[hair]`, with the same keys, ranges,
  defaults, per-version rules and comments. `skin_color`, the physique (head size included),
  the strip, the motions and the boots/gloves IDs stay in `settings.toml`, since a player
  with a face model wears them too.
- **Why they move: the game draws them on the in-game face alone.** Measured on 2026-10-24 in
  PES 21 and PES 17 Edit mode (a face model without a hair model, the same with one, and the
  marker, each with long red hair and a blue or violet iris set): beside a face model the game
  draws neither the in-game hair nor the record's iris, the model's own eyes standing (1186 of
  the machine's 1472 Fox face models carry no eye material of their own; the PES 17 sample paints
  its eyes into its texture); the marker's player draws both. The face sliders and feature
  types shape the in-game face's own mesh and textures. So 19 players in 20, who wear a face
  model, carried 90 keys that change nothing, in the file a member reads first; a key that
  changes nothing reads as a bug when it is set and nothing happens.
- **The document.** `pes_savefile`'s `settings_toml` module reads and writes it beside
  `settings.toml`, from the same key table: its keys sit at their `[appearance]` paths with
  the root dropped (`iris_color`, `face.cheek_type`, `hair.length`), in the same ranges as
  `settings.toml` (the widest per key). Team TOML keeps all of them under
  `[players.NN.appearance]`, unchanged. The player's settings stay one value, joined from the
  two files, so a row or the patch is built from one `PlayerSettings` as today:

  ```rust
  /// A player's `ingame_face.toml`: the in-game face's keys, each `None` when absent.
  pub struct IngameFaceSettings {
      pub iris_color: Option<u8>,
      pub face: FaceSettings,
      pub hair: HairSettings,
  }
  impl IngameFaceSettings {
      pub fn parse(text: &str) -> Result<Self, SettingsError>;
      /// The template: the header below, then the `iris_color` line, `[face]` and `[hair]`.
      pub fn to_toml(&self) -> Result<String, SettingsError>;
  }
  impl PlayerSettings {
      /// The in-game face's keys, as `ingame_face.toml` holds them.
      pub fn ingame_face(&self) -> IngameFaceSettings;
      /// The in-game face's keys set from `face`, an absent key unset.
      pub fn set_ingame_face(&mut self, face: IngameFaceSettings);
  }
  ```

  The template's key lines are `settings.toml`'s (the `iris_color` line, then the
  `[appearance.face]` and `[appearance.hair]` blocks under the headers `[face]` and `[hair]`),
  verbatim but for those headers; its header is:

  ```toml
  # ingame_face.toml, inside a player folder: the in-game face, which the game draws when
  # the folder holds no face model and no face link. The file marks the folder as wearing
  # it. Every key is optional: an absent key takes the value shown here, and an empty file
  # is the game's default face. A commented key shows what can be set and its range.
  ```

  `settings.toml`'s header gains a closing sentence on its first paragraph, "The in-game
  face's keys (iris, face, hair) are in ingame_face.toml.", and a TOML syntax error names no
  file in its text (`not valid TOML: …`), since the finding's line names the file.
- **The file is the marker.** Its presence does all the marker did (no face folder emitted, the
  face-classified parts removed, `ingame_face_explicit_face_model` beside a face model or face
  link); its content sets the face. An empty file is today's bare marker: every key at its
  default, the game's default face (the spec's "The defaults are the game's default face").
  `PlayerFolder.ingame_face` becomes `Option<FileDescriptor>`, beside `settings`
  (`object_model.md` "Player folders"), and the file is a settings-stage input like
  `settings.toml`: read by the deep pass, its findings reported by `check` and `compile` alike.
- **A player without the file gets the defaults** for the 90 keys: on Fox his row's face, hair
  and iris fields hold the game's default face (not drawn, his face model standing), as an
  absent key does today; on pre-Fox the same when the patch's pre-Fox half is built
  (`plans/pes_savefile/operations.md` "Aesthetics patch (new)").
- **Findings.** A file that is not UTF-8 or fails to parse (an unknown key included) is
  `ingame_face_toml_invalid` (E, the file and the error): its keys are ignored and the player
  keeps the in-game face at the defaults, as `settings_toml_invalid` ignores its file. A value
  out of the target's range is `settings_value_out_of_range`, as in `settings.toml`. A moved key
  left in `settings.toml` is an unknown key there, so `settings_toml_invalid`, its message
  naming the key; the help says where the face keys went. The old marker, bare `ingame_face`
  or `ingame_face.txt`, is `ingame_face_marker_renamed` (E, the file): recognized only to say
  so, it marks nothing, and the folder compiles as one without the file. The compiler stays
  strict ("The upgrader absorbs the old format's slack"): a Red export's marker reaches Studio
  through the upgrader, and a member typing the old name by hand is told the new one.
  `ingame_face.toml.txt` is not recognized, as `settings.toml.txt` is not.
- **The Export upgrader** writes `ingame_face.toml` wherever it writes the marker today (its
  steps 3 and 12, EU-PLY-02, 04, 16, 17, the loose mode), filled at step 8 from the record:
  the face run, the hair and the iris, every key the save's version holds, as `settings.toml`
  is (`settings_value_refused` and `settings_from_stripped_record` cover both files). A player
  without the file gets a `settings.toml` without the 90 keys. Without a record to generate
  from (no save, `settings_not_generated`) the file is written empty, so the marker's effect
  survives.
- **The Save editor** generates the two files apart: its `settings.toml` generation writes no
  face keys, and a second action, "Generate in-game face", writes `ingame_face.toml` from the
  record. A save cannot say which file a player needs: the cup keeps every "modified face" flag
  off, the game choosing the face folder whenever one exists for the id (spec "`ingame_face`
  marker"), so the member who knows the folder picks. A stripped player is skipped by both, as
  now.
- **Team TOML is not affected**: it is the save's interchange, every field of the record in
  one file; the split is the export's.

When it lands, the spec's text moves with it: `player_folders.md` (the tree, "`ingame_face`
marker", "`ingame_face` with shared links" and every later mention), `settings_toml.md` (the
template split in two, the ingame-face run's three rules under the new file),
`object_model.md` (`PlayerFolder`, the file-kind table's row for a player folder's direct
files), `team_compiler/messages.md` (the two new codes), `export_upgrader.md` (steps 3, 8, 12,
the loose mode), and the scenarios below replacing their spec namesakes. A scenario that names
the marker only as a folder's content (TC-STR-11 and 14, TC-MOD-14 to 19, 32, 35, 41, 44, 45,
62, 67, 85, 94, TC-CMN-24, EU-PLY-02, 16, 17, EU-LOOSE-01, 03, EU-MF-01) reads `ingame_face.toml`
for `ingame_face` and is otherwise unchanged. TC-STR-20, TC-BIN-48, TC-BIN-49, EU-PLY-22,
EU-SET-10 and EU-SET-11 replace TC-STR-10, TC-BIN-38, TC-BIN-31, EU-PLY-04, EU-SET-01 and
EU-SET-05, which are then withdrawn.

The other consumers change in their own plans, unbuilt: the Player aesthetics editor's forms
and its in-game face workflow (`plans/player_aesthetics_editor.md`), the Team creator's
In-game head starter (`plans/team_creator.md`), and the Save editor's Appearance lock text and
generation (`plans/save_editor.md`).

#### Acceptance (0.0.1h)

Fixtures as their spec namesakes'; /co/ is team 714, /egg/ 792.

```
TC-STR-20  GIVEN /co/ slots 05, 06 and 07 holding, in turn, ingame_face.toml, a bare
                 ingame_face and ingame_face.txt, and no model
           WHEN the export is checked, then compiled for PES 21
           THEN both runs report ingame_face_marker_renamed for 06's and 07's files and
                neither reports a disallowed file nor a finding on 05; the CPK holds no
                face/real/71405/ path and a blank face package for 71406 and 71407, as for
                a folder with no marker
TC-STR-18  GIVEN /co/ slot 05 holding an ingame_face.toml that does not parse, slot 06 one
                 holding an unknown key, each beside a settings.toml setting skin_color = 4,
                 and no model
           WHEN the export is checked, then compiled for PES 21
           THEN both runs report ingame_face_toml_invalid for each, naming the file and the
                error; the CPK holds no face/real/71405/ or 71406/ path, and both players'
                PlayerAppearance rows hold skin colour 4 and the template's defaults in every
                face, hair and iris field
TC-STR-19  GIVEN /co/ slot 05 holding a settings.toml with [appearance.hair] and
                 skin_color = 4, and an ingame_face.toml setting iris_color = 7, and no model
           WHEN the export is checked, then compiled for PES 21
           THEN both runs report settings_toml_invalid naming appearance.hair as an unknown
                key, and 71405's row holds iris 7 and the template's skin colour, 1
TC-BIN-48  GIVEN a Midcup /co/ export whose only player folder, slot 05, holds an
                 ingame_face.toml setting a facial hair type and a settings.toml setting
                 boots_id = 12, and no model
           WHEN it is compiled for PES 21
           THEN the output CPK is written and holds the three player tables, with (71405, 12)
                in BootsList.bin and 71405's appearance row from his two files, the facial
                hair type the ingame_face.toml's
TC-BIN-49  GIVEN a /co/ export whose slot 05 ingame_face.toml holds facial_hair_type = 20
                 (PES 18 unmeasured: the field's 0 to 31; PES 21: 0 to 19) and in-game hair
                 (style = 4, color_red = 40), and whose settings.toml holds
                 hunching_dribbling = 5 (PES 18: 1 to 3) and dribbling = 2 (PES 20 and 21
                 only)
           WHEN it is compiled for PES 18, and for PES 21
           THEN the PES 18 run reports settings_value_out_of_range for hunching_dribbling
                alone, 71405's row has facial hair type 20, its patch holds
                hunching_dribbling 1 and no dribbling key, no finding names dribbling; the
                PES 21 run reports it for facial_hair_type alone and its row has facial hair
                type 0; both runs' rows carry that hair; both exit 0
TC-BIN-47  GIVEN /egg/ slot 05 holding face_high.fmdl and a settings.toml, and slot 06
                 holding an ingame_face.toml setting iris_color = 7 and [hair] length = 5
           WHEN the export is compiled for PES 21
           THEN 79205's PlayerAppearance row holds the template's defaults for every face,
                hair and iris field, and 79206's holds iris 7 and hair length 5, its other
                face and hair fields the defaults
EU-PLY-22  GIVEN one player folder holding ingame_face and another ingame_face.txt, in a team
                 export and in a referee export
           WHEN each export is upgraded, the team one with a save, the referee one with
                neither flag
           THEN each of those player folders holds ingame_face.toml and no ingame_face or
                ingame_face.txt; the team export's two files hold their records' face, hair
                and iris, every key the save's version holds, and their settings.toml none of
                those keys; the referee export's two are empty
EU-SET-10  GIVEN a player with a face folder, and his save record
           WHEN the export is upgraded
           THEN his settings.toml sets every key the save's version holds but the face, hair
                and iris keys to his record's value, shirt_name to his save string, and has no
                height or weight key, and his folder holds no ingame_face.toml
EU-SET-11  GIVEN the PES 18 fixture save, holding a player with cheek_type 5 whose slot the
                 export gives no face folder and custom boots
           WHEN his team's export is upgraded with it
           THEN his ingame_face.toml has cheek_type = 5, and nothing is reported about it
EU-PLY-21  GIVEN a face folder for slot 03 and no face folder for slot 05, whose record wears
                 custom boots
           WHEN the export is upgraded with a save
           THEN Players/03's settings.toml holds no face, hair or iris key and no
                ingame_face.toml exists there; Players/05 holds ingame_face.toml with his
                record's face, hair and iris
SE-GUI-33  GIVEN a player whose record has hair length 5 and iris 7
           WHEN settings.toml generation, then Generate in-game face, run on his folder from
                the view
           THEN settings.toml holds no face, hair or iris key, and ingame_face.toml holds
                iris_color = 7, [hair] length = 5 and every other key the save's version holds
SE-GUI-34  manual  GIVEN a PES 21 player whose record has a non-default in-game face (long red
                         hair, blue iris, a slider moved), his settings.toml and
                         ingame_face.toml the view generated from it, then his record changed
                         in Edit mode to another non-default look (short blond hair, green
                         iris) and the save stripped (strip-and-seed), the folder compiled into
                         the test slot
                   WHEN he is opened in PES 21's Edit mode
                   THEN he has the generated look (long red hair, blue iris), neither the
                        changed record's nor the default face
```

## Complete export (an aesthetics export with its tactical file and its music)

A team's aesthetics export may carry its tactical half as a file at its root, `tactics.toml`:
a Team TOML (`plans/pes_savefile/operations.md` "Team TOML") written by the Save editor's
export, and its music export as a root folder, `Music/`, so one folder holds everything a team
sends to a cup, to be compiled for any version. Each part is optional: an aesthetics export
alone, a tactical export alone (a Texport or a Team TOML) and a music export alone keep working
as they do. The names follow every export's rule, folder names capitalized and file names
lower case (`Players/`, `Kits/`, `settings.toml`).

- **Each half owns its fields, so no field has two sources.** The aesthetics half owns the
  looks (the player folders, `settings.toml`, kits, the team's root files); `tactics.toml` owns
  names, gameplay and tactics, and carries no looks: an appearance table or a kit or colour key
  in it is an export-check error naming the table, and the Save editor's export offers the
  tactical file without them (Team TOML's "absent = untouched" makes such a file valid).
- **Names come from the tactical file when there is one.** The aesthetics patch writes names
  only for an export without `tactics.toml`, which keeps the autopilot teams' route (the
  `write_names` marker) as it is; a marker beside `tactics.toml` is a warning saying the
  tactical file's names are used.
- **One fixed name, recognized everywhere.** `tactics.toml` at the root is a known root file
  for the Team compiler, the Export upgrader (which carries it over untouched) and the export
  checks (which parse it as a Team TOML, its texts against the portable limits); any other
  `.toml` at the root stays `root_file_unexpected`, as now.
- **`Music/` is the music export as it stands alone**: one `.4ccm` directly inside (its file
  name free, as music exports' are) and the songs it names, resolved against that folder as a
  `.4ccm`'s always are (`plans/music_player.md` "File resolution"), so the folder copied out
  loads in the Music player or in Rigdio unchanged. It is a known root folder of a team
  export: the Team compiler never reads its songs and packs nothing of it, the Export upgrader
  carries it over untouched, and the export checks parse its `.4ccm` and resolve its songs.
  What they find is a Warning, never an Error, because no compiled output depends on it (a
  missing goalhorn cannot spoil a DLC); a `Music/` holding no `.4ccm` or several, and a
  `.4ccm` whose `name` is not the export's team (the music of another team bundled by
  mistake), are Warnings of their own. A referee export has no music: `Music/` there is an
  unexpected root folder, as now.

**Who uses it.** An official cup keeps two exports, because it has two deadlines (aesthetics,
then tactics) and often two builders: the DLC builder never sees the tactics. A Complete export
is made at the end of a cup, to collect a team so it can be entered easily into an invitational,
which has one deadline and one organizer building both the DLC and the save. So the compiler
applies `tactics.toml` in its savefile stage, after the aesthetics patch, when a local save is
configured (the "Savefile update" stage, `spec/team_compiler/pipeline.md`); without one it is checked and
otherwise left alone, and the compiler carries nothing of it into its outputs. The streamer is
usually someone else, and the organizer often gathers every team's music into one download
for them: with the music inside each export, that download is the entered exports' `Music/`
folders, one per team, and the Music player's auto team loading finds each by its `.4ccm`'s
`name`, with a folder of whole Complete exports working as its library too
(`plans/music_player.md` "Autopilot").

**The two halves name the same players.** Team TOML addresses players by roster slot, the
aesthetics export by savefile identity (`team id × 100 + NN`); by 4cc rule a team's roster slot
`NN` holds player `team id × 100 + NN`, so slot and identity agree by definition and nothing
reconciles them. PES lets a roster be reordered, so the rule is checked rather than assumed:
the AATF check reports a team that breaks it (`plans/aatf_rules.md` "The host"), and the
compiler refuses to apply `tactics.toml` onto such a team in the local save, with an error
naming the slots, since its players would land on the wrong ids.
