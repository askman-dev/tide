use crate::logging;

/// Platform-specific terminal session implementation.
///
/// On macOS this is backed by `alacritty_terminal` and `portable-pty`. On
/// non-macOS platforms it is a lightweight stub that compiles but does not
/// spawn a real PTY.
#[cfg(target_os = "macos")]
mod platform {
    use super::logging;
    use crate::services::{get_clipboard_string, set_clipboard_string};
    use alacritty_terminal::event::{Event, EventListener};
    use alacritty_terminal::grid::{Dimensions, Scroll};
    use alacritty_terminal::sync::FairMutex;
    use alacritty_terminal::term::{Config, Term};
    use alacritty_terminal::vte::ansi::{Processor, StdSyncHandler};
    use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
    use std::io::{self, Read, Write};
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread::{self, JoinHandle};
    use std::time::Instant;

    #[derive(Clone)]
    pub struct TideEventListener {
        pty_writer: Arc<Mutex<Box<dyn Write + Send>>>,
        alive: Arc<AtomicBool>,
    }

    impl TideEventListener {
        pub fn new(
            pty_writer: Arc<Mutex<Box<dyn Write + Send>>>,
            alive: Arc<AtomicBool>,
        ) -> Self {
            Self { pty_writer, alive }
        }

        fn write_to_pty(&self, text: &str) {
            let mut writer = self
                .pty_writer
                .lock()
                .expect("pty_writer mutex poisoned");

            if let Err(err) = writer.write_all(text.as_bytes()) {
                logging::log_line(
                    "ERROR",
                    &format!("Failed to write terminal event to PTY: {err}"),
                );
            }
        }
    }

    impl EventListener for TideEventListener {
        fn send_event(&self, event: Event) {
            match event {
                Event::Wakeup => logging::log_line("DEBUG", "Terminal wakeup event"),
                Event::Exit => {
                    logging::log_line("INFO", "Terminal requested exit");
                    self.alive.store(false, Ordering::SeqCst);
                }
                Event::ChildExit(code) => {
                    logging::log_line(
                        "INFO",
                        &format!("Terminal child process exited with code {code}"),
                    );
                    self.alive.store(false, Ordering::SeqCst);
                }
                Event::PtyWrite(text) => {
                    self.write_to_pty(&text);
                }
                Event::ClipboardStore(_, text) => {
                    set_clipboard_string(&text);
                }
                Event::ClipboardLoad(_, formatter) => {
                    if let Some(contents) = get_clipboard_string() {
                        let seq = formatter(&contents);
                        self.write_to_pty(&seq);
                    }
                }
                _ => {}
            }
        }
    }

    /// Simple terminal dimensions used for `Term::new` and resize.
    #[derive(Clone, Copy)]
    struct TermDimensions {
        columns: usize,
        screen_lines: usize,
    }

    impl TermDimensions {
        fn new(columns: u16, rows: u16) -> Self {
            Self {
                columns: columns as usize,
                screen_lines: rows as usize,
            }
        }
    }

    impl Dimensions for TermDimensions {
        fn total_lines(&self) -> usize {
            self.screen_lines
        }

        fn screen_lines(&self) -> usize {
            self.screen_lines
        }

        fn columns(&self) -> usize {
            self.columns
        }
    }

    /// Core PTY-backed terminal session for macOS.
    ///
    /// This owns the `alacritty_terminal::Term`, PTY handles, scrollback
    /// configuration, and the IO thread which feeds PTY output into the
    /// terminal state.
    pub struct TerminalSession {
        pub(crate) term: Arc<FairMutex<Term<TideEventListener>>>,
        pty_master: Box<dyn MasterPty + Send>,
        pty_writer: Arc<Mutex<Box<dyn Write + Send>>>,
        scrollback: usize,
        alive: Arc<AtomicBool>,
        bytes_read: Arc<AtomicU64>,
        bytes_written: AtomicU64,
        notify: Arc<dyn Fn() + Send + Sync>,
        io_thread: Option<JoinHandle<()>>,
    }

    impl TerminalSession {
        /// Create a new PTY-backed terminal session rooted at `workspace_root`.
        ///
        /// This uses a fixed 80x24 cell grid for now; Floem-driven sizing
        /// will be hooked up in a later step.
        pub fn new(
            workspace_root: &Path,
            notify: Arc<dyn Fn() + Send + Sync>,
        ) -> io::Result<Arc<Self>> {
            const DEFAULT_COLS: u16 = 80;
            const DEFAULT_ROWS: u16 = 24;
            const MIN_SCROLLBACK: usize = 500;
            const DEFAULT_SCROLLBACK: usize = 2000;

            let scrollback = DEFAULT_SCROLLBACK.max(MIN_SCROLLBACK);
            let dims = TermDimensions::new(DEFAULT_COLS, DEFAULT_ROWS);
            let term_config = Config {
                scrolling_history: scrollback,
                ..Config::default()
            };

            logging::log_line(
                "INFO",
                &format!(
                    "Starting TerminalSession at {} ({}x{}, scrollback={})",
                    workspace_root.display(),
                    DEFAULT_COLS,
                    DEFAULT_ROWS,
                    scrollback,
                ),
            );

            // Create PTY pair.
            let pty_system = native_pty_system();
            let pty_size = PtySize {
                rows: DEFAULT_ROWS,
                cols: DEFAULT_COLS,
                pixel_width: 0,
                pixel_height: 0,
            };

            let pair = pty_system
                .openpty(pty_size)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

            // Spawn default shell in the workspace root.
            let mut cmd = CommandBuilder::new_default_prog();
            cmd.cwd(workspace_root);
            cmd.env("TERM", "xterm-256color");
            cmd.env("COLORTERM", "truecolor");

            let _child = pair
                .slave
                .spawn_command(cmd)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

            // Take writer handle which we'll use for user input and clipboard events.
            let writer = pair
                .master
                .take_writer()
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
            let pty_writer = Arc::new(Mutex::new(writer));

            let alive = Arc::new(AtomicBool::new(true));
            let bytes_read = Arc::new(AtomicU64::new(0));

            // Create terminal state with configured scrollback and event listener.
            let term = Term::new(
                term_config,
                &dims,
                TideEventListener::new(Arc::clone(&pty_writer), Arc::clone(&alive)),
            );
            let term = Arc::new(FairMutex::new(term));

            // Clone a reader for the IO thread.
            let mut reader = pair
                .master
                .try_clone_reader()
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

            let term_for_thread = Arc::clone(&term);
            let alive_for_thread = Arc::clone(&alive);
            let bytes_read_for_thread = Arc::clone(&bytes_read);
            let notify_for_thread = Arc::clone(&notify);

            let io_thread = thread::Builder::new()
                .name("tide-terminal-io".to_string())
                .spawn(move || {
                    logging::log_line("INFO", "Terminal IO thread started");
                    let mut parser = Processor::<StdSyncHandler>::new();
                    let mut buf = [0u8; 4096];
                    let mut total_bytes: u64 = 0;

                    while alive_for_thread.load(Ordering::SeqCst) {
                        match reader.read(&mut buf) {
                            Ok(0) => {
                                logging::log_line(
                                    "INFO",
                                    "Terminal PTY reached EOF; stopping IO thread",
                                );
                                break;
                            }
                            Ok(n) => {
                                let chunk = &buf[..n];
                                let parse_start = Instant::now();
                                {
                                    let mut term = term_for_thread.lock();
                                    parser.advance(&mut *term, chunk);
                                }
                                notify_for_thread();
                                logging::log_slow_op(
                                    "pty parse",
                                    parse_start.elapsed(),
                                    &format!("bytes={n}"),
                                );
                                total_bytes += n as u64;
                                bytes_read_for_thread.fetch_add(
                                    n as u64,
                                    Ordering::Relaxed,
                                );
                            }
                            Err(err) => {
                                logging::log_line(
                                    "ERROR",
                                    &format!("Error reading from PTY: {err}"),
                                );
                                break;
                            }
                        }
                    }

                    alive_for_thread.store(false, Ordering::SeqCst);
                    logging::log_line(
                        "INFO",
                        &format!(
                            "Terminal IO thread exiting (bytes_read={total_bytes})"
                        ),
                    );
                })?;

            let session = TerminalSession {
                term,
                pty_master: pair.master,
                pty_writer,
                scrollback,
                alive,
                bytes_read,
                bytes_written: AtomicU64::new(0),
                notify,
                io_thread: Some(io_thread),
            };

            Ok(Arc::new(session))
        }

        /// Check if the terminal session is currently active (PTY running).
        pub fn is_active(&self) -> bool {
            self.alive.load(Ordering::SeqCst)
        }

        /// Write raw bytes to the PTY.
        pub fn write(&self, bytes: &[u8]) -> io::Result<()> {
            let mut writer = self
                .pty_writer
                .lock()
                .expect("pty_writer mutex poisoned");

            let start = Instant::now();
            let result = writer.write_all(bytes);
            if result.is_ok() {
                self.bytes_written
                    .fetch_add(bytes.len() as u64, Ordering::Relaxed);
            }
            logging::log_slow_op(
                "pty write",
                start.elapsed(),
                &format!("bytes={}", bytes.len()),
            );
            result
        }

        /// Resize both the PTY and the terminal grid.
        pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
            let dims = TermDimensions::new(cols, rows);

            self.pty_master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

            {
                let mut term = self.term.lock();
                term.resize(dims);
            }

            Ok(())
        }

        /// Scroll the visible terminal content by the given delta.
        pub fn scroll_display(&self, delta: i32) {
            let mut term = self.term.lock();
            term.scroll_display(Scroll::Delta(delta));
        }

        /// Helper giving read-only access to the underlying `Term`.
        ///
        /// The provided closure is executed while the terminal lock is held.
        pub fn with_term<R>(&self, f: impl FnOnce(&Term<TideEventListener>) -> R) -> R {
            let term = self.term.lock();
            f(&*term)
        }

        /// Helper giving mutable access to the underlying `Term`.
        ///
        /// The provided closure is executed while the terminal lock is held.
        pub fn with_term_mut<R>(&self, f: impl FnOnce(&mut Term<TideEventListener>) -> R) -> R {
            let mut term = self.term.lock();
            f(&mut *term)
        }

        /// Accessor used in tests to validate scrollback configuration.
        #[cfg(test)]
        pub(crate) fn scrollback(&self) -> usize {
            self.scrollback
        }
    }

    impl Drop for TerminalSession {
        fn drop(&mut self) {
            logging::breadcrumb("TerminalSession::drop started");
            self.alive.store(false, Ordering::SeqCst);
            logging::log_line(
                "INFO",
                &format!(
                    "Dropping TerminalSession (bytes_read={} bytes_written={})",
                    self.bytes_read.load(Ordering::Relaxed),
                    self.bytes_written.load(Ordering::Relaxed),
                ),
            );

            if let Some(handle) = self.io_thread.take() {
                logging::breadcrumb("TerminalSession joining IO thread (background)");
                let joiner = thread::Builder::new()
                    .name("tide-terminal-join".to_string())
                    .spawn(move || match handle.join() {
                        Ok(()) => {
                            logging::breadcrumb("TerminalSession IO thread joined: true");
                        }
                        Err(err) => {
                            logging::log_line(
                                "ERROR",
                                &format!("Terminal IO thread join failed: {err:?}"),
                            );
                            logging::breadcrumb("TerminalSession IO thread joined: false");
                        }
                    });
                if let Err(err) = joiner {
                    logging::log_line(
                        "ERROR",
                        &format!("Terminal join thread spawn failed: {err:?}"),
                    );
                }
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::logging;
    use std::io;
    use std::path::Path;
    use std::sync::Arc;

    /// Non-macOS stub which compiles but does not spawn a PTY.
    pub struct TerminalSession;

    impl TerminalSession {
        /// Create a new stub terminal session.
        pub fn new(
            workspace_root: &Path,
            _notify: Arc<dyn Fn() + Send + Sync>,
        ) -> io::Result<Arc<Self>> {
            logging::log_line(
                "WARN",
                &format!(
                    "Terminal is only available on macOS; stub TerminalSession for workspace {}",
                    workspace_root.display()
                ),
            );
            Ok(Arc::new(TerminalSession))
        }

        /// Stub write; does nothing on non-macOS.
        pub fn write(&self, _bytes: &[u8]) -> io::Result<()> {
            Ok(())
        }

        /// Stub resize; does nothing on non-macOS.
        pub fn resize(&self, _cols: u16, _rows: u16) -> io::Result<()> {
            Ok(())
        }

        /// Stub scroll; does nothing on non-macOS.
        pub fn scroll_display(&self, _delta: i32) {}

        /// Stub check; always active.
        pub fn is_active(&self) -> bool {
            true
        }
    }
}

pub use platform::TerminalSession;

#[cfg(test)]
mod tests {
    use super::TerminalSession;
    use std::env;
    use std::sync::Arc;

    #[test]
    fn terminal_session_new_succeeds() {
        let root = env::current_dir().unwrap();
        let session = TerminalSession::new(&root, Arc::new(|| {}))
            .expect("terminal session should construct");
        let _ = session;
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn terminal_session_scrollback_at_least_500() {
        let root = env::current_dir().unwrap();
        let session = TerminalSession::new(&root, Arc::new(|| {}))
            .expect("terminal session should construct");
        assert!(session.scrollback() >= 500);
    }
}
