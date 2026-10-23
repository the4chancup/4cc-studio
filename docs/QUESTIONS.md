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

- **Which DpFileList does VGL 27's PES 2019 install carry, and which CPK does its organizer
  compile into?** (worklog 0.0.1e, the rehearsal's finding 2.) On this machine's PES 2019, the
  4cc PES 2019 patch's 31-entry list (`4cc_40_faces`, `4cc_45_uniform`, midcups 60 to 69) has
  no `4cc_99_test`, the compiler's default `cpk_name`, so a compile with the defaults refuses
  to install (`dpfilelist_outdated`) and names `upgrade-dpfl`. That upgrade would stop the
  game loading five of the patch's CPKs the official list has no name for (`4cc_03_base`,
  `4cc_10_scoreboard`, `4cc_11_interface`, `4cc_12_cutscenes`, `4cc_13_competition`).
  Setting `cpk_name` to a listed slot (the rehearsal used `4cc_69_midcup`, the list's last)
  installs and draws in game. The answer decides what the bundle's readme tells VGL's
  organizer, and whether the preview should change a default or the finding's advice for
  such a list, before the cut (0.0.1f).

## Small confirmations
