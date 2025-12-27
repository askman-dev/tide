use std::path::Path;
use std::process::Command;

pub fn git_status_entries(root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(root)
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut lines: Vec<String> = stdout
                .lines()
                .map(|line| line.trim_end().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            if lines.is_empty() {
                lines.push("Clean working tree".to_string());
            }
            lines
        }
        _ => vec!["Not a git repository".to_string()],
    }
}
