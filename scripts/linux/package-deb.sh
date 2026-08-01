#!/usr/bin/env bash
# Gera .deb (Debian/Ubuntu) em dist/.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_common.sh
source "${SCRIPT_DIR}/_common.sh"

require_cmd dpkg-deb "Instale dpkg (Debian/Ubuntu) ou use o tarball."

"${SCRIPT_DIR}/stage.sh"

VER="$(edit_version)"
ARCH_DEB="amd64"
case "$(uname -m)" in
  x86_64|amd64) ARCH_DEB="amd64" ;;
  aarch64|arm64) ARCH_DEB="arm64" ;;
  *) ARCH_DEB="$(uname -m)" ;;
esac

DEB_ROOT="${DIST}/deb-root"
rm -rf "$DEB_ROOT"
mkdir -p "${DEB_ROOT}/DEBIAN"
cp -a "${STAGE}/." "${DEB_ROOT}/"

SIZE_KB="$(du -sk "${DEB_ROOT}" | awk '{print $1}')"
sed -e "s/^Architecture:.*/Architecture: ${ARCH_DEB}/" \
  "${ROOT}/packaging/debian/control.in" > "${DEB_ROOT}/DEBIAN/control"
{
  echo "Version: ${VER}"
  echo "Installed-Size: ${SIZE_KB}"
} >> "${DEB_ROOT}/DEBIAN/control"

# control.in already has Architecture; rewrite cleanly
{
  echo "Package: edit"
  echo "Version: ${VER}"
  echo "Section: editors"
  echo "Priority: optional"
  echo "Architecture: ${ARCH_DEB}"
  echo "Depends: libc6"
  echo "Recommends: xdg-utils"
  echo "Maintainer: edit contributors <edit@localhost>"
  echo "Homepage: https://github.com/"
  echo "Installed-Size: ${SIZE_KB}"
  echo "Description: Editor de terminal estilo Turbo Vision"
  echo " Editor TUI com menus Turbo Vision, núcleo ropey, abas,"
  echo " terminal integrado e proteção contra perda de trabalho."
} > "${DEB_ROOT}/DEBIAN/control"

OUT="${DIST}/edit_${VER}_${ARCH_DEB}.deb"
dpkg-deb --root-owner-group --build "$DEB_ROOT" "$OUT"
echo "OK: ${OUT}"
