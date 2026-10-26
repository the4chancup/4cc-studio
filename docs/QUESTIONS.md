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

## Small confirmations

- **May the shell read the monitors through `EnumDisplayMonitors` (an `unsafe` block)?**
  (Worklog 0.0.1q; the `DECISIONS.md` entry "eframe 0.37 gives the shell no monitor list";
  `plans/core/gui.md` "Window size classes".) The window reopens where it was closed, and a
  place on no monitor now (a second monitor unplugged) must fall back to the fit, which
  needs every monitor's rectangle. eframe hands the shell none, so the reader is the shell's
  own. The two ways: (a) a Windows-only function in `studio_core` over the `windows` crate
  the crate already depends on (the running-PES poll's pattern: a feature flag,
  `Win32_Graphics_Gdi`, `EnumDisplayMonitors` with a callback collecting each monitor's
  `GetMonitorInfoW` rectangle, about forty lines in two `unsafe` blocks with their `SAFETY`
  comments); Linux and macOS keep the empty list, their window managers keeping a title bar
  reachable themselves. (b) A crate such as `display-info` (a safe call, the same OS calls
  inside, the `windows` crate with more features on Windows, X11 and Core Graphics
  elsewhere), heavier for one list. Until the answer, the shell trusts the remembered place,
  so a member whose window was on an unplugged monitor gets it out of reach, as every program
  without the check does.

- **Which icon does each tool get?** (worklog 8.14.) Open `docs/prototypes/icons.html`,
  click one candidate per tool (the rail at the top shows the picks together), then copy the
  "Picks" box. The font build and the glyph constants wait on it. The Team compiler's is
  picked, (g); the other thirteen tools are open.
