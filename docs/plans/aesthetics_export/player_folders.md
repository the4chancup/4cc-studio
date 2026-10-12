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
