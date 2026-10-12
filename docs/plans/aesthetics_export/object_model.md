# 4cc Studio — Aesthetics export plan: Object model

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Object Model

### `aesthetics_export` crate layout

What exists of this is in `spec/aesthetics_export/object_model.md` "`aesthetics_export` crate layout".

The crate gains one file beyond the tree in the spec:

```
crates/libs/aesthetics_export/src/
└── players_txt.rs      # players.txt writer (Refs arranger) with atomic replacement
```

Placement rules:

- **No I/O.** `players_txt.rs` returns bytes and an atomic-replace *plan*; the caller writes.
- **Conventions are data.** Two rules that
  read like compiler behavior are conventions and live here, because a second consumer already
  depends on them: the **model source selection rule** (target-native first, then glTF, then the
  convertible opposite native format — the Player aesthetics editor uses it to decide which set
  opens visible and why leftover natives must be removed after a glTF conversion), and the
  **structural file-kind classification** of a model folder's files (model, texture, `GltfBuffer`,
  `PotentialGltfImage`, link file, marker, disallowed — from names alone; `model_convert`'s glTF
  dependency contract takes this classification as its input and only *promotes* candidates during
  deep URI resolution).
