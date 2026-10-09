#!/usr/bin/env bash
# Usage: bash packaging/test-install.sh ubuntu:24.04 dist/packages/0.8.0 [legacy-packages] [desktop-id]
# Run on the matching architecture, with a C compiler and Docker available.
set -euo pipefail

image=${1:?Supply a Debian, Ubuntu or Fedora container image}
packages=$(realpath "${2:?Supply a native-packages output directory}")
legacy_packages=$(realpath "${3:-$packages/legacy}")
desktop_id=${4:-spotifast}
case "$desktop_id" in
  spotifast|fastpotify) ;;
  *) echo 'Unsupported desktop fixture identity' >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64) target=linux-amd64 ;;
  aarch64) target=linux-arm64 ;;
  *) echo 'Unsupported test architecture' >&2; exit 1 ;;
esac
case "$image" in
  ubuntu:*|debian:*) format=deb ;;
  fedora:*) format=rpm ;;
  *) echo 'Unsupported test distribution' >&2; exit 1 ;;
esac
package_dir="$packages/packages/$target/$format"
test -d "$package_dir"
legacy_mount=()
if [[ -d "$legacy_packages" ]]; then
  legacy_mount=(--volume "$legacy_packages:/legacy:ro")
fi
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
checks=$(mktemp -d)
trap 'rm -rf -- "$checks"' EXIT
cc -std=c99 -Wall -Wextra -Werror "$script_dir/check-runtime-libs.c" -ldl -o "$checks/check-runtime-libs"

docker run --rm \
  --volume "$package_dir:/packages:ro" \
  --volume "$checks:/checks:ro" \
  "${legacy_mount[@]}" \
  --env "FORMAT=$format" \
  --env "DESKTOP_ID=$desktop_id" \
  "$image" sh -ec '
    set -- /packages/*."$FORMAT"
    test "$#" -eq 1
    test -f "$1"
    mkdir -p /root/.config/fastpotify
    printf "%s\n" "preserve-existing-settings" > /root/.config/fastpotify/rename-fixture
    if [ "$FORMAT" = deb ]; then
      apt-get update
      if [ -d /legacy ]; then
        DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends /legacy/*.deb
        dpkg-query -W fastpotify
      fi
      DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends "$1"
      dpkg-query -W spotifast
      test "$(dpkg-query -W -f="\${db:Status-Status}" fastpotify 2>/dev/null || true)" != installed
    else
      if [ -d /legacy ]; then
        dnf install -y --setopt=install_weak_deps=False /legacy/*.rpm
        rpm -q fastpotify
      fi
      dnf install -y --setopt=install_weak_deps=False "$1"
      rpm -q spotifast
      ! rpm -q fastpotify
    fi
    # --version exercises linked libraries; the probe checks dlopen libraries
    # without installing a desktop, compiler, interpreter or test dependencies.
    # Trace the isolated fixture assertions so a failed check identifies itself.
    set -x
    fastpotify --version
    spotifast --version
    test -f /usr/bin/spotifast
    test ! -L /usr/bin/spotifast
    test "$(readlink /usr/bin/fastpotify)" = spotifast
    test -f /usr/share/licenses/spotifast/LICENSE
    if [ "$FORMAT" = deb ]; then
      # Slim Debian/Ubuntu images exclude /usr/share/doc at installation time.
      # Verify the regular file in the package, not the intentionally stripped root.
      dpkg-deb --contents "$1" | grep -E "^-.* ./usr/share/doc/spotifast/README.md$"
    else
      test -f /usr/share/doc/spotifast/README.md
    fi
    /checks/check-runtime-libs
    test -s "/usr/share/applications/$DESKTOP_ID.desktop"
    test -s "/usr/share/icons/hicolor/scalable/apps/$DESKTOP_ID.svg"
    grep -qx "Icon=$DESKTOP_ID" "/usr/share/applications/$DESKTOP_ID.desktop"
    grep -qx "StartupWMClass=$DESKTOP_ID" "/usr/share/applications/$DESKTOP_ID.desktop"
    if [ "$DESKTOP_ID" = spotifast ]; then other_id=fastpotify; else other_id=spotifast; fi
    test ! -e "/usr/share/applications/$other_id.desktop"
    test ! -e "/usr/share/icons/hicolor/scalable/apps/$other_id.svg"
    test -s /usr/share/spotifast/omarchy/spotifast.json.tpl
    test -x /usr/share/spotifast/omarchy/spotifast-theme
    test "$(cat /root/.config/fastpotify/rename-fixture)" = preserve-existing-settings
    if [ "$FORMAT" = deb ]; then
      apt-get remove -y spotifast
    else
      dnf remove -y spotifast
    fi
    test ! -e /usr/bin/fastpotify
    test ! -e /usr/bin/spotifast
    test ! -e /usr/share/applications/fastpotify.desktop
    test ! -e /usr/share/applications/spotifast.desktop
    test ! -e /usr/share/icons/hicolor/scalable/apps/fastpotify.svg
    test ! -e /usr/share/icons/hicolor/scalable/apps/spotifast.svg
    test ! -e /usr/share/spotifast/omarchy/spotifast.json.tpl
    test ! -e /usr/share/spotifast/omarchy/spotifast-theme
    test "$(cat /root/.config/fastpotify/rename-fixture)" = preserve-existing-settings
  '
