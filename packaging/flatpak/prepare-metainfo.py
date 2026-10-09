#!/usr/bin/env python3
"""Use a release's original metainfo with the current Flatpak identity."""

from pathlib import Path
import re
import subprocess
import sys
import xml.etree.ElementTree as ET

APP_ID = "rocks.spotifast.Spotifast"
LEGACY_ID = "rocks.fastpotify.Fastpotify"


def rename_metainfo(contents, version):
    component = ET.fromstring(contents)
    app_id = component.find("id")
    launchable = component.find("launchable[@type='desktop-id']")
    name = component.find("name")
    release = component.find("releases/release")
    if app_id is None or app_id.text not in (APP_ID, LEGACY_ID):
        raise ValueError("unexpected application ID in release metainfo")
    if launchable is None or launchable.text != app_id.text + ".desktop":
        raise ValueError("release desktop ID does not match its application ID")
    if name is None or release is None or release.get("version") != version:
        raise ValueError("release metainfo version does not match the requested tag")
    app_id.text = APP_ID
    launchable.text = APP_ID + ".desktop"
    name.text = "Spotifast"
    return ET.tostring(component, encoding="utf-8", xml_declaration=True) + b"\n"


def release_metainfo(root, tag):
    for app_id in (APP_ID, LEGACY_ID):
        result = subprocess.run(
            # The hosted container can have a different UID from checkout.
            # Trust only this known source directory, for this read command.
            ["git", "-c", f"safe.directory={root}", "show",
             f"{tag}:packaging/flatpak/{app_id}.metainfo.xml"],
            cwd=root, capture_output=True, check=False,
        )
        if result.returncode == 0:
            return rename_metainfo(result.stdout, tag[1:])
    raise ValueError("no application metainfo found at the requested release tag")


def main(tag, output):
    if not re.fullmatch(r"v\d+\.\d+\.\d+(?:[.-][a-zA-Z0-9.-]+)?", tag):
        raise ValueError("expected a release tag such as v0.8.0")
    root = Path(__file__).resolve().parents[2]
    Path(output).write_bytes(release_metainfo(root, tag))


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: prepare-metainfo.py TAG OUTPUT")
    try:
        main(*sys.argv[1:])
    except (ValueError, OSError, ET.ParseError) as error:
        sys.exit(str(error))
