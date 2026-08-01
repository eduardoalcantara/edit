#!/usr/bin/env bash
# Compila o edit em release.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_common.sh
source "${SCRIPT_DIR}/_common.sh"

cd "$ROOT"
echo "Compilando edit $(edit_version) (release)…"
cargo build --release
echo "OK: ${ROOT}/target/release/edit"
