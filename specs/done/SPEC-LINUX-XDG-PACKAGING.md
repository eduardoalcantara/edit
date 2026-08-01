# SPEC — Linux XDG + freedesktop + pacotes

**Autor:** Cursor  
**Data:** 2026-07-31  
**Versão:** 1.0  
**Status:** done

## Objetivo

No Linux, gravar config e sessão segundo XDG Base Directory; integrar o lançador via freedesktop (`.desktop` + ícone); fornecer scripts que geram artefatos de distribuição (tarball, AppImage, Arch, deb, rpm).

## Paths de dados

| Modo | Config | Sessão (abas / undo) |
|------|--------|----------------------|
| Linux global | `$XDG_CONFIG_HOME/edit/edit.json` (fallback `~/.config/edit/edit.json`) | `$XDG_STATE_HOME/edit/.edit-session/` (fallback `~/.local/state/edit/.edit-session/`) |
| Windows global | pasta do `edit.exe` / `edit.json` | pasta do exe / `.edit-session/` |
| `--workspace` | `./.edit/.edit.workspace` | `./.edit/.edit-session/` |

## Migração (Linux, uma vez)

Se o `edit.json` XDG ainda não existir:

1. Copiar `edit.json` ao lado do executável, se houver.
2. Senão, migrar recentes de `.edit/recent.json` / `.editor-linux/recent.json` (cwd).
3. Se existir `.edit-session/` ao lado do exe e o state XDG estiver vazio, copiar para o state XDG.

## Freedesktop

- `edit.desktop` com `Terminal=true`, `Icon=edit`, categorias `Utility;TextEditor;`.
- Ícone hicolor (SVG scalable + PNG 48/128).
- Instalação: `/usr/bin/edit`, `/usr/share/applications/`, `/usr/share/icons/hicolor/…`.

## Pacotes

Scripts em `scripts/linux/` geram em `dist/`:

- `edit-<ver>-x86_64.tar.gz`
- `edit-<ver>-x86_64.AppImage`
- pacote Arch (`makepkg`)
- `.deb` (Debian/Ubuntu)
- `.rpm` (Fedora)

## Fora de escopo

Assinatura GPG, AUR/COPR/PPA, Flatpak/Snap, ícone na barra do emulador de terminal.
