use crate::logging;
use crate::model::TreeEntry;
use std::fs;
use std::path::Path;
use std::time::Instant;

pub fn build_tree_entries(root: &Path, _max_depth: usize) -> Vec<TreeEntry> {
    list_dir_entries(root, 0)
}

pub fn list_dir_entries(root: &Path, depth: usize) -> Vec<TreeEntry> {
    let start = Instant::now();
    let read_dir = match fs::read_dir(root) {
        Ok(read_dir) => read_dir,
        Err(_) => return Vec::new(),
    };
    let mut children = read_dir.filter_map(|entry| entry.ok()).collect::<Vec<_>>();
    children.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());

    let entries = children
        .into_iter()
        .map(|entry| {
            let path = entry.path();
            let is_dir = path.is_dir();
            let name = entry.file_name().to_string_lossy().to_string();
            TreeEntry {
                id: path.to_string_lossy().to_string(),
                path,
                name,
                depth,
                is_dir,
                expanded: false,
            }
        })
        .collect::<Vec<_>>();

    logging::log_slow_op(
        "list_dir_entries",
        start.elapsed(),
        &format!("root={} depth={depth}", root.display()),
    );

    entries
}
