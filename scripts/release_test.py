"""Tests of scripts/release.py; `just release` runs them before the recipe."""

import re
import subprocess
import tempfile
import unittest
from pathlib import Path

from release import (
    expected_entries,
    exe_name,
    find_7z,
    launcher_calls,
    listing_paths,
    readme_names_version,
    sha256_line,
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
            },
        )

    def test_sha256_line(self) -> None:
        self.assertEqual(sha256_line("ab12cd", "a.7z"), "ab12cd  a.7z\n")


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


if __name__ == "__main__":
    unittest.main()
