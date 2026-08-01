#!/usr/bin/env bash
# Gera .rpm (Fedora/RHEL) em dist/.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_common.sh
source "${SCRIPT_DIR}/_common.sh"

require_cmd rpmbuild "Instale rpm-build (Fedora) ou use o tarball."

"${SCRIPT_DIR}/stage.sh"

VER="$(edit_version)"
RPM_TOP="${DIST}/rpmbuild"
rm -rf "$RPM_TOP"
mkdir -p "${RPM_TOP}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

SPEC="${RPM_TOP}/SPECS/edit.spec"
sed -e "s|@VERSION@|${VER}|g" \
    -e "s|@STAGE@|${STAGE}|g" \
    "${ROOT}/packaging/fedora/edit.spec.in" > "$SPEC"

rpmbuild --define "_topdir ${RPM_TOP}" -bb "$SPEC"

shopt -s nullglob
RPMS=( "${RPM_TOP}/RPMS"/*/*.rpm )
if [[ ${#RPMS[@]} -eq 0 ]]; then
  echo "Erro: nenhum .rpm gerado." >&2
  exit 1
fi
for rpm in "${RPMS[@]}"; do
  cp -f "$rpm" "$DIST"/
  echo "OK: ${DIST}/$(basename "$rpm")"
done
