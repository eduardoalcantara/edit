//! Catálogo estático de configs comuns; só entradas cujo path existe.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigEntry {
    pub category: &'static str,
    pub name: &'static str,
    /// Path absoluto ou começado por `~/`.
    pub path: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub category: String,
    pub name: String,
    pub path: PathBuf,
    pub description: String,
}

/// Catálogo embutido (v1). Extensão pelo utilizador = fase 2.
pub const CATALOG: &[ConfigEntry] = &[
    // Rede
    ConfigEntry {
        category: "Rede",
        name: "hosts",
        path: "/etc/hosts",
        description: "Nomes locais e bloqueios DNS",
    },
    ConfigEntry {
        category: "Rede",
        name: "resolv.conf",
        path: "/etc/resolv.conf",
        description: "Servidores DNS do sistema",
    },
    ConfigEntry {
        category: "Rede",
        name: "hostname",
        path: "/etc/hostname",
        description: "Nome da máquina",
    },
    ConfigEntry {
        category: "Rede",
        name: "NetworkManager.conf",
        path: "/etc/NetworkManager/NetworkManager.conf",
        description: "Configuração do NetworkManager",
    },
    ConfigEntry {
        category: "Rede",
        name: "nftables.conf",
        path: "/etc/nftables.conf",
        description: "Regras de firewall nftables",
    },
    ConfigEntry {
        category: "Rede",
        name: "iptables.rules",
        path: "/etc/iptables/iptables.rules",
        description: "Regras iptables (se usadas)",
    },
    ConfigEntry {
        category: "Rede",
        name: "20-wired.network",
        path: "/etc/systemd/network/20-wired.network",
        description: "Exemplo systemd-networkd (cabo)",
    },
    // SSH
    ConfigEntry {
        category: "SSH",
        name: "sshd_config",
        path: "/etc/ssh/sshd_config",
        description: "Servidor SSH (sistema)",
    },
    ConfigEntry {
        category: "SSH",
        name: "ssh_config",
        path: "/etc/ssh/ssh_config",
        description: "Cliente SSH (sistema)",
    },
    ConfigEntry {
        category: "SSH",
        name: "config",
        path: "~/.ssh/config",
        description: "Cliente SSH do utilizador",
    },
    ConfigEntry {
        category: "SSH",
        name: "known_hosts",
        path: "~/.ssh/known_hosts",
        description: "Hosts SSH já aceites",
    },
    ConfigEntry {
        category: "SSH",
        name: "authorized_keys",
        path: "~/.ssh/authorized_keys",
        description: "Chaves autorizadas a entrar",
    },
    // Shells
    ConfigEntry {
        category: "Shell",
        name: "shells",
        path: "/etc/shells",
        description: "Shells válidos no sistema",
    },
    ConfigEntry {
        category: "Shell",
        name: ".bashrc",
        path: "~/.bashrc",
        description: "Bash interativo (utilizador)",
    },
    ConfigEntry {
        category: "Shell",
        name: ".bash_profile",
        path: "~/.bash_profile",
        description: "Login Bash (utilizador)",
    },
    ConfigEntry {
        category: "Shell",
        name: ".profile",
        path: "~/.profile",
        description: "Perfil de login genérico",
    },
    ConfigEntry {
        category: "Shell",
        name: ".zshrc",
        path: "~/.zshrc",
        description: "Zsh interativo (utilizador)",
    },
    ConfigEntry {
        category: "Shell",
        name: ".zprofile",
        path: "~/.zprofile",
        description: "Login Zsh (utilizador)",
    },
    ConfigEntry {
        category: "Shell",
        name: ".inputrc",
        path: "~/.inputrc",
        description: "Atalhos readline",
    },
    // Sistema
    ConfigEntry {
        category: "Sistema",
        name: "fstab",
        path: "/etc/fstab",
        description: "Montagens de discos no boot",
    },
    ConfigEntry {
        category: "Sistema",
        name: "locale.conf",
        path: "/etc/locale.conf",
        description: "Idioma e locale do sistema",
    },
    ConfigEntry {
        category: "Sistema",
        name: "environment",
        path: "/etc/environment",
        description: "Variáveis de ambiente (PAM)",
    },
    ConfigEntry {
        category: "Sistema",
        name: "99-edit.conf (user)",
        path: "~/.config/environment.d/99-edit.conf",
        description: "Env utilizador gerido pelo edit",
    },
    ConfigEntry {
        category: "Sistema",
        name: "99-edit.conf (system)",
        path: "/etc/environment.d/99-edit.conf",
        description: "Env sistema gerido pelo edit",
    },
    ConfigEntry {
        category: "Sistema",
        name: "sudoers",
        path: "/etc/sudoers",
        description: "Permissões sudo (cuidado)",
    },
    ConfigEntry {
        category: "Sistema",
        name: "sysctl.conf",
        path: "/etc/sysctl.conf",
        description: "Parâmetros do kernel",
    },
    ConfigEntry {
        category: "Sistema",
        name: "crontab",
        path: "/etc/crontab",
        description: "Agendamentos do sistema",
    },
    // Pacotes / boot
    ConfigEntry {
        category: "Pacotes",
        name: "pacman.conf",
        path: "/etc/pacman.conf",
        description: "Pacman (Arch e derivados)",
    },
    ConfigEntry {
        category: "Pacotes",
        name: "mirrorlist",
        path: "/etc/pacman.d/mirrorlist",
        description: "Mirrors pacman",
    },
    ConfigEntry {
        category: "Pacotes",
        name: "apt sources",
        path: "/etc/apt/sources.list",
        description: "Fontes APT (Debian/Ubuntu)",
    },
    ConfigEntry {
        category: "Boot",
        name: "grub",
        path: "/etc/default/grub",
        description: "Opções do GRUB",
    },
    ConfigEntry {
        category: "Boot",
        name: "mkinitcpio.conf",
        path: "/etc/mkinitcpio.conf",
        description: "Initramfs (Arch)",
    },
    ConfigEntry {
        category: "Systemd",
        name: "journald.conf",
        path: "/etc/systemd/journald.conf",
        description: "Logs do journald",
    },
    ConfigEntry {
        category: "Systemd",
        name: "logind.conf",
        path: "/etc/systemd/logind.conf",
        description: "Sessões e sleep (logind)",
    },
    // Apps / XDG
    ConfigEntry {
        category: "Apps",
        name: "mimeapps.list",
        path: "~/.config/mimeapps.list",
        description: "Aplicações padrão (MIME)",
    },
    ConfigEntry {
        category: "Apps",
        name: "user-dirs.dirs",
        path: "~/.config/user-dirs.dirs",
        description: "Pastas XDG (Desktop, Docs…)",
    },
    ConfigEntry {
        category: "Apps",
        name: "gitconfig",
        path: "~/.gitconfig",
        description: "Git global do utilizador",
    },
    ConfigEntry {
        category: "Apps",
        name: "edit.json",
        path: "~/.config/edit/edit.json",
        description: "Configuração deste editor",
    },
];

/// Expande `~/…` para `$HOME/…`.
pub fn expand_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    if path == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn path_exists_usable(path: &Path) -> bool {
    path.is_file()
}

/// Entradas do catálogo cujo path existe, ordenadas por categoria e nome.
pub fn list_existing_configs() -> Vec<ResolvedConfig> {
    let mut out: Vec<ResolvedConfig> = CATALOG
        .iter()
        .filter_map(|e| {
            let path = expand_path(e.path);
            if !path_exists_usable(&path) {
                return None;
            }
            Some(ResolvedConfig {
                category: e.category.to_string(),
                name: e.name.to_string(),
                path,
                description: e.description.to_string(),
            })
        })
        .collect();
    out.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_uses_home() {
        let p = expand_path("~/.bashrc");
        let s = p.to_string_lossy();
        assert!(!s.starts_with("~/"), "got {s}");
        assert!(s.ends_with(".bashrc") || s.ends_with("bashrc"));
    }

    #[test]
    fn catalog_nonempty() {
        assert!(CATALOG.len() >= 20);
    }

    #[test]
    fn list_existing_only_real_paths() {
        for item in list_existing_configs() {
            assert!(
                item.path.exists(),
                "listed missing path: {}",
                item.path.display()
            );
        }
    }
}
