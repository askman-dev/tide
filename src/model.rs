use crate::services::TerminalSession;
use floem::ext_event::ExtSendTrigger;
use floem::reactive::RwSignal;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// A single terminal pane with its own session
#[derive(Clone)]
pub struct TerminalPane {
    pub id: usize,
    pub session: RwSignal<Option<Arc<TerminalSession>>>,
    pub trigger: ExtSendTrigger,
    /// Flex ratio for width (1.0 = equal share with other panes)
    pub flex_ratio: RwSignal<f64>,
    /// Pane title (dynamically updated from command)
    pub title: RwSignal<String>,
    /// Signal to programmatically request focus
    pub should_focus: RwSignal<bool>,
    /// Buffer for cross-thread title updates
    pub title_buffer: Arc<Mutex<Option<String>>>,
}

#[derive(Clone)]
pub struct WorkspaceTab {
    pub id: usize,
    pub name: RwSignal<String>,
    pub root: RwSignal<PathBuf>,
    pub file_tree: RwSignal<Vec<TreeEntry>>,
    pub git_status: RwSignal<Vec<String>>,
    /// Buffer for cross-thread git status updates
    pub git_status_buffer: Arc<Mutex<Option<Vec<String>>>>,
    /// Editor tabs for this workspace
    pub editor_tabs: RwSignal<Vec<EditorTab>>,
    pub active_editor_tab: RwSignal<Option<usize>>,
    /// Focused terminal pane ID
    pub focused_pane_id: RwSignal<Option<usize>>,
    /// Terminal panes (supports splits - multiple panes side by side)
    pub terminal_panes: RwSignal<Vec<TerminalPane>>,
    /// ID counter for creating new panes
    pub next_pane_id: RwSignal<usize>,
    /// ID counter for creating new editor tabs
    pub next_editor_tab_id: RwSignal<usize>,
}

#[derive(Clone)]
pub struct EditorTab {
    pub id: usize,
    pub path: PathBuf,
    pub name: String,
    pub is_pinned: RwSignal<bool>,
    pub content: String,
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