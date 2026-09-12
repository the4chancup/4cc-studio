# 4cc Studio

One Rust suite for every tool the 4chan Cup community uses to run its PES (2015-2021) cups: a
Team compiler (aesthetics exports to CPK archives), a save editor, and the smaller tools around
them (kit configs, referees, balls, music, match tracking, model conversion), in a single binary
that is both a GUI and a CLI.

**Status: planning.** Nothing compiles yet. The architectural plan lives in `docs/plans/`
(start at `docs/plans/README.md`). `AGENTS.md` is the short orientation for anyone working on
the code, human or model; `docs/CONTRIBUTING.md` holds the coding rules and verification gates;
`docs/WORKLOG.md` says where the work currently stands.

## License

Licensed under either of

- Apache License, Version 2.0 (`LICENSE-APACHE` or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (`LICENSE-MIT` or <http://opensource.org/licenses/MIT>)

at your option. Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any additional terms or
conditions.

**Not covered by these licenses:** data extracted from the Pro Evolution Soccer games and kept in
this repository or embedded in the binary for interoperability: the player skeletons under
`resources/skeletons/`, `DpFileList.bin` and the other `.bin` templates, the kit-icon reference
sheet, and similar game-derived files. Those remain the property of Konami Digital Entertainment.
They are included so the tools can produce files the games read, and nothing here grants any
right to them beyond that use.
