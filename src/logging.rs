use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init() {
    let log_dir = log_dir();
    if let Err(err) = fs::create_dir_all(&log_dir) {
        eprintln!("log init failed: {err}");
        return;
    }

    let log_path = log_dir.join(format!("tide-{}.log", timestamp()));
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(file) => {
            let _ = LOG_PATH.set(log_path);
            let _ = LOG_FILE.set(Mutex::new(file));
            log_line("INFO", "log started");
        }
        Err(err) => eprintln!("log file open failed: {err}"),
    }

    std::panic::set_hook(Box::new(|info| {
        log_line("PANIC", &format!("{info}"));
    }));
}

pub fn log_path() -> Option<PathBuf> {
    LOG_PATH.get().cloned()
}

pub fn log_line(level: &str, message: &str) {
    let Some(lock) = LOG_FILE.get() else {
        return;
    };
    if let Ok(mut file) = lock.lock() {
        let _ = writeln!(file, "[{}] {}", level, message);
    }
}

fn log_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join("Library/Logs/Tide");
        }
    }
    PathBuf::from("logs")
}

fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-{:03}", now.as_secs(), now.subsec_millis())
}
