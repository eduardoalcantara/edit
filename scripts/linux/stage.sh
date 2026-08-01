#!/usr/bin/env bash
# Monta árvore FHS em dist/stage (usr/bin, share/applications, icons, metainfo).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_common.sh
source "${SCRIPT_DIR}/_common.sh"

ensure_release_binary

rm -rf "$STAGE"
mkdir -p \
  "${STAGE}/usr/bin" \
  "${STAGE}/usr/share/applications" \
  "${STAGE}/usr/share/metainfo" \
  "${STAGE}/usr/share/icons/hicolor/scalable/apps" \
  "${STAGE}/usr/share/icons/hicolor/48x48/apps" \
  "${STAGE}/usr/share/icons/hicolor/128x128/apps"

install -m 755 "${ROOT}/target/release/edit" "${STAGE}/usr/bin/edit"
install -m 644 "${PACKAGING}/edit.desktop" "${STAGE}/usr/share/applications/edit.desktop"
install -m 644 "${PACKAGING}/edit.metainfo.xml" "${STAGE}/usr/share/metainfo/edit.metainfo.xml"
install -m 644 "${PACKAGING}/icons/hicolor/scalable/apps/edit.svg" \
  "${STAGE}/usr/share/icons/hicolor/scalable/apps/edit.svg"
install -m 644 "${PACKAGING}/icons/hicolor/48x48/apps/edit.png" \
  "${STAGE}/usr/share/icons/hicolor/48x48/apps/edit.png"
install -m 644 "${PACKAGING}/icons/hicolor/128x128/apps/edit.png" \
  "${STAGE}/usr/share/icons/hicolor/128x128/apps/edit.png"

echo "Staging FHS em ${STAGE}"
