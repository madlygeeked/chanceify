"""Exercise native and Flatpak launcher installs using current and old payloads.

Uses the actual AUR package() and Flatpak install commands, without compiling or
launching the app. Ruby provides the existing YAML parser; no Python packages.
"""
import configparser
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent


def yaml(path):
    return json.loads(subprocess.check_output([
        "ruby", "-rjson", "-ryaml", "-e",
        "puts JSON.generate(YAML.safe_load_file(ARGV.fetch(0), aliases: false))", str(path),
    ], text=True))


def payload(root, desktop):
    (root / "packaging/applications").mkdir(parents=True)
    (root / "packaging/icons").mkdir(parents=True)
    source = ROOT / ("packaging/applications/spotifast.desktop" if desktop == "spotifast"
                     else "packaging/fixtures/fastpotify-0.8.0.desktop")
    shutil.copyfile(source, root / f"packaging/applications/{desktop}.desktop")
    shutil.copyfile(ROOT / "packaging/icons/spotifast.svg", root / f"packaging/icons/{desktop}.svg")
    (root / "target/release").mkdir(parents=True)
    for name in ["fastpotify", "target/release/fastpotify"]:
        shutil.copyfile(shutil.which("true"), root / name)
    if desktop == "spotifast":
        for name in ["spotifast", "target/release/spotifast"]:
            shutil.copyfile(shutil.which("true"), root / name)
    for name in ["LICENSE", "README.md"]:
        shutil.copyfile(ROOT / name, root / name)
    shutil.copytree(ROOT / "contrib/omarchy", root / "contrib/omarchy")
    (root / "packaging/flatpak").mkdir()
    shutil.copyfile(ROOT / "packaging/flatpak/rocks.spotifast.Spotifast.metainfo.xml",
                    root / "packaging/flatpak/rocks.spotifast.Spotifast.metainfo.xml")


class LauncherInstallTest(unittest.TestCase):
    def test_generated_aur_versions_start_at_the_requested_release(self):
        for package in ["spotifast", "spotifast-bin", "spotifast-git"]:
            with self.subTest(package=package), tempfile.TemporaryDirectory() as directory:
                recipe = Path(directory) / "PKGBUILD"
                template = (ROOT / f"packaging/arch/{package}/PKGBUILD.in").read_text()
                recipe.write_text(template.replace("@VERSION@", "9.8.7"))
                version = subprocess.check_output([
                    "bash", "-c", 'source "$1"; printf "%s" "$pkgver"', "check", str(recipe),
                ], text=True)
                self.assertEqual(version, "9.8.7")

    def check_launcher(self, prefix, desktop, window_class, icon):
        entries = list((prefix / "share/applications").glob("*.desktop"))
        self.assertEqual([p.name for p in entries], [desktop + ".desktop"])
        parser = configparser.ConfigParser(interpolation=None)
        parser.read(entries[0])
        entry = parser["Desktop Entry"]
        self.assertEqual(entry["Name"], "Spotifast")
        self.assertEqual(entry["Icon"], icon)
        self.assertEqual(entry["StartupWMClass"], window_class)
        self.assertEqual(entry["Exec"], "spotifast %u")
        self.assertIn("x-scheme-handler/spotify;", entry["MimeType"])
        icons = list((prefix / "share/icons/hicolor/scalable/apps").glob("*.svg"))
        self.assertEqual([p.name for p in icons], [icon + ".svg"])
        self.assertEqual(icons[0].read_bytes(), (ROOT / "packaging/icons/spotifast.svg").read_bytes())
        self.assertFalse((prefix / "bin/spotifast").is_symlink())
        self.assertTrue((prefix / "bin/spotifast").is_file())
        self.assertEqual(os.readlink(prefix / "bin/fastpotify"), "spotifast")

    def test_current_and_historical_aur_payloads_install_matching_launchers(self):
        for package in ["spotifast", "spotifast-bin", "spotifast-git"]:
            for desktop in ["spotifast", "fastpotify"]:
                if package.endswith("-git") and desktop == "fastpotify":
                    continue  # The git package builds only the current source.
                with self.subTest(package=package, desktop=desktop), tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    if package.endswith("-bin"):
                        relative = desktop + "-v9.8.7-x86_64-unknown-linux-gnu"
                    elif package.endswith("-git"):
                        relative = package
                    else:
                        relative = desktop + "-9.8.7"
                    source = root / "src" / relative
                    payload(source, desktop)
                    template = ROOT / f"packaging/arch/{package}/PKGBUILD.in"
                    command = "source " + shlex.quote(str(template)) + "; pkgver=9.8.7; package"
                    subprocess.run(["bash", "-euc", command], check=True, cwd=root, env={
                        **os.environ, "CARCH": "x86_64", "srcdir": str(root / "src"),
                        "pkgdir": str(root / "pkg"),
                    })
                    self.check_launcher(root / "pkg/usr", desktop, desktop, desktop)

    def test_flatpak_install_commands_keep_app_id_and_icon_aligned(self):
        for manifest in ["rocks.spotifast.Spotifast.yml", "rocks.spotifast.Spotifast.bundle.yml"]:
            config = yaml(ROOT / "packaging/flatpak" / manifest)
            commands = config["modules"][-1]["build-commands"]
            for desktop in ["spotifast", "fastpotify"]:
                with self.subTest(manifest=manifest, desktop=desktop), tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    payload(root / "source", desktop)
                    # Exercise installation only. Cargo build coverage belongs to
                    # the app suites; this fixture supplies the completed input.
                    commands_to_run = [c for c in commands if not c.startswith("cargo ")]
                    command = "\n".join(c.replace("/app/", str(root / "app") + "/") for c in commands_to_run)
                    subprocess.run(["bash", "-euc", command], check=True, cwd=root / "source")
                    self.check_launcher(root / "app", config["id"],
                                        config["id"] if desktop == "spotifast" else desktop, config["id"])

    def test_native_manifest_preserves_the_input_launcher_filename(self):
        config = yaml(ROOT / "native-packages.yaml")
        entries = [c for c in config["nfpm"]["contents"] if "/packaging/" in c["src"]]
        self.assertEqual(len(entries), 2)
        for desktop in ["spotifast", "fastpotify"]:
            with self.subTest(desktop=desktop), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                payload(root / (desktop + "-v9.8.7-linux-amd64"), desktop)
                for entry in entries:
                    pattern = entry["src"].replace("@PAYLOAD@/", "").replace("@VERSION@", "9.8.7").replace("@TARGET@", "linux-amd64")
                    files = list(root.glob(pattern))
                    self.assertEqual(len(files), 1)
                    self.assertEqual(files[0].stem, desktop)
                    self.assertTrue(entry["dst"].endswith("/"))


if __name__ == "__main__":
    unittest.main()
