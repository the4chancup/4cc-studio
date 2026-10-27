"""Tests of scripts/release.py; `just release` runs them before the recipe."""

import re
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path

from release import (
    BUNDLED_RULES,
    CUSTOM_CHECKS_STUB,
    assemble_rules,
    existing_glob,
    expected_entries,
    exe_name,
    find_7z,
    launcher_calls,
    listing_paths,
    readme_names_version,
    sha256_line,
    symbols_entries,
    workspace_version,
)

ROOT = Path(__file__).resolve().parent.parent

HELP = """Unlimited Tool Works: the 4cc community's PES tools

Usage: UnlimitedToolWorks [OPTIONS] [COMMAND]

Commands:
  help  Print a help chapter, or export them all as Markdown
"""


class Pure(unittest.TestCase):
    def test_workspace_version(self) -> None:
        text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        declared = re.search(r'\[workspace\.package\][^[]*?\nversion = "([^"]+)"', text)
        self.assertIsNotNone(declared)
        self.assertEqual(workspace_version(text), declared.group(1))
        self.assertEqual(workspace_version('[workspace.package]\nversion = "9.9.9"\n'), "9.9.9")

    def test_exe_name(self) -> None:
        self.assertEqual(exe_name(HELP), "UnlimitedToolWorks")
        self.assertIsNone(exe_name("no usage line\n"))

    def test_launcher_calls(self) -> None:
        for bat in ("quick_compile.bat", "quick_upgrade.bat"):
            text = (ROOT / "resources/bundle" / bat).read_text(encoding="utf-8")
            self.assertTrue(launcher_calls(text, "UnlimitedToolWorks"), bat)
            self.assertFalse(launcher_calls(text, "4cc-studio"), bat)

    def test_readme_names_version(self) -> None:
        # The shipped readme against the workspace's version, which every release bumps with it.
        text = (ROOT / "resources/bundle/README.txt").read_text(encoding="utf-8")
        declared = workspace_version((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertTrue(readme_names_version(text, declared))
        self.assertTrue(readme_names_version("Unlimited Tool Works 0.0.1\n===\n", "0.0.1"))
        self.assertFalse(readme_names_version("Unlimited Tool Works 0.0.1\n===\n", "0.0.2"))
        # The program's name is not the script's to check: any name before the version.
        self.assertTrue(readme_names_version("4cc Studio 1.0.0\n", "1.0.0"))
        self.assertFalse(readme_names_version("4cc Studio 1.0.0.1\n", "1.0.0"))

    def test_assemble_rules(self) -> None:
        self.assertEqual(
            assemble_rules(
                'const RULESET = #{ name: "T" };\n\n\n',
                "// ==== LOGIC (ruleset-logic 1): the Ruleset editor replaces this section ====\n"
                "fn check_team(team) { [] }\n\n",
            ),
            'const RULESET = #{ name: "T" };\n'
            "\n"
            "// ==== LOGIC (ruleset-logic 1): the Ruleset editor replaces this section ====\n"
            "fn check_team(team) { [] }\n"
            "\n"
            "// ==== CUSTOM CHECKS: optional, hand-written; the Ruleset editor keeps this section as it is ====\n"
            "fn custom_checks(team) { [] }\n",
        )

    def test_assemble_rules_keeps_a_blocks_own_custom_checks(self) -> None:
        # As `aatf`'s `assemble`: the block's section goes after the logic, and no stub
        # beside it, which would define `custom_checks` twice.
        self.assertEqual(
            assemble_rules(
                "const RULESET = #{};\n\n"
                "// ==== CUSTOM CHECKS: the cup's own ====\n"
                "fn custom_checks(team) { [1] }\n",
                "// ==== LOGIC (ruleset-logic 1): x ====\nfn check_team(team) { [] }\n",
            ),
            "const RULESET = #{};\n"
            "\n"
            "// ==== LOGIC (ruleset-logic 1): x ====\n"
            "fn check_team(team) { [] }\n"
            "\n"
            "// ==== CUSTOM CHECKS: the cup's own ====\n"
            "fn custom_checks(team) { [1] }\n",
        )

    def test_the_stub_is_aatfs(self) -> None:
        source = (ROOT / "crates/libs/aatf/src/file.rs").read_text(encoding="utf-8")
        banner = next(
            line
            for line in source.splitlines()
            if line.startswith('pub(crate) const CUSTOM_CHECKS: &str = "')
        )
        self.assertEqual(CUSTOM_CHECKS_STUB.split("\n")[0], banner.split('"')[1])
        self.assertIn("fn custom_checks(team) {{ [] }}", source)

    def test_the_bundled_settings_name_the_rules_file(self) -> None:
        settings = tomllib.loads(
            (ROOT / "resources/bundle/data/settings.toml").read_text(encoding="utf-8")
        )
        self.assertEqual(settings["save-editor"]["aatf_rules"], BUNDLED_RULES)
        self.assertIs(settings["team-compiler"]["multicpk_mode"], False)
        self.assertEqual(settings["team-compiler"]["cpk_name"], "4cc_99_test")

    def test_expected_entries(self) -> None:
        self.assertEqual(
            expected_entries("UnlimitedToolWorks", "0.0.1"),
            {
                "UnlimitedToolWorks-0.0.1",
                "UnlimitedToolWorks-0.0.1/UnlimitedToolWorks.exe",
                "UnlimitedToolWorks-0.0.1/quick_compile.bat",
                "UnlimitedToolWorks-0.0.1/quick_upgrade.bat",
                "UnlimitedToolWorks-0.0.1/README.txt",
                "UnlimitedToolWorks-0.0.1/LICENSE-MIT",
                "UnlimitedToolWorks-0.0.1/LICENSE-APACHE",
                "UnlimitedToolWorks-0.0.1/exports",
                "UnlimitedToolWorks-0.0.1/old_exports",
                "UnlimitedToolWorks-0.0.1/data",
                "UnlimitedToolWorks-0.0.1/data/settings.toml",
                "UnlimitedToolWorks-0.0.1/data/rules",
                "UnlimitedToolWorks-0.0.1/data/rules/dbc7.4ccrules",
                "UnlimitedToolWorks-0.0.1/data/rules/vgl27.4ccrules",
            },
        )

    def test_sha256_line(self) -> None:
        self.assertEqual(sha256_line("ab12cd", "a.7z"), "ab12cd  a.7z\n")

    def test_symbols_entries(self) -> None:
        self.assertEqual(
            symbols_entries("UnlimitedToolWorks"),
            {"UnlimitedToolWorks.exe", "4cc_studio.pdb"},
        )

    def test_existing_glob_covers_the_bundle_and_the_symbols(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            for file in (
                "Tiny-0.0.1-windows.7z",
                "Tiny-0.0.1-windows.7z.sha256",
                "Tiny-0.0.1-symbols.7z",
            ):
                (dist / file).write_bytes(b"x")
            (dist / "Tiny-0.0.2-windows.7z").write_bytes(b"x")
            self.assertEqual(
                sorted(dist.glob(existing_glob("0.0.1"))),
                sorted(
                    dist / file
                    for file in (
                        "Tiny-0.0.1-windows.7z",
                        "Tiny-0.0.1-windows.7z.sha256",
                        "Tiny-0.0.1-symbols.7z",
                    )
                ),
            )
            self.assertEqual(
                list(dist.glob(existing_glob("0.0.2"))),
                [dist / "Tiny-0.0.2-windows.7z"],
            )


class Listing(unittest.TestCase):
    def test_a_packed_bundle_lists_the_staged_set(self) -> None:
        seven_z = find_7z()
        if seven_z is None:
            self.fail("7-Zip not found: neither `7z` on PATH nor C:\\Program Files\\7-Zip\\7z.exe")
        with tempfile.TemporaryDirectory() as tmp:
            stage = Path(tmp)
            top = stage / "Tiny-9.9.9"
            (top / "exports").mkdir(parents=True)
            (top / "old_exports").mkdir()
            (top / "data/rules").mkdir(parents=True)
            (top / "data/settings.toml").write_bytes(b"x")
            (top / "data/rules/dbc7.4ccrules").write_bytes(b"x")
            (top / "data/rules/vgl27.4ccrules").write_bytes(b"x")
            for bundled in (
                "Tiny.exe",
                "quick_compile.bat",
                "quick_upgrade.bat",
                "README.txt",
                "LICENSE-MIT",
                "LICENSE-APACHE",
            ):
                (top / bundled).write_bytes(b"x")
            archive = stage / "tiny.7z"
            subprocess.run(
                [seven_z, "a", "-t7z", "-mx=9", str(archive), top.name],
                cwd=stage,
                check=True,
                capture_output=True,
            )
            listing = subprocess.run(
                [seven_z, "l", "-slt", str(archive)],
                cwd=stage,
                check=True,
                capture_output=True,
                text=True,
            )
            found = listing_paths(listing.stdout)
            expected = expected_entries("Tiny", "9.9.9")
            self.assertEqual(found, expected)
            self.assertNotEqual(found, expected - {"Tiny-9.9.9/README.txt"})

    def test_a_packed_symbols_archive_lists_its_two_entries(self) -> None:
        # The same argv `release.py` packs the symbols with: two bare names under the
        # staging folder, so the archive's entries are flat.
        seven_z = find_7z()
        if seven_z is None:
            self.fail("7-Zip not found: neither `7z` on PATH nor C:\\Program Files\\7-Zip\\7z.exe")
        with tempfile.TemporaryDirectory() as tmp:
            stage = Path(tmp)
            (stage / "Tiny.exe").write_bytes(b"x")
            (stage / "4cc_studio.pdb").write_bytes(b"y")
            archive = stage / "tiny-symbols.7z"
            subprocess.run(
                [seven_z, "a", "-t7z", "-mx=9", str(archive), "Tiny.exe", "4cc_studio.pdb"],
                cwd=stage,
                check=True,
                capture_output=True,
            )
            listing = subprocess.run(
                [seven_z, "l", "-slt", str(archive)],
                cwd=stage,
                check=True,
                capture_output=True,
                text=True,
            )
            self.assertEqual(listing_paths(listing.stdout), symbols_entries("Tiny"))


if __name__ == "__main__":
    unittest.main()
