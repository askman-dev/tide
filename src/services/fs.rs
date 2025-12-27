use crate::model::TreeEntry;
use std::fs;
use std::path::Path;

pub fn build_tree_entries(root: &Path, max_depth: usize) -> Vec<TreeEntry> {
    let mut entries = Vec::new();
    let root_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace")
        .to_string();
    entries.push(TreeEntry {
        id: root.to_string_lossy().to_string(),
        name: root_name,
        depth: 0,
        is_dir: true,
    });
    visit_dir(root, 1, max_depth, &mut entries);
    entries
}

fn visit_dir(path: &Path, depth: usize, max_depth: usize, entries: &mut Vec<TreeEntry>) {
    if depth > max_depth {
        return;
    }
    let read_dir = match fs::read_dir(path) {
        Ok(read_dir) => read_dir,
        Err(_) => return,
    };
    let mut children = read_dir.filter_map(|entry| entry.ok()).collect::<Vec<_>>();
    children.sort_by_key(|entry| entry.file_name().to_string_lossy().to_lowercase());
    for entry in children {
        let path = entry.path();
        let is_dir = path.is_dir();
        let name = entry.file_name().to_string_lossy().to_string();
        entries.push(TreeEntry {
            id: path.to_string_lossy().to_string(),
            name,
            depth,
            is_dir,
        });
        if is_dir {
            visit_dir(&path, depth + 1, max_depth, entries);
        }
    }
}
