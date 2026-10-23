"""The Ruleset editor's summary goldens: the draft's own summary() run on each embedded ruleset.

The draft (`docs/prototypes/ruleset_editor.html`) is where the summary's wording was agreed, so
its function is the independent source of the expected text; the Rust port must equal it. Each
ruleset is fed to it as `libs/aatf`'s writer writes it (`ruleset_summary_dump.rs`), not as the
hand-written file: the writer lists every map in the Rust type's order (a tier's stats in stat
order, its free allowances by group id), which is the order the port iterates them in.

Usage:
  1. Build `ruleset_summary_dump.rs` as a throwaway crate (its header says how) and run it:
     it writes `written_<name>.rhai` for the nine embedded rulesets into a scratch folder.
  2. python scripts/provenance/fixtures/ruleset_summary_goldens.py <scratch folder>
     Needs node. Writes crates/tools/ruleset_editor/tests/fixtures/summary/<name>.md.
"""
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[3]
HTML = ROOT / "docs" / "prototypes" / "ruleset_editor.html"
OUT = ROOT / "crates" / "tools" / "ruleset_editor" / "tests" / "fixtures" / "summary"
NAMES = ["autumn25", "autumn26", "dbc7", "spring26", "vgl25", "vgl26", "vgl27", "vtl10", "vtl11"]

lines = HTML.read_text(encoding="utf-8").splitlines()


def block(first: str, last: str) -> str:
    """The lines from the first starting with `first` to the next starting with `last`."""
    start = next(i for i, line in enumerate(lines) if line.startswith(first))
    end = next(i for i in range(start, len(lines)) if lines[i].startswith(last))
    return "\n".join(lines[start : end + 1])


def before(text: str, marker: str) -> str:
    return text[: text.rindex(marker)]


# The draft's constants and rulesets (the summary reads its label tables), its ruleset
# builders, clone and human, and the "Readable rules" block that holds summary().
constants = before(block("const STATS = [", "const CLOUDS = {"), "const CLOUDS = {")
builders = before(block("function tier(", "function blank()"), "function blank()")
clone = block("function clone(v)", "}")
human = block("function human(id)", "}")
readable = block("// ---- Readable rules", "// ---- Layout ----")
assert "function summary(r)" in readable, "the draft's summary() moved"

DRIVER = r"""
const fs = require("fs");
const [blocks, out] = [process.argv[2], process.argv[3]];
let R;
// A written block's RULESET map read as JS (Rhai's `#{` is JS's `{`), with the schema's serde
// defaults filled in: the writer leaves a field out at its default, the draft's own
// rulesets spell every one.
function load(name) {
  const text = fs.readFileSync(blocks + "/written_" + name + ".rhai", "utf8");
  const start = text.indexOf("const RULESET = #{");
  const end = text.indexOf("\n};", start);
  const r = (0, eval)("(" + text.slice(start + "const RULESET = ".length, end + 2).replace(/#\{/g, "{") + ")");
  r.stats.at_most = r.stats.at_most || [];
  r.card_groups = r.card_groups || {};
  r.medal_systems = r.medal_systems || [];
  r.heights = r.heights || { brackets: [], systems: [] };
  r.heights.systems.forEach(s => { s.quotas = s.quotas || {}; });
  r.specials = r.specials || [];
  r.specials.forEach(s => { s.severity = s.severity || "error"; });
  r.team_settings = r.team_settings || [];
  r.team_settings.forEach(t => { t.forbidden_shapes = t.forbidden_shapes || []; t.versions = t.versions || []; });
  r.suggestions = r.suggestions || [];
  r.universal.fielded_at_a = r.universal.fielded_at_a || [];
  r.tiers.forEach(t => {
    t.count = t.count === undefined ? null : t.count;
    t.free = t.free || {};
    t.stats = t.stats || {};
  });
  return r;
}
for (const name of process.argv.slice(4)) {
  R = load(name);
  fs.writeFileSync(out + "/" + name + ".md", summary(R) + "\n");
}
"""


def main() -> int:
    blocks = pathlib.Path(sys.argv[1]).resolve()
    for name in NAMES:
        assert (blocks / f"written_{name}.rhai").is_file(), f"missing written_{name}.rhai"
    OUT.mkdir(parents=True, exist_ok=True)
    js = "\n".join([constants, builders, clone, human, readable, DRIVER])
    js_path = blocks / "ruleset_summary_goldens.js"
    js_path.write_text(js, encoding="utf-8")
    result = subprocess.run(["node", str(js_path), str(blocks), str(OUT), *NAMES])
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
