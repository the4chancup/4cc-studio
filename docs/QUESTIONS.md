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

## Small confirmations
