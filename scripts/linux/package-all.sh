#!/usr/bin/env bash
# Orquestra todos os pacotes Linux disponíveis nesta máquina.
# Formatos sem ferramenta instalada são pulados com aviso (não aborta o restante).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_common.sh
source "${SCRIPT_DIR}/_common.sh"

mkdir -p "$DIST"
ensure_release_binary

run_one() {
  local name="$1"
  local script="$2"
  echo ""
  echo "=== ${name} ==="
  if bash "$script"; then
    echo ">>> ${name}: OK"
  else
    echo ">>> ${name}: FALHOU ou ferramenta ausente (continuando)" >&2
  fi
}

run_one "tarball" "${SCRIPT_DIR}/package-tarball.sh"
run_one "deb" "${SCRIPT_DIR}/package-deb.sh"
run_one "rpm" "${SCRIPT_DIR}/package-rpm.sh"
run_one "arch" "${SCRIPT_DIR}/package-arch.sh"
run_one "appimage" "${SCRIPT_DIR}/package-appimage.sh"

echo ""
echo "Artefatos em ${DIST}:"
ls -lh "$DIST"/*.{tar.gz,deb,rpm,AppImage,pkg.tar.zst,pkg.tar.xz} 2>/dev/null || ls -lh "$DIST"
