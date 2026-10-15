# 4cc Studio — Aesthetics export plan: Player folders

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Player folders (the Studio export format)

What exists of this is in `spec/aesthetics_export/player_folders.md` "Player folders (the Studio export format)".

A team export's root also admits the valueless `write_names` marker: the aesthetics patch writes
its players' names ("Player settings in exports" in [`settings_toml.md`](settings_toml.md)).

The block size is fixed for the lifetime of a scheme version — IDs are baked into distributed
outputs — so builds record `allocation_scheme_version = 1` in their metadata.

**A `Midcup` export keeps the shared IDs the team already has** (spec step 2, "Assigns IDs
automatically"). A cup's Midcup export carries only the shared boots and gloves folders that
changed, and its CPK loads above the cup's main one, whose rows stay for every player it does
not compile; numbering its own folders alphabetically would give a new folder B the ID an
absent folder A holds below, and A's wearers would wear B. So every compiled CPK records, per
team, which shared folder took which ID, in `4cc_studio/shared_ids/<team_id>.toml` (a path no
game reads; on both engines):

```toml
# Written by 4cc Studio; do not edit.
[boots]
1690 = "Crocs"
[gloves]
1684 = "Grip test"
```

A `Full` export numbers its folders alphabetically, as now, and records them. A `Midcup` export
first reads the team's record from the installed CPKs below its output (the topmost holding
one, the walk "Bins accumulation" makes for the working bins; `check` reads it too, so the
two report the pool and `shared_ids_unrecorded` alike): a folder named there, compared
case-folded as folder names are, keeps its ID; a folder not named takes the lowest of the
team's 17 shared IDs of its kind neither the record nor an earlier new folder holds,
alphabetically; and the CPK records the installed entries with the new ones, each folder the
Midcup holds under its own spelling. When no installed
CPK holds a record (the main CPK built by Red, or before this rule), the Midcup numbers its
folders alphabetically and reports `shared_ids_unrecorded` (a warning naming the team): its IDs
may be ones the main CPK gives other folders. A Midcup whose new folders do not all fit among the
free IDs is refused as a Full export with more than 17 is (`boots_id_pool_exhausted`,
`gloves_id_pool_exhausted`). We record in the CPK rather than in the
export or a Studio setting because the CPK is what every member installs and compiles over:
the record travels with the IDs it describes, and a Midcup compiled on another machine reads
the same one. We do not require a Midcup to carry every shared folder, which would make
members copy folders they did not change.

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
