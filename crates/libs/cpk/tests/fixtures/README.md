# cpk fixtures

Konami-derived files are Konami's, kept for interoperability (see the repository README). Sizes
are the smallest archives on hand that exercise each writer variant; one per variant.

| File | Source | Notes |
|---|---|---|
| `konami_pes17_dt42_win.cpk` | PES 2017 `Data/dt42_win.cpk` | Konami's packer (`Tvers` "CPKMC2.21.10, DLL3.00.00"), 35 header columns, TOC at 0x800, `ContentOffset` 0x1000, no ETOC, 5 uncompressed entries (WESYS-wrapped `.bin` files) |
| `konami_pes21_dt42_all.cpk` | PES 2021 `Data/dt42_all.cpk` | Konami's packer ("CPKMC2.49.32, DLL3.24.00"), 44 header columns, otherwise the same shape, 5 entries |
| `red_pes16_face_73113.cpk` | 4cc aet compiler Red 4.1.0 output (`patches_contents/Singlecpk/.../face/real/73113.cpk`, Winter 26) | pes-file-tools' `CpkWriter` (the writer whose bytes Studio reproduces): `Tvers` "pes-file-tools", ETOC with modification times, 3 entries, 2048 alignment |
| `cpkmc136_placeholder.cpk` | 4cc PES17 DLC `4cc_02_misc.cpk` | CRI Packed File Maker 1.36 ("CPKMC1.36.00, DLL1.36.00"), 24 header columns, ETOC present, one entry with an empty `DirName` (`/placeholder`) |
| `crilayla/settings_json.crilayla` | PES 2021 `Data/dt70_x64.cpk`, entry `movie/fade/settings.json` as stored | 548 bytes CRILAYLA (header, bitstream, 0x100-byte raw prefix) |
| `crilayla/settings_json.bin` | the same entry decompressed (Blue's `crilayla.decompressCrilayla`) | 1614 bytes; genuinely begins with 256 spaces, then CRLF and `{` |
| `crilayla/symbol_816_dds.crilayla` | PES 2021 `Data/dt14_all.cpk`, entry `common/render/symbol/player/816.dds` as stored | 7364 bytes CRILAYLA |
| `crilayla/symbol_816_dds.bin` | the same entry decompressed | 16512 bytes, a DDS file |
