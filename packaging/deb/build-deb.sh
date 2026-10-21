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
  "$PKG/usr/share/metainfo" \
  "$PKG/usr/share/icons/hicolor/scalable/apps"

for size in 16 22 24 32 48 64 128 256 512; do
  mkdir -p "$PKG/usr/share/icons/hicolor/${size}x${size}/apps"
done

sed -e "s/^Version: .*/Version: $VERSION/" \
    -e "s/^Architecture: .*/Architecture: $ARCH/" \
    "$ROOT/packaging/deb/DEBIAN/control" > "$PKG/DEBIAN/control"
install -m 0755 "$ROOT/packaging/deb/DEBIAN/postinst" "$PKG/DEBIAN/postinst"

install -m 0755 "$ROOT/target/release/rmux" "$PKG/usr/bin/rmux"
ln -sf rmux "$PKG/usr/bin/cmux-native"
install -m 0644 "$ROOT/data/org.rmux.Terminal.desktop" "$PKG/usr/share/applications/org.rmux.Terminal.desktop"
install -m 0644 "$ROOT/data/cmux-native.desktop" "$PKG/usr/share/applications/cmux-native.desktop"
install -m 0644 "$ROOT/data/org.rmux.Terminal.metainfo.xml" "$PKG/usr/share/metainfo/org.rmux.Terminal.metainfo.xml"
install -m 0644 "$ROOT/data/icons/rmux.svg" "$PKG/usr/share/icons/hicolor/scalable/apps/rmux.svg"
install -m 0644 "$ROOT/data/icons/cmux-native.svg" "$PKG/usr/share/icons/hicolor/scalable/apps/cmux-native.svg"
for size in 16 22 24 32 48 64 128 256 512; do
  install -m 0644 "$ROOT/data/icons/hicolor/rmux-${size}.png" "$PKG/usr/share/icons/hicolor/${size}x${size}/apps/rmux.png"
done

dpkg-deb --build "$PKG"
echo "Built: $PKG.deb"
