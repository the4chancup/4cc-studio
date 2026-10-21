# Open questions for the maintainer

The one list of what only the maintainer can answer: in-game behavior, cup practice, a
preference between options the plan does not pick. The lead adds an entry when it parks a
question (`AGENTS.md` "When the plan has gaps") and removes it when answered, logging the
answer where it belongs (a plan edit and a `DECISIONS.md` entry, or a worklog note). Lead and
sidekick work in progress stays in `WORKLOG.md` "Issues"; this file holds nothing the lead can
settle itself. Entries are grouped by what an answer unblocks, newest last within a group; each
names where the question came from so the context can be read there.

## In-game checks

## Cup practice

### DBC7: the stat total

From worklog 20.0e (`plans/aatf_rules.md` "DBC7"). The organizer's answer of 2026-10-21 keeps
DBC7-Editor's warning band (a regular's stat up to 7 above 77, a silver's up to 5 above 88) and
adds an error when "the combined stat total of an entire NM/Silver player" exceeds "the combined
stat total of an all 77/88". Two readings differ on real players, and `dbc7.rhai` takes the
first of each until the organizer says otherwise:

1. **Which total is the cap.** The sum of the player's own targets, so a Manlet regular at 175
   who takes the +5 bonus is held to an all-82 player's total (as the Manlet bonus lets him
   reach 82 on every stat); or a flat all-77 (all-88) total for every regular (silver), which
   would make a Manlet regular's bonus unusable beyond trading points.
2. **Which stats are added.** Every ability stat the game has (on PES 2017, the 23 the editor
   checks), so an outfielder's low goalkeeping stats count, and leave room to raise his other
   stats by more in total; or only the stats that matter for his role (which ones would then
   need a list).

### 4ccEditor's team trim (Ctrl+Shift+T)

From `plans/save_editor.md` "Feature inventory", the 2026-10-21 source audit. 4ccEditor has a
hidden shortcut on the team selector that empties a team's roster slots past 23 (PES 19 to 21
rosters hold 40). Do cups still use it (a team brought down to a 23-man squad), so the Save
editor should keep it as a team operation, or can it go?

### Autumn 26: six points for the 4cc organizer

From worklog 20.1 (`spec/aatf_rules.md` "The official ruleset", DECISIONS 2026-10-21 "Autumn 26
follows the wiki's written rules…"). `autumn26.rhai` follows the wiki's Appendix:Rules on the
first four and the tools on the last two, where every tool agrees; the organizer's page `.tmp/20_au26/autumn26_rule_choices.html` (one pick per point and a line to
post back) sets each beside the two 4ccEditor editions:

1. May a gold trade both of his free COM styles for A positions, or only one?
2. Which Red CBs may be 189 cm: any non-medal registered at CB (the wiki), only one whose sole A
   is CB (the official 4ccEditor), or only one fielded solely at CB (the earlier community
   version)?
3. May a Red CB who qualifies stay at 185 cm (the wiki's "can go up to 189"), or must he be 189
   (both 4ccEditor editions)?
4. A Green silver at 194 cm, legal with every stat 3 lower (the wiki), or never (the official
   4ccEditor)?
5. Heights 176 to 179: counted as 180 (every tool), or illegal (the wiki's "exactly 180")?
6. A gold or a goalkeeper at 194 cm: a warning, the team still passing (every tool), or a
   failure (the wiki's "cannot be this height")?

## Small confirmations
