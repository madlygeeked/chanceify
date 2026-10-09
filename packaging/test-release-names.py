import importlib.util
from pathlib import Path
import tempfile
import sys
import tomllib
import unittest

sys.dont_write_bytecode = True

spec = importlib.util.spec_from_file_location("release_names", Path(__file__).with_name("release-names.py"))
release_names = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release_names)


class ReleaseNamesTest(unittest.TestCase):
    def test_current_version_has_written_release_notes_before_tagging(self):
        root = Path(__file__).resolve().parent.parent
        version = tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"]
        notes = root / "packaging/release-notes" / f"v{version}.md"
        self.assertTrue(notes.is_file(), f"Write {notes} before tagging the release")
        self.assertTrue(notes.read_text().strip(), "Release notes must not be empty")

    def test_releases_after_the_bridge_have_only_the_new_name(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "spotifast-v0.9.2-linux.tar.gz").write_bytes(b"new build")
            release_names.prepare(root, "v0.9.2")
            self.assertEqual(len((root / "checksums.txt").read_text().splitlines()), 1)
            self.assertFalse((root / "fastpotify-v0.9.2-linux.tar.gz").exists())
            (root / "fastpotify-v0.9.2-linux.tar.gz").write_bytes(b"legacy build")
            with self.assertRaisesRegex(ValueError, "forbidden"):
                release_names.prepare(root, "v0.9.2")

    def test_old_and_new_inputs_keep_their_bytes_and_checksums(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "fastpotify-v0.8.0-linux.tar.gz").write_bytes(b"unchanged Linux build")
            (root / "spotifast-v0.8.0-macos.dmg").write_bytes(b"notarized Mac build")
            release_names.prepare(root, "v0.8.0")
            self.assertEqual((root / "spotifast-v0.8.0-linux.tar.gz").read_bytes(), b"unchanged Linux build")
            self.assertEqual((root / "fastpotify-v0.8.0-macos.dmg").read_bytes(), b"notarized Mac build")
            before = (root / "checksums.txt").read_bytes()
            self.assertEqual(len(before.splitlines()), 4)
            release_names.prepare(root, "v0.8.0")
            self.assertEqual((root / "checksums.txt").read_bytes(), before)

    def test_conflicting_inputs_fail_without_replacing_either_file(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            old = root / "fastpotify-v0.8.0-linux.tar.gz"
            new = root / "spotifast-v0.8.0-linux.tar.gz"
            old.write_bytes(b"old")
            new.write_bytes(b"new")
            with self.assertRaisesRegex(ValueError, "Different bytes"):
                release_names.prepare(root, "v0.8.0")
            self.assertEqual(old.read_bytes(), b"old")
            self.assertEqual(new.read_bytes(), b"new")
            self.assertFalse((root / "checksums.txt").exists())

    def test_missing_inputs_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "No release"):
                release_names.prepare(Path(directory), "v0.8.0")


if __name__ == "__main__":
    unittest.main()
