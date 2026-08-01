#!/usr/bin/env bash
# Funções compartilhadas pelos scripts de empacotamento Linux.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST="${ROOT}/dist"
STAGE="${DIST}/stage"
PACKAGING="${ROOT}/packaging/linux"

edit_version() {
  grep -m1 '^version' "${ROOT}/Cargo.toml" | sed -E 's/.*"([^"]+)".*/\1/'
}

require_cmd() {
  local cmd="$1"
  local hint="${2:-}"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Erro: comando '${cmd}' não encontrado." >&2
    if [[ -n "$hint" ]]; then
      echo "  ${hint}" >&2
    fi
    exit 1
  fi
}

ensure_release_binary() {
  local bin="${ROOT}/target/release/edit"
  if [[ ! -x "$bin" ]]; then
    echo "Binário release ausente; compilando…"
    (cd "$ROOT" && cargo build --release)
  fi
  if [[ ! -x "$bin" ]]; then
    echo "Erro: falha ao obter ${bin}" >&2
    exit 1
  fi
}
