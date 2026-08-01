# Editor de variáveis de ambiente

**Autor:** Cursor  
**Data:** 2026-07-31  
**Versão:** 1.0  
**Spec:** `specs/done/SPEC-ENV-EDITOR.md`

## Objetivo

Painel direito estruturado (estilo Rapid Environment Editor) para ver, criar, editar e excluir variáveis de ambiente **persistentes** de Usuário e Sistema. O alvo é o que novos terminais e apps herdam após login/sessão — não o `environ` só do processo `edit` atual.

## Como abrir

| Caminho | Ação |
|---------|------|
| Menu **Ferramentas → Variáveis de ambiente…** | `Alt+T`, depois o item |
| Atalho | **`Ctrl+Alt+V`** (não `Ctrl+Shift+V`: emuladores Linux usam isso para Paste e o chord não chega ao `edit`) |

Título na borda: `[ Variáveis — Usuário ]` ou `[ Variáveis — Sistema ]`.

## Persistência

| Escopo | Linux | Windows |
|--------|-------|---------|
| **Usuário** | Gravação: `~/.config/environment.d/99-edit.conf` | `HKCU\Environment` |
| **Sistema** | Gravação: `/etc/environment.d/99-edit.conf` | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` |

### Leitura (Linux)

| Escopo | Fontes |
|--------|--------|
| **Usuário** | Só `~/.config/environment.d/*.conf` (persistente; análogo a `HKCU\Environment`) |
| **Sistema** | `/etc/environment` + `/etc/environment.d/*.conf` |

- **Não** misturar `systemctl --user show-environment` nem o environ do processo `edit` na lista editável — isso puxava a sessão live (Cursor AppImage, GNOME, RUSTUP…) e, ao gravar, poluía `99-edit.conf`.
- Overrides `EDIT_ENV_*` **só em testes**.
- Formato: linhas `NOME=valor` (sem `export`). Valores com `$''` (quoting do systemd) são descodificados na leitura.
- Só o ficheiro `99-edit.conf` é **escrito** pelo `edit`.
- Após salvar Usuário: **`systemctl --user set-environment KEY=VAL`** só para as variáveis do ficheiro — **nunca** `import-environment` sem argumentos.
- Novas apps/terminais herdam as variáveis; a sessão gráfica já aberta pode exigir logout/login.

## Elevação (Sistema)

1. Em **Salvar** no escopo Sistema, o `edit` mostra o modal `[ Privilégio necessário ]`.
2. **Continuar:** suspende brevemente a TUI e grava via **`pkexec`** (PolicyKit) no Linux, ou exige admin no Windows (HKLM).
3. A senha/autenticação é do SO (polkit/UAC), **não** um campo de senha dentro do TUI.
4. Sem agente polkit (ex. SSH puro): mensagem de erro; use o escopo Usuário ou instale `polkit`.

## UX do painel

- Lista tabular `NOME | valor` (valor longo truncado; edição em modal).
- Barra de botões estilo modal: **[Origem] [Nova] [Mudar] [Remover] [Gravar] [Ler] [Fechar]** com **Alt+O/N/M/R/G/L/C** (padrão Windows; não colidem com menus Alt+A/E/X/F/S/T/H).
- Tab também alterna Usuário ↔ Sistema (confirma se houver alterações não salvas).
- Confirmação obrigatória em Remover (botões **[Sim] [Cancelar]**).
- Valores em claro na v1 (aviso no help): sem mascarar secrets ainda.
- Fechar restaura a aba que estava no split direito (stash), como o painel de referência.
- Scroll do mouse na lista; clique nos botões ou nas linhas.
## Nomes válidos

Padrão: `[A-Za-z_][A-Za-z0-9_]*`.

## Fora de escopo (v1)

- Editar `.bashrc` / PATH por segmentos.
- Aplicar env a processos já em execução (Cursor, etc.) sem restart.
- `sudo -S` dentro do TUI.
- Coluna “só em memória” do processo `edit` (fase 2 possível).

## Código

| Módulo | Papel |
|--------|-------|
| `src/env_editor/` | Parse/serialize, load/save, validação |
| `src/env_pane.rs` / `src/app_env_pane.rs` | Painel e dispatch |
| `src/modal/env_var.rs` | Modal Novo/Editar |

Testes: overrides `EDIT_ENV_USER_CONF` / `EDIT_ENV_SYSTEM_CONF` para paths temporários.
