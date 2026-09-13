# archives fixtures

One small export-like tree, `sample/`, and the same tree packed six ways by the tools members
use (`.tmp/archives_fixtures.py` in the writing session). The three binary files in the tree are
Konami-derived PES files from a real referee export (a 176-byte `face.dds`, a 476-byte
`materials.mtl`, a 960-byte `face_diff.bin`), kept for interoperability (see the repository
README); the text files are ours. The container is what is under test, not the contents.

| File | Made by | What it exercises |
|---|---|---|
| `sample/` | copied | the expected entry list and bytes: `Note.txt`, `Faces/Player One/face.dds`, `Faces/Player One/materials.mtl`, `Faces/Player Two/face_diff.bin`, `Faces/Player Two/empty.txt` (0 bytes), `Kits/Réf.txt` (non-ASCII name), plus an empty folder `Empty/` that git does not store |
| `sample.7z` | 7-Zip 24, `-t7z -mx=5` | LZMA2, one solid block, directory entries, UTF-16 names |
| `sample_7zip.zip` | 7-Zip, `-tzip -mx=5` | Deflate; names in the OEM code page without the UTF-8 flag (`é` stored as `0x82`) |
| `sample_store.zip` | 7-Zip, `-tzip -mx=0` | stored entries |
| `sample_ps.zip` | PowerShell `Compress-Archive` | Deflate with the UTF-8 name flag; no entry for the empty folder's parent |
| `encrypted.7z` | 7-Zip, `-pfoo -mhe=on` | header encryption: fails at open |
| `encrypted.zip` | 7-Zip, `-pfoo` | entry encryption: lists, fails at read |

The archives all contain the tree under a top-level `Sample Export/` folder, as a member's
archive would.
