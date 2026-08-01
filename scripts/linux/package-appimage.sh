#!/usr/bin/env bash
# Gera AppImage em dist/ (requer appimagetool ou linuxdeploy).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_common.sh
source "${SCRIPT_DIR}/_common.sh"

"${SCRIPT_DIR}/stage.sh"

VER="$(edit_version)"
ARCH="$(uname -m)"
APPDIR="${DIST}/Edit.AppDir"
rm -rf "$APPDIR"
mkdir -p "${APPDIR}/usr"

cp -a "${STAGE}/usr/." "${APPDIR}/usr/"
cp -f "${PACKAGING}/edit.desktop" "${APPDIR}/edit.desktop"
cp -f "${PACKAGING}/icons/hicolor/128x128/apps/edit.png" "${APPDIR}/edit.png"
ln -sf usr/bin/edit "${APPDIR}/AppRun"
chmod +x "${APPDIR}/usr/bin/edit"

OUT="${DIST}/edit-${VER}-${ARCH}.AppImage"

if command -v linuxdeploy >/dev/null 2>&1; then
  (
    cd "$DIST"
    linuxdeploy --appdir "$APPDIR" \
      --executable "${APPDIR}/usr/bin/edit" \
      --desktop-file "${APPDIR}/edit.desktop" \
      --icon-file "${APPDIR}/edit.png" \
      --output appimage
  )
  shopt -s nullglob
  FOUND=""
  for g in "${DIST}"/*.AppImage; do
    if [[ -f "$g" ]]; then FOUND="$g"; break; fi
  done
  if [[ -z "$FOUND" ]]; then
    echo "Erro: linuxdeploy não produziu AppImage." >&2
    exit 1
  fi
  mv -f "$FOUND" "$OUT"
elif command -v appimagetool >/dev/null 2>&1; then
  ARCH="$ARCH" appimagetool "$APPDIR" "$OUT"
else
  echo "Erro: comando 'appimagetool' ou 'linuxdeploy' não encontrado." >&2
  echo "  Instale AppImageKit / linuxdeploy, ou use o tarball." >&2
  echo "  AppDir preparado em: ${APPDIR}" >&2
  exit 1
fi

echo "OK: ${OUT}"
