use std::path::PathBuf;

#[derive(Clone)]
pub struct WorkspaceTab {
    pub id: usize,
    pub name: String,
    pub root: PathBuf,
    pub file_tree: Vec<TreeEntry>,
    pub git_status: Vec<String>,
}

#[derive(Clone)]
pub struct TreeEntry {
    pub id: String,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub expanded: bool,
}
