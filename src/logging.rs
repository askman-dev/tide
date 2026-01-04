use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const BREADCRUMB_CAP: usize = 64;
const SLOW_RENDER_MS: u64 = 50;
const SLOW_OP_MS: u64 = 250;
const SLOW_UI_EVENT_MS: u64 = 50;

static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static LAST_HEARTBEAT_MS: OnceLock<AtomicU64> = OnceLock::new();
static HEARTBEAT_STALE: OnceLock<AtomicBool> = OnceLock::new();
static BREADCRUMBS: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
static LAST_RENDER_MS: OnceLock<AtomicU64> = OnceLock::new();
static LAST_RENDER_AT_MS: OnceLock<AtomicU64> = OnceLock::new();
static LAST_RENDER_CELLS: OnceLock<AtomicU64> = OnceLock::new();
static LAST_RENDER_COLS: OnceLock<AtomicU64> = OnceLock::new();
static LAST_RENDER_ROWS: OnceLock<AtomicU64> = OnceLock::new();
static UI_THREAD_LABEL: OnceLock<String> = OnceLock::new();

pub fn init() {
    let current = std::thread::current();
    let ui_label = match current.name() {
        Some(name) => format!("{name}/{:?}", current.id()),
        None => format!("{:?}", current.id()),
    };
    let _ = UI_THREAD_LABEL.set(ui_label);

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
        log_line("PANIC", &format!("Caught panic: {info}"));
        let backtrace = std::backtrace::Backtrace::force_capture();
        log_line("PANIC", &format!("Backtrace:\n{backtrace}"));
    }));

    touch_heartbeat();
}

pub fn log_path() -> Option<PathBuf> {
    LOG_PATH.get().cloned()
}

pub fn log_line(level: &str, message: &str) {
    let Some(lock) = LOG_FILE.get() else {
        return;
    };
    if let Ok(mut file) = lock.lock() {
        let ts = timestamp_ms();
        let thread = thread_label();
        if message.is_empty() {
            let _ = writeln!(file, "[{ts}] [{thread}] [{level}]");
        } else {
            for line in message.lines() {
                let _ = writeln!(file, "[{ts}] [{thread}] [{level}] {line}");
            }
        }
        let _ = file.flush();
    }
}

pub fn breadcrumb(message: impl Into<String>) {
    let entry = format!("{} [{}] {}", timestamp_ms(), thread_label(), message.into());
    let store = BREADCRUMBS.get_or_init(|| Mutex::new(VecDeque::new()));
    if let Ok(mut buffer) = store.try_lock() {
        if buffer.len() == BREADCRUMB_CAP {
            buffer.pop_front();
        }
        buffer.push_back(entry);
    }
}

pub fn dump_breadcrumbs(reason: &str) {
    log_line("WARN", &format!("breadcrumbs: {reason}"));
    let store = BREADCRUMBS.get_or_init(|| Mutex::new(VecDeque::new()));
    if let Ok(buffer) = store.try_lock() {
        for entry in buffer.iter() {
            log_line("WARN", &format!("breadcrumb: {entry}"));
        }
    } else {
        log_line("WARN", "breadcrumb buffer locked");
    }
}

pub fn record_terminal_render(duration: Duration, cell_count: usize, cols: u16, rows: u16) {
    let now = now_millis();
    let ms = duration.as_millis() as u64;
    LAST_RENDER_MS
        .get_or_init(|| AtomicU64::new(0))
        .store(ms, Ordering::Relaxed);
    LAST_RENDER_AT_MS
        .get_or_init(|| AtomicU64::new(0))
        .store(now, Ordering::Relaxed);
    LAST_RENDER_CELLS
        .get_or_init(|| AtomicU64::new(0))
        .store(cell_count as u64, Ordering::Relaxed);
    LAST_RENDER_COLS
        .get_or_init(|| AtomicU64::new(0))
        .store(cols as u64, Ordering::Relaxed);
    LAST_RENDER_ROWS
        .get_or_init(|| AtomicU64::new(0))
        .store(rows as u64, Ordering::Relaxed);

    if ms >= SLOW_RENDER_MS {
        log_line(
            "WARN",
            &format!(
                "slow terminal render: {ms}ms cells={cell_count} grid={cols}x{rows}"
            ),
        );
    }
}

pub fn log_slow_op(op: &str, elapsed: Duration, detail: &str) {
    let ms = elapsed.as_millis() as u64;
    if ms >= SLOW_OP_MS {
        log_line("WARN", &format!("slow op: {op} {ms}ms {detail}"));
    }
}

pub fn measure_ui_event<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    let ms = start.elapsed().as_millis() as u64;
    if ms >= SLOW_UI_EVENT_MS {
        log_line("WARN", &format!("slow ui event: {label} {ms}ms"));
    }
    result
}

pub fn touch_heartbeat() {
    let now = now_millis();
    let heartbeat = LAST_HEARTBEAT_MS.get_or_init(|| AtomicU64::new(now));
    heartbeat.store(now, Ordering::Relaxed);
    let _ = HEARTBEAT_STALE.get_or_init(|| AtomicBool::new(false));
}

pub fn check_heartbeat(stale_after: Duration) {
    let now = now_millis();
    let last = LAST_HEARTBEAT_MS
        .get_or_init(|| AtomicU64::new(now))
        .load(Ordering::Relaxed);
    let stale_threshold_ms = stale_after.as_millis() as u64;
    let elapsed_ms = now.saturating_sub(last);
    let is_stale = elapsed_ms >= stale_threshold_ms;
    let stale_state = HEARTBEAT_STALE.get_or_init(|| AtomicBool::new(false));
    let was_stale = stale_state.swap(is_stale, Ordering::Relaxed);

    if is_stale && !was_stale {
        log_hang_context(elapsed_ms);
    } else if !is_stale && was_stale {
        log_line("INFO", "UI heartbeat restored");
    }
}

fn log_hang_context(elapsed_ms: u64) {
    log_line(
        "WARN",
        &format!("UI heartbeat stale for {elapsed_ms}ms (possible not responding)"),
    );
    if let Some(ui_thread) = UI_THREAD_LABEL.get() {
        log_line("WARN", &format!("ui thread: {ui_thread}"));
    }

    let last_render_ms = LAST_RENDER_MS
        .get_or_init(|| AtomicU64::new(0))
        .load(Ordering::Relaxed);
    let last_render_at = LAST_RENDER_AT_MS
        .get_or_init(|| AtomicU64::new(0))
        .load(Ordering::Relaxed);
    if last_render_ms > 0 && last_render_at > 0 {
        let render_age = now_millis().saturating_sub(last_render_at);
        let cells = LAST_RENDER_CELLS
            .get_or_init(|| AtomicU64::new(0))
            .load(Ordering::Relaxed);
        let cols = LAST_RENDER_COLS
            .get_or_init(|| AtomicU64::new(0))
            .load(Ordering::Relaxed);
        let rows = LAST_RENDER_ROWS
            .get_or_init(|| AtomicU64::new(0))
            .load(Ordering::Relaxed);
        log_line(
            "WARN",
            &format!(
                "last render: {last_render_ms}ms age={render_age}ms cells={cells} grid={cols}x{rows}"
            ),
        );
    } else {
        log_line("WARN", "last render: unavailable");
    }

    dump_breadcrumbs("ui heartbeat stale");
}

fn log_dir() -> PathBuf {
    PathBuf::from("logs")
}

fn timestamp_ms() -> String {
    let now = now_millis();
    let secs = now / 1000;
    let millis = now % 1000;
    format!("{secs}.{millis:03}")
}

fn thread_label() -> String {
    let current = std::thread::current();
    match current.name() {
        Some(name) => format!("{name}/{:?}", current.id()),
        None => format!("{:?}", current.id()),
    }
}

fn now_millis() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_millis() as u64
}

fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-{:03}", now.as_secs(), now.subsec_millis())
}
