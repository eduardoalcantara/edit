mod tab;
mod workspace;

pub use tab::{create_tab_from_defaults, next_novo_counter, novo_display_name, snapshot_path, Tab};
pub use workspace::{swap_active_with_tab, PromptReason, TabSortStrategy, Workspace, WorkspaceAction};

use std::sync::atomic::{AtomicU64, Ordering};

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn new_session_id() -> String {
    let n = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("tab-{n}")
}
