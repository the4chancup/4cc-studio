# Placeholder portraits

The game's own portrait for a player who has none, which the Save editor's Tactics pitch draws
on a starter's marker whenever it cannot show his portrait (`plans/save_editor.md` "Tactics
editor", the pitch's look). They are embedded in the Studio binary, so the pitch draws them
with no PES folder set or reachable.

Each is `common/render/symbol/blank/player-dummy.png` copied byte for byte from a base data
CPK of the maintainer's installs (2026-10-23), a 128 x 128 8-bit paletted PNG. The game
changed it twice; the versions in between carry the same bytes.

| File | Used for | Source | SHA-256 (first 12) |
|---|---|---|---|
| `player-dummy_15.png` | PES 2015 | PES 2015's `dt15.cpk`: a white silhouette on a light blue square | `d4b0dcb33e33` |
| `player-dummy_16.png` | PES 2016-2018 | PES 2016's and 2017's `dt15_win.cpk`, PES 2018's `dt15_x64.cpk`: a blue-grey face, transparent around it | `db83a69d69f0` |
| `player-dummy_19.png` | PES 2019-2021 | PES 2019's, 2020's and 2021's `dt15_x64.cpk`: a beige face, transparent around it | `1b36c23e86d7` |
