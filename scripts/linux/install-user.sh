#!/usr/bin/env bash
# Instala o binário `edit` em ~/.local/bin (PATH típico do utilizador).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=_common.sh
source "${SCRIPT_DIR}/_common.sh"

ensure_release_binary

DEST_DIR="${HOME}/.local/bin"
DEST="${DEST_DIR}/edit"
mkdir -p "${DEST_DIR}"
install -m 755 "${ROOT}/target/release/edit" "${DEST}"

echo "Instalado: ${DEST}"
if command -v edit >/dev/null 2>&1; then
  echo "OK: o comando 'edit' está no PATH ($(command -v edit))."
else
  echo "Aviso: ~/.local/bin ainda não está no PATH desta sessão."
  echo "  Adicione ao shell, por exemplo no ~/.bashrc:"
  echo '  export PATH="$HOME/.local/bin:$PATH"'
  echo "  Depois: source ~/.bashrc   # ou abra um terminal novo"
fi
echo "Teste: edit --help"
