# wezlib fixtures

Game-derived data, Konami's, kept for interoperability (see the repository README).

| File | Source | Notes |
|---|---|---|
| `RefereeColor.bin.wesys` | PES 2017 `Data/dt00_win.cpk`, entry `common/character0/model/character/uniform/team/RefereeColor.bin` | 133 bytes as stored in the CPK: 16-byte WESYS header + zlib stream |
| `RefereeColor.bin` | the same entry, decompressed (Blue's `zlib_plus.decompress`) | 255 bytes, the expected payload |
