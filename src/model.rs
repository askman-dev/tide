use crate::services::TerminalSession;
use floem::ext_event::ExtSendTrigger;
use floem::reactive::RwSignal;
use std::path::PathBuf;
use std::sync::Arc;

/// A single terminal pane with its own session
#[derive(Clone)]
pub struct TerminalPane {
    pub id: usize,
    pub session: RwSignal<Option<Arc<TerminalSession>>>,
    pub trigger: ExtSendTrigger,
    /// Flex ratio for width (1.0 = equal share with other panes)
    pub flex_ratio: RwSignal<f64>,
}

#[derive(Clone)]
pub struct WorkspaceTab {
    pub id: usize,
    pub name: RwSignal<String>,
    pub root: RwSignal<PathBuf>,
    pub file_tree: RwSignal<Vec<TreeEntry>>,
    pub git_status: RwSignal<Vec<String>>,
    /// Terminal panes (supports splits - multiple panes side by side)
    pub terminal_panes: RwSignal<Vec<TerminalPane>>,
    /// ID counter for creating new panes
    pub next_pane_id: RwSignal<usize>,
}

#[derive(Clone)]
pub struct TreeEntry {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}
