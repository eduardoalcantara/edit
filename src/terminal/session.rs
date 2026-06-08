//! Uma sessão PTY (shell filho).

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

use super::scrollback::Scrollback;

pub struct TerminalSession {
    pub id: String,
    pub label: String,
    pub scrollback: Scrollback,
    pub scroll_offset: usize,
    pub follow_tail: bool,
    pub cwd: PathBuf,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
    output_rx: Receiver<Vec<u8>>,
    reader_thread: Option<JoinHandle<()>>,
    last_size: (u16, u16),
}

impl TerminalSession {
    pub fn spawn(id: String, cwd: PathBuf, cols: u16, rows: u16) -> std::io::Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let (shell, label) = default_shell();
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(&cwd);
        #[cfg(windows)]
        {
            let _ = &shell;
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let (tx, output_rx) = mpsc::channel();
        let reader_thread = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            id,
            label,
            scrollback: Scrollback::new(),
            scroll_offset: 0,
            follow_tail: true,
            cwd,
            master: pair.master,
            child,
            writer,
            output_rx,
            reader_thread: Some(reader_thread),
            last_size: (cols, rows),
        })
    }

    pub fn drain_output(&mut self) {
        while let Ok(chunk) = self.output_rx.try_recv() {
            self.scrollback.push_bytes(&chunk);
            if self.follow_tail {
                self.scroll_offset = 0;
            }
        }
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
        let _ = self.writer.flush();
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        if cols == 0 || rows == 0 {
            return;
        }
        if (cols, rows) == self.last_size {
            return;
        }
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
        self.last_size = (cols, rows);
    }

    pub fn scroll_to_tail(&mut self) {
        self.scroll_offset = 0;
        self.follow_tail = true;
    }

    /// `delta > 0` rola para cima (histórico); `delta < 0` rola para baixo (fim/prompt).
    pub fn scroll_page(&mut self, delta: isize, visible_height: usize) {
        if delta == 0 || visible_height == 0 {
            return;
        }
        let max = self.scrollback.max_scroll_offset(visible_height);
        if delta < 0 && self.scroll_offset == 0 && self.follow_tail {
            return;
        }
        if delta > 0 && self.scroll_offset >= max {
            return;
        }
        let next = ((self.scroll_offset as isize) + delta).clamp(0, max as isize) as usize;
        self.scroll_offset = next;
        if self.scroll_offset == 0 {
            self.follow_tail = true;
        } else {
            self.follow_tail = false;
        }
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.writer.flush();
        if let Some(handle) = self.reader_thread.take() {
            // Não bloquear a UI: no Windows o join pode travar se o ConPTY não fechar o read.
            thread::spawn(move || {
                let _ = handle.join();
            });
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.kill();
    }
}

fn default_shell() -> (String, String) {
    #[cfg(windows)]
    {
        if let Ok(comspec) = std::env::var("COMSPEC") {
            let label = Path::new(&comspec)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("cmd")
                .to_string();
            return (comspec, label);
        }
        return ("cmd.exe".to_string(), "cmd".to_string());
    }
    #[cfg(not(windows))]
    {
        if let Ok(shell) = std::env::var("SHELL") {
            let label = Path::new(&shell)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("sh")
                .to_string();
            return (shell, label);
        }
        return ("/bin/sh".to_string(), "sh".to_string());
    }
}

pub fn shell_label_for_spawn() -> String {
    default_shell().1
}

