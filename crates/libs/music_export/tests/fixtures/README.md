# music_export fixtures

Cup music exports, copied as found on the writing machine (CRLF line endings kept:
`.gitattributes` stores fixtures as is). The grammar is in `docs/plans/music_player.md`
"The `.4ccm` format".

| File | Source | Notes |
|---|---|---|
| `98hu - Set A.4ccm` | `Teams_Main/98HU/Music/98HU Music Export for VRL5/` | hand-written layout with its own comments; `match` lists on anthems; `special` victory anthems without labels; `end loop`; player names starting with a `[CF]` position tag (the slot field is never tokenized); `pause restart`; a comment-only chants section |
| `o music summer.4ccm` | `Teams_Main/O/Music/` | `goals = 1` (the single `=`); `start 00:22.617` with millisecond decimals; a `[COOL VIBRATIONS]` player name |
| `jp.4ccm` | `Teams_Main/JP/Music/JP MUSIC EXPORT SUMMER 2024/` | the RigDJ-written layout; `opponent` on anthems and goalhorns; a player name with commas (line breaks on the button) and slashes |
| `vg.4ccm` | `Lab/Temp/vg summer 26 audio export/` | `sync;yes`; `comeback`, `first`, `not first`, `advance`, `randomise` sets, `end loop` with `start` |
| `fgog music.4ccm` | `Teams_Main/FGOG/Music/` | `start 0:39.6` (one decimal); `every 3`; `comeback;once` |
| `v.4ccm` | `Music/V/` | `lead > 3` on the default goalhorn; `match` with `consolation` |
| `every_keyword.4ccm` | hand-written | every condition and instruction the cup exports above do not use (`teamgoals`, `lead`, `mostgoals`, `special` with a spaced label, `time`, `home`, `speed`, `louder`, `warcry`, `unrandom`, `pause ... every`, `event` clips), bracketed and escaped tokens, a slot-only line, both flags opted out; LF line endings |
