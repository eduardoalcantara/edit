use std::io;
use std::time::Duration;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::editor::Editor;
use crate::events;
use crate::theme::ThemeId;
use crate::ui;

pub struct App {
    pub editor: Editor,
    pub theme: ThemeId,
    pub should_quit: bool,
    pub status_message: String,
    pub document_title: String,
    #[allow(dead_code)]
    pub side_panel_visible: bool,
    #[allow(dead_code)]
    pub bottom_terminal_visible: bool,
    pub mouse_enabled: bool,
}

impl App {
    pub fn new(mouse_enabled: bool) -> Self {
        let theme = ThemeId::Dark;
        let palette = theme.palette();
        Self {
            editor: Editor::new(&palette),
            theme,
            should_quit: false,
            status_message: "Pronto".to_string(),
            document_title: "Sem título".to_string(),
            side_panel_visible: false,
            bottom_terminal_visible: false,
            mouse_enabled,
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| ui::draw(frame, self))?;

            if events::poll(Duration::from_millis(50))? {
                let event = events::read()?;
                events::dispatch(self, event);
            }
        }

        Ok(())
    }
}
