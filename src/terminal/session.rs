//! Uma sessão PTY (shell filho) com emulador VT100 para tela e cursor.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use vt100::Parser as VtParser;

use super::selection::TerminalSelection;

pub const SCROLLBACK_LINES: usize = 10_000;
const CURSOR_BLINK_MS: u64 = 530;

pub struct TerminalSession {
    pub id: String,
    pub label: String,
    pub vt: VtParser,
    pub scroll_offset: usize,
    pub follow_tail: bool,
    pub cwd: PathBuf,
    cursor_blink_on: bool,
    last_blink: Instant,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
    output_rx: Receiver<Vec<u8>>,
    reader_thread: Option<JoinHandle<()>>,
    last_size: (u16, u16),
}

impl TerminalSession {
    pub fn spawn(id: String, cwd: PathBuf, cols: u16, rows: u16) -> std::io::Result<Self> {
        let cols = cols.max(1);
        let rows = rows.max(1);
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
            vt: VtParser::new(rows, cols, SCROLLBACK_LINES),
            scroll_offset: 0,
            follow_tail: true,
            cwd,
            cursor_blink_on: true,
            last_blink: Instant::now(),
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
            self.vt.process(&chunk);
            if self.follow_tail {
                self.scroll_offset = 0;
                self.vt.screen_mut().set_scrollback(0);
            }
        }
    }

    pub fn tick_cursor_blink(&mut self) {
        if self.last_blink.elapsed() >= Duration::from_millis(CURSOR_BLINK_MS) {
            self.cursor_blink_on = !self.cursor_blink_on;
            self.last_blink = Instant::now();
        }
    }

    pub fn apply_scrollback_view(&mut self) {
        self.vt
            .screen_mut()
            .set_scrollback(self.scroll_offset);
    }

    pub fn row_text(&self, row: u16, width: u16) -> String {
        let screen = self.vt.screen();
        let w = width as usize;
        let mut chars: Vec<char> = vec![' '; w];
        for col in 0..width {
            if let Some(cell) = screen.cell(row, col) {
                if let Some(ch) = cell.contents().chars().next() {
                    chars[col as usize] = ch;
                }
            }
        }
        chars.into_iter().collect()
    }

    pub fn cursor_display(&self) -> Option<(u16, u16, bool)> {
        let screen = self.vt.screen();
        if screen.hide_cursor() {
            return None;
        }
        let (row, col) = screen.cursor_position();
        Some((row, col, self.cursor_blink_on))
    }

    pub fn extract_selection(&self, selection: TerminalSelection) -> String {
        let (a, b) = selection.normalized();
        self.vt.screen().contents_between(
            a.line as u16,
            a.col as u16,
            b.line as u16,
            b.col as u16,
        )
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
        let _ = self.writer.flush();
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        let cols = cols.max(1);
        let rows = rows.max(1);
        if (cols, rows) == self.last_size {
            return;
        }
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
        self.vt.screen_mut().set_size(rows, cols);
        self.last_size = (cols, rows);
    }

    pub fn scroll_to_tail(&mut self) {
        self.scroll_offset = 0;
        self.follow_tail = true;
        self.vt.screen_mut().set_scrollback(0);
    }

    pub fn max_scroll_offset(&self) -> usize {
        SCROLLBACK_LINES.saturating_sub(1)
    }

    /// `delta > 0` rola para cima (histórico); `delta < 0` rola para baixo (fim/prompt).
    pub fn scroll_page(&mut self, delta: isize, _visible_height: usize) {
        if delta == 0 {
            return;
        }
        let max = self.max_scroll_offset();
        if delta < 0 && self.scroll_offset == 0 && self.follow_tail {
            return;
        }
        if delta > 0 && self.scroll_offset >= max {
            return;
        }
        let next = ((self.scroll_offset as isize) + delta).clamp(0, max as isize) as usize;
        self.scroll_offset = next;
        self.follow_tail = next == 0;
        self.vt.screen_mut().set_scrollback(next);
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.writer.flush();
        if let Some(handle) = self.reader_thread.take() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn screen_row(screen: &vt100::Screen, row: u16, width: u16) -> String {
        let w = width as usize;
        let mut chars: Vec<char> = vec![' '; w];
        for col in 0..width {
            if let Some(cell) = screen.cell(row, col) {
                if let Some(ch) = cell.contents().chars().next() {
                    chars[col as usize] = ch;
                }
            }
        }
        chars.into_iter().collect::<String>().trim_end().to_string()
    }

    #[test]
    fn vt100_carriage_return_overwrites_line_start() {
        let mut vt = VtParser::new(5, 40, 100);
        vt.process(b"Microsoft Windows\r\n(c) Microsoft\r\n\rC:\\> ");
        let screen = vt.screen();
        assert_eq!(screen_row(screen, 0, 40), "Microsoft Windows");
        assert_eq!(screen_row(screen, 1, 40), "(c) Microsoft");
        assert!(screen_row(screen, 2, 40).starts_with("C:\\>"));
        let (row, col) = screen.cursor_position();
        assert_eq!((row, col), (2, 5));
    }
}
