# ftex fixtures

Konami-derived files are Konami's, kept for interoperability (see the repository README). Each
`.dds` is the matching `.ftex` converted by pes-file-tools' `ftexToDdsBuffer` (Blue's
`utils/ftex.py`), the reference output. Sources are PES 2021 extracted CPK trees on the
maintainer's machine (`E:\PES2021\Data\<cpk>_files`).

| File | Source | Notes |
|---|---|---|
| `konami_bc7_nb00b.ftex` | `dt32_g4/Asset/model/character/uniform/edit/font/#windx11/nb00b.ftex` | format 11 (BC7), 1024x128, 9 mips, texture type 0x1, version 2.040, flags 0x111, first frame in 2 chunks, 1056 bytes. Its DDS is 174980 bytes and is NOT committed: the test checks length and SHA-256 `e1853fe7888013ce91b659b2c34d45dc28fef8880386df3a2f89a71af22fcdeb` |
| `konami_bc1_bibs_metalness.ftex` / `.dds` | `dt32_g4/.../bibs_reserve/sourceimages/#windx11/bibs_metalness.ftex` | format 2 (BC1), 16x16, 3 mips, type 0x1, version 2.030, flags 0x11, 304 bytes; DDS 296 bytes |
| `konami_bc3_dummy_nrm.ftex` / `.dds` | `dt32_g4/.../bibs/sourceimages/#windx11/dummy_nrm.ftex` | format 4 (BC3), 128x128, 6 mips, type 0x9 (normal map), version 2.040, flags 0x111, 448 bytes; DDS 21968 bytes |
| `konami_rgba32_gr_dither_nrt.ftex` / `.dds` | `dt00_x64/Asset/effect/gr_pic/#windx11/gr_dither_nrt_32b.ftex` | format 0 (A8R8G8B8, uncompressed), 8x8, 2 mips, type 0x1, version 2.030, flags 0x11, 416 bytes; DDS 448 bytes (no DX10 header) |
| `konami_bc1_cubemap_default_reflection.ftex` / `.dds` | `dt00_x64/Fox/Asset/#windx11/default_reflection.ftex` | format 2 (BC1) cube map (type 0x7), 16x16, 3 mips × 6 faces, version 2.040, 832 bytes; DDS 1136 bytes |
| `konami_bc3_1x1_cup_logo.ftex` / `.dds` | `dt00_x64/Asset/model/character/common/sourceimages/#windx11/cup_logo.ftex` | format 4 (BC3), 1x1, 1 mip, type 0x3 (sRGB), version 2.040, 89 bytes (the frame is one padded block); DDS 144 bytes |
| `handbuilt_raw_frames.ftex` | `konami_bc1_bibs_metalness.dds`, by `scripts/provenance/fixtures/ftex_fixtures.py` | every frame stored raw (chunk count 0, compressed size 0), the storage Konami uses for 13 frames of the 10370 PES 2021 FTEX files (`Fox/Asset/#windx11/ggx_lookup_rgba.ftex`, `face/montage/#windx11/mark.ftex`, both 87 KB); header from pes-file-tools' writer. pes-file-tools' `ftexToDdsBuffer` converts it back to the DDS byte for byte |
| `handbuilt_single_zlib.ftex` | same script | every frame one zlib stream (chunk count 0, compressed size > 0): a storage the reference reader handles and no PES 2021 file uses. Converts back to the DDS byte for byte |
| `pft_volume_rgba16f.ftex` / `.dds` | same script | a 4x4x4 R16G16B16A16_FLOAT volume DDS (three mips) through pes-file-tools' `ddsToFtexBuffer(dds, "LINEAR")`, then `ftexToDdsBuffer` back: the `.dds` is the reference's volume header (depth flag, volume caps, DX10 dimension 4). The only Konami volume texture is `light/#windx11/lut_Match.ftex` (33x33x33, 181 KB) |
| `pft_from_dummy_nrm_dds.ftex` | `konami_bc3_dummy_nrm.dds` converted by pes-file-tools' `ddsToFtexBuffer(dds, "NORMAL")` | 432 bytes: the writer's header layout (version 2.03 for BC1–BC3, nrt 0x02, flags 0x11, zero hashes), zlib level 3 chunks of 16 KiB. Converts back to the DDS losslessly. Studio's own writer matches its header and mip table field by field; the compressed chunk bytes differ (different deflate implementation), so parity is checked by converting back |
