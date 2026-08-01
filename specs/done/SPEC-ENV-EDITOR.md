# SPEC — Editor de variáveis de ambiente

**Autor:** Cursor  
**Data:** 2026-07-31  
**Versão:** 1.0  
**Status:** done

## Objetivo

Painel direito estruturado (estilo Rapid Environment Editor) para ver, criar, editar e excluir variáveis de ambiente **persistentes** de Usuário e Sistema no Linux e no Windows.

## Persistência

| Escopo | Linux | Windows |
|--------|-------|---------|
| Usuário | `~/.config/environment.d/99-edit.conf` | `HKCU\Environment` |
| Sistema | `/etc/environment.d/99-edit.conf` | `HKLM\...\Session Manager\Environment` |

Formato Linux: linhas `NOME=valor` (sem `export`). Apenas o ficheiro `99-edit.conf` é gerido pelo `edit`.

## Elevação

- Linux sistema: modal `[ Privilégio necessário ]` → `pkexec` copia ficheiro temp para `/etc/environment.d/99-edit.conf`.
- Windows sistema: modal se sem admin; instruir a executar como administrador (v1).

## UX

- Menu **Ferramentas → Variáveis de ambiente…**; atalho **`Ctrl+Alt+V`** (evita `Ctrl+Shift+V`, Paste nos emuladores Linux).
- Split direito; lista `NOME | valor`; escopos Usuário / Sistema.
- Ações: Novo, Editar, Excluir, Salvar, Recarregar, Fechar.
- Confirmação em Excluir e ao trocar escopo com dirty.
- Não é painel de referência read-only; painel dedicado `EnvPane`.

## Fora de escopo

Tabela ASCII completa; `sudo -S` no TUI; editar `.bashrc`; aplicar env a processos já em execução.
