# Open questions for the maintainer

The one list of what only the maintainer can answer: in-game behavior, cup practice, a
preference between options the plan does not pick. The lead adds an entry when it parks a
question (`AGENTS.md` "When the plan has gaps") and removes it when answered, logging the
answer where it belongs (a plan edit and a `DECISIONS.md` entry, or a worklog note). Lead and
sidekick work in progress stays in `WORKLOG.md` "Issues"; this file holds nothing the lead can
settle itself. Entries are grouped by what an answer unblocks, newest last within a group; each
names where the question came from so the context can be read there.

## In-game checks

### Face diffs are engine-specific
From: maintainer, 2026-10-03 (worklog Issues). A `face_diff.bin` (or its `face_diff.xml` and
`<dif>` text forms) authored for one engine misplaces the face on the other, as the kit layout
did; the compiler passes a diff through for whatever target it compiles. Deferred until after
Phase 4's kits work. To investigate then: what differs (format, bones moved, rest pose),
whether one engine's diff converts into the other's, and how a folder says which engine its
diff is for (the kit layout's `pre-fox`/`fox` markers are the model).
Investigated offline 2026-10-15 (`.tmp/face_diff/findings.md`; 18,622 diffs in the installed
games' CPKs). The format and values are the same on both engines: 1,006 of the 1,094 faces
present in both PES 17 and PES 18 have byte-identical diffs, and no header byte tells the
engines apart. The real split is the diff's version: 4 (32 bone entries, 944 bytes) against 5
(33, 960), which Konami ships in every game. Konami converted five faces 4 to 5 mechanically
(one entry inserted at index 31, the count and offset table updated). The "autoconverted"
community default used by 547 cup faces is that conversion with the count left at 32 and the
inserted entry 1.4 km off. So the misplacement you saw may come from a version 4 diff, from
that half-converted file, from eye-hiding values, or from the face model rather than the
engine. Example given 2026-10-17: a member's anime face, `lapwing`, whose pre-Fox diff he
edited for Fox (`.tmp/face_diff/lapwing/findings.md`). His fix is two head floats, 0x2C and
0x30, from 35 to 45 (eye parameters, by the census), and cup members' Fox diffs hold the same
hand-set pair (45, 45, -10, -10) more than any other value. In PES 21's Edit mode those floats
change nothing visible, even at 0 and 90: they act on moving eyes. What did the member see
wrong with the 35 diff on Fox, and where (Edit mode, the eyes during a match, a cutscene
close-up)? With that, a run there with 35 and 45 on PES 21 and PES 17 tells whether one diff
serves both engines or a folder needs one per engine.

## Cup practice and preferences

### Old content that never reached the game: a leftover, or unmigrated?
From: worklog 6.14's verify, the upgrade census rerun of 2026-10-19 (639 real inputs, 250
clean; per-class lines in `.tmp/6_15/census_614_classes.md`). Today a leftover (W, the run
stays clean) is content outside the old format's layout; anything inside its folders that no
step migrates is `unmigrated_content` (E, nothing written). The census's remaining Errors
include inside-the-layout content that Red packed but that no player ever saw in game:
- a file directly in `Faces/`, `Boots/` or `Gloves/`, where the layout holds folders only
  (`Faces/73606 - Civic cool.7z`, `Boots/k2755 - Maouchan.7z`, `Boots/boots.fmdl`,
  `Faces/face.xml`; 8 inputs): Red's walks list subfolders only and never read them;
- a kit texture of a `Full` export for a slot with no config and no Note entry (ASP's
  `u0705p2.dds`, UMAJP's `u0XXXp5.dds`, tf2g's p1, p2, g1; 3 inputs), and a copy such as
  `u0XXXp2 - Copy.dds` (2 inputs): packed, but no kit row named them;
- a portrait Red misnamed into a file the game never loads (`player_73906-.dds` became
  `player_7396-.dds` in 6 /o/ exports; BAG's `Vanilla vitamins.png`);
- the second colour of a team-colour entry holding two (WOWG's `- 1st: #0070DD - #FFF468`;
  Red kept the first);
- an item folder no save record names (VG Summer 20's misspelled second set `k2501`-`k2523`
  beside its `k1501`-`k1523`).
Should "never reached a player in game" make such content a leftover too (dropped with a
Warning naming it, the run clean), or should each stay an Error the member fixes in the old
export first? Waiting: worklog 6.14 (its verify).

### A Midcup kit that updates its textures only
From: the same census. Three Midcup/Additions exports (JP 2024 Spring Additions for Day 5, JP
2024 Winter Garbage Day Additions, VG Day 0 Midcup) carry `Kit Textures/` and no `Kit
Configs/`: they update kit textures while the cup's installed configs stay. Red did exactly
that. A Studio kit folder without `config.toml` compiles with the template config, in a
`Midcup` too (`kit_config_generated`), so migrating them as they are would change those kits'
look. Should a `Midcup` kit folder without `config.toml` keep the slot's installed config (as
the slots a Midcup does not resend already do), the template applying only where nothing is
installed? Or should a Midcup kit always carry its config? Until then those exports stay
unmigrated. Waiting: worklog 6.14.

### A second goalkeeper kit
From: the same census. Four inputs carry second and later GK kits (`u0XXXg2`+ textures,
`XXX_DEF_GK2nd_realUni.bin`, a Note's `- 2nd GK` line: VP Test, CIV4XG twice, 3 SBCF26 with
g2 to g8). Studio has the game's ten slots `p1`-`p9` and `g1`, and the upgrader reports the
rest as unmigrated content. Does the game use a team's second GK kit (Red wrote it to
`UniColor.bin` as kit 0x11), and should Studio gain a `g2` slot? Waiting: worklog 6.14.

## Small confirmations
