---
title: Download
description: Download the app for macOS, Windows, or Linux, with install instructions for each.
nav_order: 1
---

Spotifast was previously called **Fastpotify**. Version 0.9.1 completed the
profile and package rename. Choose Spotifast when installing or updating.
Your settings and sign-ins carry over, except when
[switching the Flatpak installation](/renaming/#flatpak).
If you use a version older than 0.9.1, see the [upgrade guide](/renaming/)
before updating: 0.9.1 was the last release older updaters can install.

{% assign v = site.spotifast_version %}
{% assign base = "https://github.com/crmne/spotifast/releases/download/v" | append: v %}

The current version is **v{{ v }}**. You can use the
[download checksums]({{ base }}/checksums.txt) to check that a file has not
been damaged or changed. Older versions are on the
[releases page](https://github.com/crmne/spotifast/releases).

## macOS

One download for both Apple Silicon and Intel:

- [Download for Mac]({{ base }}/spotifast-v{{ v }}-macos-universal.dmg)

Open the downloaded file and drag **Spotifast** to **Applications**.

If you use [Homebrew](https://brew.sh), you can install it with:

```sh
brew install --cask crmne/tap/spotifast
```

### First open on macOS

Open **Spotifast** from Applications. If your Mac asks whether you want to
open an app downloaded from the internet, choose **Open**.

Starting with 0.8.0, the Mac download passes Apple's security checks. You do
not need to change security settings or run commands in Terminal to open it.

Quit the old app before opening Spotifast. Your saved settings and sign-ins
carry over. If an older version reports that its update helper exited before
it was ready, install the DMG above manually once. The helper changes take
effect for updates started from 0.9.1 onward.

## Windows

Download and run the installer. It adds Spotifast to the Start menu and does
not need an administrator password. Choose the first download for most PCs,
or the ARM version if your PC uses an ARM processor:

- [Windows installer (most PCs)]({{ base }}/spotifast-v{{ v }}-x86_64-pc-windows-msvc-setup.exe)
- [Windows installer (ARM PCs)]({{ base }}/spotifast-v{{ v }}-aarch64-pc-windows-msvc-setup.exe)

To run Spotifast without an installer, download a ZIP file, extract it, and
open `spotifast.exe`.

- [Windows ZIP (most PCs)]({{ base }}/spotifast-v{{ v }}-x86_64-pc-windows-msvc.zip)
- [Windows ZIP (ARM PCs)]({{ base }}/spotifast-v{{ v }}-aarch64-pc-windows-msvc.zip)

Either way, SmartScreen may warn about an unknown publisher on first run;
choose **More info**, then **Run anyway**.

Spotifast needs OpenGL 2.0 or newer, which every current Windows graphics
driver provides. In a virtual machine or remote session that only has the
Microsoft Basic Display Adapter, it may not start; install the machine's
graphics driver, or enable GPU acceleration for the virtual machine.

To choose which app opens Spotify links, use **Settings → Apps → Default apps**
in Windows.

## Linux

### Arch Linux

Spotifast is in the AUR, Arch's community package collection. Installing it
also adds it to your app launcher:

```sh
yay -S spotifast-bin      # the released build, ready made
yay -S spotifast          # the release, built from source
yay -S spotifast-git      # built from the latest commit
```

If you already have an old `fastpotify` package, install the corresponding
`spotifast` package above and accept the replacement. Your settings and saved
sign-ins are kept.

### Flatpak

Download the [Spotifast Flatpak]({{ base }}/spotifast-v{{ v }}-x86_64.flatpak?flatpak-id=rocks.spotifast.Spotifast).
It requires Flatpak and the Freedesktop 24.08 runtime, a set of shared
components used by Flatpak apps:

```sh
flatpak install --user ~/Downloads/spotifast-v{{ v }}-x86_64.flatpak
flatpak run rocks.spotifast.Spotifast
```

The application ID is **`rocks.spotifast.Spotifast`**. Existing Fastpotify
Flatpak users install this as a new application, then remove the old one.
See [switching Flatpak installations](/renaming/#flatpak) to retain settings
and history. Sign in again after switching.

To update this Flatpak installation, download and install the new release.
Flathub support is planned.

Packages from other stores are maintained by their publishers. Report
problems specific to those packages to their maintainers.

### AppImage

- [AppImage (Intel and AMD, 64-bit)]({{ base }}/spotifast-{{ v }}-x86_64.AppImage)
- [AppImage (ARM, 64-bit)]({{ base }}/spotifast-{{ v }}-aarch64.AppImage)

One file, no installation: make it executable and run it.

```sh
chmod +x ~/Downloads/spotifast-{{ v }}-x86_64.AppImage
~/Downloads/spotifast-{{ v }}-x86_64.AppImage
```

The AppImage bundles no libraries: like the DEB and RPM, it needs glibc 2.39
or newer and your desktop's own libraries. Running it needs FUSE; without
FUSE, start it with `--appimage-extract-and-run`. It does not update itself:
download the new file when Spotifast says a release is out.

### Other distributions

- [Linux archive (Intel and AMD, 64-bit)]({{ base }}/spotifast-v{{ v }}-x86_64-unknown-linux-gnu.tar.gz)
- [Linux archive (ARM, 64-bit)]({{ base }}/spotifast-v{{ v }}-aarch64-unknown-linux-gnu.tar.gz)

Unpack, put `spotifast` on your PATH, and copy the desktop entry and icon
from the bundled `packaging/` directory if you want it in your launcher and
handling `spotify:` links.
The binary needs ALSA, PulseAudio or PipeWire, and Wayland or X11.

Or build from source: see the [build instructions](https://github.com/crmne/spotifast#install).

## Nix

Add the repository [flake](https://github.com/crmne/spotifast) to your
inputs:

```nix
inputs.spotifast.url = "github:crmne/spotifast";
```

On NixOS, install the default package:

```nix
environment.systemPackages = [
  inputs.spotifast.packages."${pkgs.stdenv.hostPlatform.system}".default
];
```

### nix-darwin

On macOS, use the same `spotifast` package to install Spotifast as a Mac app:

```nix
environment.systemPackages = [
  inputs.spotifast.packages."${pkgs.stdenv.hostPlatform.system}".spotifast
];
environment.pathsToLink = [ "/Applications" ];
```

The bundle appears in `/Applications/Nix Apps`. With Home Manager,
`home.packages` is enough; its darwin support links app bundles into
`~/Applications`:

```nix
home.packages = [
  inputs.spotifast.packages."${pkgs.stdenv.hostPlatform.system}".spotifast
];
```
