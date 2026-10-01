# Archive-source fixtures (step 3.8d)

Written by the lead with Python's `zipfile` (fixed timestamps) and 7-Zip 7z.exe, from a script
kept in the session scratch (`.tmp/make_source_fixtures.py`, not part of the repository).

| File | Contents | For |
|---|---|---|
| `co - Spring.zip` | `players.txt` (`01 Keeper`), `notes.txt`, directory entries `Kits/` and `Kits/p2/`, deflate, no wrapper folder | TC-SRC-01: the same export as a folder (built by the test: git keeps no empty folder) and as a `.7z` |
| `co - Spring.7z` | the same files and folders, solid LZMA2 (`7z a -t7z -ms=on -m0=lzma2`; 7-Zip reports `Solid = +`) | TC-SRC-01, and the solid-7z memory charge (`libs/pipeline.md`) |
| `co - Case.zip` | `players.txt` and `Players.txt` | TC-SRC-07: two names that fold to one |
| `co - Escape.zip` | `players.txt` and `../x` | TC-SRC-07: a path escaping the root |
