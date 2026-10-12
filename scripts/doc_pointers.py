"""Section-pointer check: `doc_pointers.py` (in `just gates`).

A section pointer names a Markdown file and one or more places in it: `` `path.md` "Heading" ``,
with or without the backticks, further titles following after `, `, ` and ` or ` or `
(`` `object_model.md` "Core types", "Player folders" ``). Every pointer in `AGENTS.md`, the
documents under `docs/` and the `.rs` and `.md` files under `crates/` must resolve: its file
exists and holds an anchor the title names. `DECISIONS.md` and `LOG.md` are not checked: they
are append-only, so their old pointers stay as they were written.

The documents name a place by a heading, a bold run-in title (`**Subfolders.**`, anywhere in a
line), the start of a paragraph or list item ("Rules:"), or a table row's first cell (a
glossary term), so each of these is an anchor, with its list marker, bold markers and
backticks removed, and a heading also without its number ("5. Writer" is also "Writer"). A
title names an anchor equal to it or starting with it and then a space or `:`, `,`, `.`, so
"Testing" names "Testing and verification". The check is for a pointer whose text is gone
from its file (a section moved to the other tree, renamed or deleted); leniency on the exact
wording costs it nothing there.

A path resolves against, in order, the pointing file's own directory, `docs/`, the repository
root and `docs/plans/`; the first that names an existing file is the pointer's file. The last
base is how the worklog and the other working documents name a plan (`aatf_rules.md`); a
section of the spec is named with its tree (`spec/aatf_rules.md`), so a pointer never leaves
the reader to guess which tree it means. A path that resolves nowhere is a problem only when
the pointer is in `docs/` or `AGENTS.md`, or names `docs/`, `plans/` or `spec/`:
`` `README.md` "Usage" `` in a crate is about that crate.
"""

import re
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

TITLE = r'"[^"\n]{1,120}"'
POINTER = re.compile(
    rf"(?<![\w/.])`?(?P<path>[\w./-]+\.md)`?[,;]? (?P<titles>{TITLE}(?:,? (?:and |or )?{TITLE})*)"
)
QUOTED = re.compile(TITLE)
HEADING = re.compile(r"^#+\s+(?P<title>.+?)\s*#*\s*$")
NUMBER = re.compile(r"^\d+\.\s+")
LIST_MARKER = re.compile(r"^\s*(?:[-*]|\d+\.)\s+")
BOLD = re.compile(r"\*\*(?P<title>[^*]+?)\*\*")
SKIPPED = {"docs/DECISIONS.md", "docs/LOG.md"}


@dataclass(frozen=True)
class Pointer:
    source: str
    line: int
    path: str
    title: str


def pointers(source: str, text: str) -> list[Pointer]:
    found = []
    for number, line in enumerate(text.splitlines(), start=1):
        for match in POINTER.finditer(line):
            for quoted in QUOTED.findall(match["titles"]):
                found.append(Pointer(source, number, match["path"], quoted[1:-1]))
    return found


def clean(text: str) -> str:
    return text.replace("`", "").replace("**", "").strip()


def anchors(text: str) -> set[str]:
    found = set()
    in_fence = False
    previous = ""
    for line in text.splitlines():
        if line.startswith("```"):
            in_fence = not in_fence
        elif not in_fence:
            heading = HEADING.match(line)
            if heading:
                title = clean(heading["title"])
                found |= {title, NUMBER.sub("", title)}
            elif line.startswith("|"):
                found.add(clean(line.split("|")[1]))
            elif LIST_MARKER.match(line) or not previous.strip() or HEADING.match(previous):
                found.add(clean(LIST_MARKER.sub("", line)))
            found |= {clean(bold["title"]) for bold in BOLD.finditer(line)}
        previous = line
    return found


def names_anchor(found: set[str], title: str) -> bool:
    title = clean(title)
    return title in found or any(
        anchor.startswith((title + " ", title + ":", title + ",", title + ".")) for anchor in found
    )


def resolve(root: Path, source: str, path: str) -> Path | None:
    bases = [(root / source).parent, root / "docs", root, root / "docs" / "plans"]
    for base in bases:
        candidate = (base / path).resolve()
        if candidate.is_file():
            return candidate
    return None


def problems(root: Path, found: list[Pointer]) -> list[str]:
    cache: dict[Path, set[str]] = {}
    out = []
    for pointer in found:
        target = resolve(root, pointer.source, pointer.path)
        where = f"{pointer.source}:{pointer.line}"
        if target is None:
            in_docs = pointer.source.startswith("docs/") or pointer.source == "AGENTS.md"
            if in_docs or pointer.path.startswith(("docs/", "plans/", "spec/")):
                out.append(f"{where}: `{pointer.path}` names no file")
            continue
        if target not in cache:
            cache[target] = anchors(target.read_text(encoding="utf-8"))
        if not names_anchor(cache[target], pointer.title):
            shown = target.relative_to(root).as_posix()
            out.append(f'{where}: {shown} has no "{pointer.title}"')
    return out


def sources(root: Path) -> list[Path]:
    files = [root / "AGENTS.md"]
    files += sorted((root / "docs").rglob("*.md"))
    files += sorted((root / "crates").rglob("*.rs"))
    files += sorted((root / "crates").rglob("*.md"))
    return [f for f in files if f.relative_to(root).as_posix() not in SKIPPED and "target" not in f.parts]


def main() -> int:
    found = []
    for file in sources(ROOT):
        found += pointers(file.relative_to(ROOT).as_posix(), file.read_text(encoding="utf-8"))
    broken = problems(ROOT, found)
    for line in broken:
        print(line)
    print(f"{len(found)} section pointers, {len(broken)} broken")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main())
