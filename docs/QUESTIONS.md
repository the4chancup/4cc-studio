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

### DBC7: the organizer's editor errs on what the page leaves free

From worklog 20.0d (`plans/aatf_rules.md` "DBC7"). The DBC7 wiki page sets heights and medals
and says "No tactical export rules for anything else … go crazy", but the organizer's
DBC7-Editor (`aatf.cpp` at `ee73917`) also reports as errors, "You ruined Christmas": a
registered position not rated A, an age outside 15 to 50, a weight outside the official range
(height − 129 floored at 30, to height − 81), no captain, no registered goalkeeper, and a 189 cm
goalkeeper with a second A position. It also lets a stat sit above its tier's target with only
a warning, up to 7 above for a regular and 5 for a silver or a Manlet regular, where the page's
"Non-Medals are 77 rated" reads as a cap. `dbc7.rhai` follows the page on all of these (DECISIONS
2026-10-20). The band is what decides most: on the maintainer's two PES 2017 saves the fork
passes 29 and 36 real teams (Autumn 24 qualifier teams among them, regulars at 82, silvers at
92), and `dbc7.rhai` fails every one of them on those stats alone, nothing else (worklog 20.0d's
parity run). Does the organizer go by the editor's verdict on any of them? An answer turns the
matching checks on in the data block alone; the warning band would need a schema field of its
own, since a stat's violation has no severity setting.

## Small confirmations
