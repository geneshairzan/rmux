#!/bin/bash
# Build rmux .deb from the release binary. Run from the repo root:
#   ./packaging/deb/build-deb.sh [version] [arch]
set -euo pipefail

VERSION="${1:-1.0.0}"
ARCH="${2:-amd64}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
STAGE="$ROOT/target/deb-stage"
PKG="$STAGE/rmux_${VERSION}_${ARCH}"

cargo build --release

rm -rf "$PKG"
mkdir -p "$PKG/DEBIAN" \
  "$PKG/usr/bin" \
  "$PKG/usr/share/applications" \
  "$PKG/usr/share/icons/hicolor/scalable/apps"

sed -e "s/^Version: .*/Version: $VERSION/" \
    -e "s/^Architecture: .*/Architecture: $ARCH/" \
    "$ROOT/packaging/deb/DEBIAN/control" > "$PKG/DEBIAN/control"

install -m 0755 "$ROOT/target/release/rmux" "$PKG/usr/bin/rmux"
ln -sf rmux "$PKG/usr/bin/cmux-native"
install -m 0644 "$ROOT/data/rmux.desktop" "$PKG/usr/share/applications/rmux.desktop"
install -m 0644 "$ROOT/data/cmux-native.desktop" "$PKG/usr/share/applications/cmux-native.desktop"
install -m 0644 "$ROOT/data/icons/rmux.svg" "$PKG/usr/share/icons/hicolor/scalable/apps/rmux.svg"
install -m 0644 "$ROOT/data/icons/cmux-native.svg" "$PKG/usr/share/icons/hicolor/scalable/apps/cmux-native.svg"

dpkg-deb --build "$PKG"
echo "Built: $PKG.deb"
