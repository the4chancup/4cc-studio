# `DpFileList.bin` fixture

Lead-written fixture for the working-bin walk (step 4.21, TC-BIN-05, 08, 09, 13, 21), generated
by `scripts/provenance/fixtures/dpfl_fixtures.py`. Do not edit it by hand; change the script and
rerun it. The layout it follows is in `docs/spec/team_compiler/pipeline.md` "DpFileList
upgrade"; the reader's own fixtures are `cpk`'s (`crates/libs/cpk/tests/fixtures/dpfl/`).

| File | What it is |
|---|---|
| `walk.bin` | the list a test installs as `download/DpFileList.bin`: `4cc_08_bins.cpk`, `4cc_61_midcup.cpk`, `4cc_99_test.cpk`, in that load order, built in the layout of the maintainer's PES 2021 list (header word 0, 48-byte NUL-padded records), with that list's 1204-byte zero tail; 1,364 bytes |
