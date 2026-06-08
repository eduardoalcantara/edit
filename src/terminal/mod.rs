mod input;
mod render;
mod scrollback;
mod selection;
mod session;
mod workspace;

pub use input::key_event_to_pty_bytes;
pub use render::{
    editor_content_in_shell, paint_terminal_panel, render_terminal_bottom_row,
    render_terminal_divider, sidebar_cols_for_shell, terminal_panel_outer,
    clamp_terminal_panel_rows, terminal_reserved_rows, TERMINAL_PANEL_ROWS_DEFAULT,
    TERMINAL_PANEL_ROWS_MAX, TERMINAL_PANEL_ROWS_MIN,
};
pub use selection::{mouse_to_coord, TerminalSelection};
pub use workspace::{
    default_spawn_cwd, layout_terminal_panel, sidebar_button_help,
    sidebar_click, terminal_split_col, SidebarClick, TerminalPanelLayout, TerminalWorkspace,
};
