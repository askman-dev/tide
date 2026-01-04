use crate::logging;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub fn git_status_entries(root: &Path) -> Vec<String> {
    let start = Instant::now();
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
            logging::log_slow_op(
                "git status",
                start.elapsed(),
                &format!("root={}", root.display()),
            );
            lines
        }
        _ => {
            logging::log_slow_op(
                "git status",
                start.elapsed(),
                &format!("root={}", root.display()),
            );
            vec!["Not a git repository".to_string()]
        }
    }
}
