# SPEC — Painel Configurações Linux (Editor 2)

**Status:** done  
**Data:** 2026-08-01  
**Autor:** Cursor  
**Plataforma:** Unix/Linux apenas

## Objetivo

Listar ficheiros de configuração comuns do Linux no **painel direito** (Editor 2). Enter ou clique abre o ficheiro no **Editor 1**, para manutenção sem navegar em vários diretórios.

## Comportamento

| Ação | Detalhe |
|------|---------|
| Abrir | **Ferramentas → Configurações do sistema…** ou **`Ctrl+Alt+L`** |
| Lista | Catálogo embutido; só entradas cujo path **existe** (expandir `~/`) |
| Linha | Categoria/nome + path \| descrição breve |
| Ativar | **Enter** ou **clique** → `focus` esquerda + `open_path`; painel direito permanece |
| Fechar | Esc / Fechar / `Alt+C`; botão **Ler** (`Alt+L`) refresca a lista |
| Exclusão | Fecha env/referência ao abrir; vice-versa |

Windows: item e atalho ausentes.

## Catálogo v1

Módulo estático `src/linux_configs/catalog.rs` (rede, SSH, shells, sistema, pacotes, boot, systemd, apps). Extensão pelo utilizador = fase 2.

Gravação de ficheiros root-owned segue o save atual (sem `pkexec` automático nesta feature).

## Arquitetura

| Peça | Ficheiro |
|------|----------|
| Catálogo | `src/linux_configs/catalog.rs` |
| Painel | `src/linux_configs/pane.rs` |
| App | `src/app_linux_configs.rs` |
| Split | `EditorSplit.linux_configs` |
| UI | `src/ui/layers/editor.rs`, compositor Esc |
| Menu | `ActionId::LinuxConfigs`, `tools_menu` |

## Testes

- `expand_path`, `list_existing_configs`, chord `Ctrl+Alt+L`, catálogo não vazio

Suite: `cargo test -- --test-threads=1`.
