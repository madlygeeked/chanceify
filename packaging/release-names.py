#!/usr/bin/env python3
"""Publish Spotifast filenames and byte-identical names for older updaters."""

import argparse
import hashlib
from pathlib import Path
import re
import shutil


def prepare(directory, tag):
    if not re.fullmatch(r"v\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?", tag):
        raise ValueError("Expected a release tag")
    bridge = tuple(map(int, tag[1:].split("-")[0].split("."))) <= (0, 9, 1)
    pairs = {}
    for path in directory.iterdir():
        for prefix in ("spotifast-", "fastpotify-"):
            if path.name.startswith(prefix + tag + "-"):
                if not bridge and prefix == "fastpotify-":
                    raise ValueError("Legacy release downloads are forbidden after 0.9.1")
                if not path.is_file() or path.is_symlink():
                    raise ValueError(f"Expected a regular release file: {path.name}")
                suffix = path.name[len(prefix):]
                pairs[suffix] = (
                    directory / ("spotifast-" + suffix),
                    directory / ("fastpotify-" + suffix),
                )
    if not pairs:
        raise ValueError("No release downloads found")
    # Check every pair before writing, so conflicting inputs never get replaced.
    def digest(path):
        with path.open("rb") as stream:
            return hashlib.file_digest(stream, "sha256").hexdigest()

    for canonical, legacy in pairs.values():
        if canonical.exists() and legacy.exists() and digest(canonical) != digest(legacy):
            raise ValueError(f"Different bytes for {canonical.name} and {legacy.name}")
    for canonical, legacy in pairs.values():
        if not canonical.exists():
            shutil.copyfile(legacy, canonical)
        elif bridge and not legacy.exists():
            shutil.copyfile(canonical, legacy)
    files = sorted(path for pair in pairs.values() for path in (pair if bridge else pair[:1]))
    manifest = "".join(f"{digest(path)}  {path.name}\n" for path in files)
    (directory / "checksums.txt").write_text(manifest)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("tag")
    args = parser.parse_args()
    prepare(args.directory, args.tag)
