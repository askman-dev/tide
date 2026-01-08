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

    let elapsed = start.elapsed();
    logging::log_line(
        "INFO",
        &format!(
            "dir list: root={} depth={} entries={} ms={}",
            root.display(),
            depth,
            entries.len(),
            elapsed.as_millis()
        ),
    );
    logging::log_slow_op(
        "list_dir_entries",
        elapsed,
        &format!("root={} depth={depth}", root.display()),
    );

    entries
}

pub fn read_file_preview(path: &std::path::Path) -> std::io::Result<String> {
    let metadata = fs::metadata(path)?;
    let size = metadata.len();

    // Max size 1MB for preview, read first 100KB if larger
    const MAX_PREVIEW_SIZE: u64 = 1024 * 1024; // 1MB
    const TRUNCATE_SIZE: usize = 100 * 1024; // 100KB

    if size > MAX_PREVIEW_SIZE {
        use std::io::Read;
        let mut file = fs::File::open(path)?;
        let mut buffer = vec![0; TRUNCATE_SIZE];
        let n = file.read(&mut buffer)?;
        let mut content = String::from_utf8_lossy(&buffer[..n]).to_string();
        content.push_str("\n\n... (file truncated, too large to preview)");
        Ok(content)
    } else {
        fs::read_to_string(path)
    }
}
