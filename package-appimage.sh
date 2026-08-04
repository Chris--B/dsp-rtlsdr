#!/usr/bin/env bash
set -euo pipefail

# Configuration
APP_NAME="wavy-mcgee"
TARGET_ARCH="x86_64-unknown-linux-gnu"
BUILD_DIR="target/AppDir"
OUTPUT_DIR="SteamDeck"
OUTPUT_APPIMAGE="${OUTPUT_DIR}/${APP_NAME}-x86_64.AppImage"

echo "==> Building ${APP_NAME} for ${TARGET_ARCH}..."

echo "==> Preparing AppDir directory structure..."
rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/lib"
mkdir -p "${OUTPUT_DIR}"

echo "==> Copying binary..."
cp "target/${TARGET_ARCH}/release/${APP_NAME}" "${BUILD_DIR}/usr/bin/"

echo "==> Copying shared library dependencies (.so)..."
# Using cp -L to dereference symlinks so actual payload files are included
cp -L /usr/lib/x86_64-linux-gnu/librtlsdr.so* "${BUILD_DIR}/usr/lib/" 2>/dev/null || true
cp -L /usr/lib/x86_64-linux-gnu/libusb-1.0.so* "${BUILD_DIR}/usr/lib/" 2>/dev/null || true

echo "==> Creating AppRun entrypoint script..."
cat <<'EOF' > "${BUILD_DIR}/AppRun"
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/wavy-mcgee" "$@" > wavy.stdout 2> wavy.stderr
EOF
chmod +x "${BUILD_DIR}/AppRun"

echo "==> Creating Desktop Entry..."
cat <<EOF > "${BUILD_DIR}/${APP_NAME}.desktop"
[Desktop Entry]
Name=${APP_NAME}
Exec=${APP_NAME}
Icon=${APP_NAME}
Type=Application
Categories=Utility;
Comment=SDR App for Steam Deck
EOF

echo "==> Creating icon..."
if [ -f "assets/rtlsdr.png" ]; then
    cp "assets/rtlsdr.png" "${BUILD_DIR}/${APP_NAME}.png"
else
    # AppImage requires an icon file matching Icon= in .desktop
    touch "${BUILD_DIR}/${APP_NAME}.png"
fi

echo "==> Running appimagetool..."
export ARCH=x86_64
export APPIMAGE_EXTRACT_AND_RUN=1

appimagetool "${BUILD_DIR}" "${OUTPUT_APPIMAGE}"

echo "==> Success! Built: ${OUTPUT_APPIMAGE}"
