#!/usr/bin/env bash
# Gera dist/edit-<ver>-x86_64.tar.gz a partir do staging FHS.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_common.sh
source "${SCRIPT_DIR}/_common.sh"

"${SCRIPT_DIR}/stage.sh"

VER="$(edit_version)"
ARCH="$(uname -m)"
OUT="${DIST}/edit-${VER}-${ARCH}.tar.gz"

mkdir -p "$DIST"
tar -C "$STAGE" -czf "$OUT" .
echo "OK: ${OUT}"
