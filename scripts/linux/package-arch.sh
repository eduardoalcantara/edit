#!/usr/bin/env bash
# Gera pacote Arch (makepkg) em dist/.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_common.sh
source "${SCRIPT_DIR}/_common.sh"

require_cmd makepkg "Instale base-devel (Arch) ou use o tarball."

"${SCRIPT_DIR}/package-tarball.sh"

VER="$(edit_version)"
ARCH_DIR="${DIST}/arch-build"
rm -rf "$ARCH_DIR"
mkdir -p "$ARCH_DIR"

# Fonte: tarball FHS já gerado
TARBALL="${DIST}/edit-${VER}-$(uname -m).tar.gz"
cp -f "$TARBALL" "${ARCH_DIR}/edit-fhs.tar.gz"

sed -e "s|@VERSION@|${VER}|g" \
  "${ROOT}/packaging/arch/PKGBUILD.in" > "${ARCH_DIR}/PKGBUILD"

# Extrair layout esperado pelo PKGBUILD: conteúdo do tar (usr/...)
# makepkg extrai source para $srcdir; nosso tar tem usr/ na raiz.
(
  cd "$ARCH_DIR"
  makepkg -f --nodeps
)

shopt -s nullglob
PKGS=( "${ARCH_DIR}"/*.pkg.tar.* )
if [[ ${#PKGS[@]} -eq 0 ]]; then
  echo "Erro: nenhum pacote Arch gerado." >&2
  exit 1
fi
for pkg in "${PKGS[@]}"; do
  cp -f "$pkg" "$DIST"/
  echo "OK: ${DIST}/$(basename "$pkg")"
done
