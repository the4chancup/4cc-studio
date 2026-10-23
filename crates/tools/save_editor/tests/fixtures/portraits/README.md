# Portrait fixtures

Real portraits as the games' `download/` CPKs hold them, for the Tactics pitch's portrait
reading (`plans/save_editor.md` "Tactics editor", "Portraits"). Lead-written: do not edit.

| File | Source | Bytes | SHA-256 (first 16) |
|---|---|---|---|
| `75823_faces.dds` | PES 2021 `download/4cc_40_faces.cpk`, `common/render/symbol/player/75823.dds` | 22000 | `bd0edd2636d2065a` |
| `75823_midcup66.dds` | PES 2021 `download/4cc_66_midcup.cpk`, the same path: the portrait a later listed CPK overrides the faces CPK's with | 22000 | `d78bf77891f45957` |
| `player_71801_pes17.dds` | PES 2017 `download/4cc_40_faces.cpk`, `common/render/symbol/player/player_71801.dds`: a WESYS-wrapped DDS | 7902 | `41c4fa772d37ecfc` |

Both 75823 portraits are 128 x 128; their pixel at (64, 64) is `(148, 219, 247, 255)` in the
faces CPK's and `(255, 255, 255, 255)` in the midcup's (12,641 of 16,384 pixels differ).
