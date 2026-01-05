use crate::services::TerminalSession;
use floem::ext_event::ExtSendTrigger;
use floem::reactive::RwSignal;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct WorkspaceTab {
    pub id: usize,
    pub name: RwSignal<String>,
    pub root: RwSignal<PathBuf>,
    pub file_tree: RwSignal<Vec<TreeEntry>>,
    pub git_status: RwSignal<Vec<String>>,
    pub terminal: RwSignal<Option<Arc<TerminalSession>>>,
    pub terminal_trigger: ExtSendTrigger,
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
