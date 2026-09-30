# Fallback bins

The bases the Team compiler builds a bin on when no installed CPK supplies one (a from-scratch
compile; Team compiler plan, `pipeline.md` "Bins accumulation"). They are embedded in the Studio
binary.

| File | Used for | Source |
|---|---|---|
| `UniformParameter18.bin` | PES 18 | Red's `Engines/bins/`, commit `c94e673` (2024-05-17, "Update fallback bin files") |
| `UniformParameter19.bin` | PES 19-21 | same |

The per-version choice is Red's (19 and later build on the 19 base), so a from-scratch compile's
`UniformParameter.bin` is comparable with Red's output. `TeamColor.bin` and `UniColor.bin` join
this folder with the phase that writes them (Phase 4).
