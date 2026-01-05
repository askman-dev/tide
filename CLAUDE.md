# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tide is a terminal-based IDE built with Rust and the Floem UI framework. It features a three-pane resizable layout with interactive terminal emulation (macOS only), comprehensive logging/diagnostics, and a watchdog system for detecting UI hangs.

## Build Commands

```bash
cargo build          # Build the project
cargo run            # Run the application
cargo build --release # Build for release
```

Rust edition: 2021

Main dependencies:
- floem (git from lapce/floem) - UI framework
- alacritty_terminal 0.25.1 (macOS) - Terminal emulation
- portable-pty 0.9.0 (macOS) - PTY management
- arboard 3 (macOS) - Clipboard operations
- dispatch 0.2 (macOS) - GCD for direct terminal resize

Debug environment variables:
- Default (unset): Standard macOS titlebar - no blur during zoom animation
- `TIDE_WINDOW_STYLE=fullsize` - Transparent titlebar + full content view (tabs in titlebar, has zoom blur)
- `TIDE_WINDOW_STYLE=hidden` - Hidden titlebar via floem's show_titlebar(false) - WARNING: causes zoom blur
- `TIDE_DISALLOW_HIDPI=1` - Disable HiDPI backing scale (may reduce resize/GPU pressure)

Note: floem's `show_titlebar(false)` on macOS internally enables `fullsize_content_view`, causing zoom animation blur.

## Architecture

```
src/
├── main.rs           # Entry point, window configuration
├── app.rs            # App view composition, tab management, UI watchdog
├── logging.rs        # Diagnostics: breadcrumbs, performance tracking, heartbeat monitoring
├── model.rs          # Data models (WorkspaceTab)
├── theme.rs          # UiTheme with color definitions
├── components/
│   ├── mod.rs        # Public exports: layout views (app_shell, main_layout, tab_bar, get_last_window_size), terminal functions (terminal_view, force_terminal_repaint, direct_terminal_resize on macOS), UI atoms (tab_button), icons
│   ├── layout.rs     # SplitDragCapture: custom three-pane resizable layout, WindowResized event processing with breadcrumb logging, animation timer (1.2s fixed delay spawned via std::thread), calls direct_terminal_resize when timer expires, window size tracking (LAST_WINDOW_WIDTH/HEIGHT atomics exported via get_last_window_size), debounced resize clamping (100ms intervals via last_clamp_at)
│   ├── terminal.rs   # Terminal canvas rendering with alacritty_terminal, force repaint trigger system (FORCE_REPAINT_TRIGGER), direct resize system (GLOBAL_TERMINAL_SESSION, CACHED_CELL_WIDTH/HEIGHT atomics), functions: register_force_repaint_trigger, force_terminal_repaint, register_terminal_session, direct_terminal_resize, update_cached_cell_size, get_cached_cell_size
│   ├── atoms.rs      # Basic UI components (buttons, panels)
│   ├── panels.rs     # Panel view components (file tree, git status)
│   ├── icons.rs      # Icon definitions (FILE, FOLDER, GIT)
│   ├── file_tree.rs  # File explorer tree view
│   └── git_status.rs # Git status display
└── services/
    ├── mod.rs        # Public exports: clipboard (get/set_clipboard_string), fs (build_tree_entries, list_dir_entries), git (git_status_entries), terminal (TerminalSession)
    ├── terminal.rs   # TerminalSession: PTY management, IO thread
    ├── fs.rs         # Directory traversal functions
    ├── git.rs        # Git status parsing
    └── clipboard.rs  # Clipboard operations (arboard integration)
```

### Key Architectural Patterns

**Custom View Implementation** (see `SplitDragCapture` in layout.rs):
- Implement `View` trait with `id()`, `view_style()`, `event_before_children()`
- Use `ViewId::new()` and `id.set_children_vec()` for child management
- Debounce resize operations (clamp_widths every 100ms) to avoid blocking UI thread
- Force immediate layout/repaint on WindowResized to minimize macOS scaled screenshot artifacts
- WindowResized event logs timing breakdown: total, layout, paint, repaint phases (throttled to 200ms or when total>5ms)
- Handle hit-testing for drag areas with proper coordinate bounds checking
- Track last_resize_log_at and last_clamp_at to throttle logging and resize calculations

**Animation Detection System** (layout.rs):
- ANIMATION_TIMER_ACTIVE and RESIZE_BURST_START_MS atomics track resize burst timing
- start_animation_timer() spawns background thread with fixed 1.2s delay from burst start
- Detects new resize bursts (>500ms gap since last burst started) and starts one timer per burst
- Key insight: macOS visual animation ~1s, but WindowResized events delayed ~2s during zoom animation
- Timer directly calls direct_terminal_resize() after 1.2s, bypassing floem event queue delay
- Only one timer thread active at a time (via atomic swap check)
- LAST_WINDOW_WIDTH/HEIGHT atomics store latest window size from WindowResized events (f64::to_bits for atomic storage)
- get_last_window_size() exports window size atomics for external access (public API via components/mod.rs, uses f64::from_bits to reconstruct)
- WindowResized event handler: logs breadcrumb with size, updates atomics immediately via LAST_WINDOW_WIDTH.store(size.width.to_bits()), starts animation timer
- Animation timer logs "animation timer: 1.2s elapsed, executing resize directly for {w}x{h}" at DEBUG level
- Uses current_time_ms() helper (SystemTime since UNIX_EPOCH) for millisecond-precision timing

**Terminal Rendering Flow** (macOS only):
1. `canvas()` closure receives paint context and size
2. Calculate grid dimensions from canvas size and cell metrics
3. Build color palette from theme using `TerminalColorList::from_palette()`
4. Debounce PTY resize (50ms) via `ExtSendTrigger` + background thread
5. Render using `last_size` (not `pending_size`) to avoid flicker during resize
6. Track render performance via `logging::record_terminal_render()`
7. IME cursor positioning: Effect tracks cell_size/cursor changes, anchors IME candidate window at terminal caret using canvas-relative coordinates
8. Force repaint trigger (FORCE_REPAINT_TRIGGER) registered during initialization for external repaint requests

**Force Repaint System** (terminal.rs, macOS only):
- FORCE_REPAINT_TRIGGER global (OnceLock<Mutex<Option<ExtSendTrigger>>>) enables external repaint requests
- register_force_repaint_trigger() stores trigger during terminal view initialization (called from terminal_view)
- force_terminal_repaint() clones and registers trigger to force repaint, called by animation timer in layout.rs
- Public export via components/mod.rs (#[cfg(target_os = "macos")] guard, exported alongside terminal_view)
- Logs "force_terminal_repaint: triggered from animation timer" when activated with DEBUG level
- Integration flow: layout.rs WindowResized → start_animation_timer → 1.2s delay → force_terminal_repaint → ExtSendTrigger → terminal canvas repaint
- Thread-safe access via mutex lock with graceful handling if trigger not registered

**Direct Terminal Resize System** (terminal.rs, macOS only):
- GLOBAL_TERMINAL_SESSION stores weak reference to TerminalSession for direct PTY resize access (OnceLock<Mutex<SendSyncSession>>)
- CACHED_CELL_WIDTH/HEIGHT atomics cache cell dimensions for grid calculations without canvas access (f64::to_bits for atomic storage)
- register_terminal_session(session: &Arc<TerminalSession>) called during terminal view initialization to store session reference
- update_cached_cell_size(width, height) updates cached cell dimensions from canvas measurements (uses f64::to_bits + AtomicU64::store with SeqCst)
- get_cached_cell_size() -> (f64, f64) retrieves cached dimensions for resize calculations (uses f64::from_bits)
- direct_terminal_resize(window_w, window_h) bypasses floem event queue to resize PTY immediately
  - Called from layout.rs animation timer after 1.2s delay (via std::thread::spawn background thread)
  - Gets cached cell size, falls back to force_terminal_repaint() if not available
  - Upgrades weak session reference from GLOBAL_TERMINAL_SESSION, falls back if unavailable
  - Estimates terminal canvas size from window dimensions: ~500px non-terminal width (left 200 + handle 10 + right 260 + handle 10 + padding), ~80px top overhead (tab bar + padding)
  - Terminal canvas estimate: width = (window_width - 500).max(300), height = (window_height - 80).max(200)
  - Accounts for 16px total padding (8px each side), calculates available space: available = (canvas - 16).max(1)
  - Calculates grid size: cols = (available_width / cell_width).floor().max(1.0) as u16, rows = (available_height / cell_height).floor().max(1.0) as u16
  - Calls session.resize(cols, rows) directly on background thread (PTY operations are thread-safe)
  - Logs detailed calculation: "direct_terminal_resize: window={w}x{h} canvas_est={cw}x{ch} cell={cell_w}x{cell_h} -> {cols}x{rows}" at DEBUG level
  - Logs fallback messages: "direct_terminal_resize: no cached cell size yet" or "direct_terminal_resize: no session registered" at DEBUG level
  - Triggers force_terminal_repaint() to update canvas after resize
  - Error handling: logs "direct_terminal_resize failed: {err}" at ERROR level if session.resize() fails
- SendSyncSession wrapper makes Weak<TerminalSession> Sync for cross-thread access (unsafe impl Send + Sync, safe because only holds Weak reference accessed from main thread via GCD dispatch)
- Public export via components/mod.rs (#[cfg(target_os = "macos")] guard)

**Heartbeat Watchdog** (app.rs):
- Background thread pings UI every 500ms via `ExtSendTrigger`
- Detects stale heartbeat after 2s, dumps breadcrumbs for debugging

## Coding Conventions

### Logging
- `logging::init()` - Initialize logging system (call once at startup from main.rs)
  - Captures UI thread label from std::thread::current() (format: "name/ThreadId" or just "ThreadId")
  - Creates log directory `~/Library/Logs/com.tide/` and file `tide-{timestamp}.log`
  - Sets panic hook with backtrace: logs "Caught panic: {info}" and full backtrace via std::backtrace::Backtrace::force_capture()
  - Calls touch_heartbeat() to initialize heartbeat tracking
- `logging::log_path()` - Get path to current log file (returns `Option<PathBuf>`)
- `logging::breadcrumb(msg)` - Lightweight event tracking
  - Circular buffer, 64 entries via BREADCRUMBS OnceLock<Mutex<VecDeque<String>>>
  - Format: "{timestamp_ms} [{thread_label}] {message}"
  - Uses try_lock to avoid blocking, silently drops if locked
  - When buffer full, pops front before pushing new entry
- `logging::log_line(level, msg)` - Persistent logging with smart flush strategy
  - Format: `[timestamp_ms] [thread_label] [LEVEL] message`
  - Selective flush strategy: WARN/ERROR/PANIC flush immediately, others batch flush every 250ms (tracked via LAST_FLUSH_AT_MS atomic) to balance crash diagnostics with performance
  - Supports multi-line messages (each line logged separately via message.lines())
  - Handles empty messages by logging just timestamp, thread, and level markers
  - Uses OnceLock<Mutex<std::fs::File>> for thread-safe file access (LOG_FILE static)
  - Flush tracking: LAST_FLUSH_AT_MS atomic stores last flush time, checks elapsed >= 250ms, updates atomic when flushing
  - Rationale: Immediate flush on errors ensures crash/hang diagnostics hit disk, but flushing every log line creates performance bottlenecks in render loops (especially terminal render)
- `logging::dump_breadcrumbs(reason)` - Dump all breadcrumbs to log with WARN level
  - Logs "breadcrumbs: {reason}" header
  - Iterates buffer entries: "breadcrumb: {entry}"
  - Handles locked buffer gracefully: logs "breadcrumb buffer locked"
- `logging::measure_ui_event(label, || { ... })` - Time UI operations, warn if >=50ms (SLOW_UI_EVENT_MS), format: "slow ui event: {label} {ms}ms"
- `logging::touch_heartbeat()` - Update heartbeat timestamp (LAST_HEARTBEAT_MS atomic), call in UI loops to prevent false hang detection, also initializes HEARTBEAT_STALE flag
- `logging::check_heartbeat(stale_after)` - Check if heartbeat is stale, dumps breadcrumbs if stale, sets HEARTBEAT_STALE flag
- `logging::record_terminal_render(duration, cells, cols, rows)` - Track render performance
  - Updates atomics: LAST_RENDER_MS (duration), LAST_RENDER_AT_MS (now_millis), LAST_RENDER_CELLS (cell_count), LAST_RENDER_COLS/ROWS (grid dimensions)
  - Detects resize: compares prev_cols/prev_rows from atomic swap with current cols/rows
  - Logs on resize: "terminal render after resize: {ms}ms cells={count} grid={cols}x{rows} (was {prev_cols}x{prev_rows})" at DEBUG level
  - Warns if slow: "slow terminal render: {ms}ms cells={cell_count} grid={cols}x{rows}" at WARN level if >=50ms (SLOW_RENDER_MS)
- `logging::log_slow_op(op, elapsed, detail)` - Log operations >=250ms (SLOW_OP_MS), format: "slow op: {op} {ms}ms {detail}"
- Internal helpers: `now_millis()` (SystemTime since UNIX_EPOCH as millis), `timestamp()` (YYYYMMDD-HHMMSS), `timestamp_ms()` (YYYYMMDD-HHMMSS-mmm), `thread_label()` (returns UI_THREAD_LABEL or current thread info), `log_dir()` (platform-specific log directory)
- Log file location: `~/Library/Logs/com.tide/tide-{timestamp}.log` on macOS
- Performance constants: BREADCRUMB_CAP=64, SLOW_RENDER_MS=50, SLOW_OP_MS=250, SLOW_UI_EVENT_MS=50

### UI Constants
- Pane minimums: LEFT=200, CENTER=300, RIGHT=260 (adjusted to prevent layout overflow in windowed mode, fixes splitter hit-testing issues)
- Handle width: 10px
- Debounce: resize clamping 100ms (prevents UI thread blocking during animations), log throttling 200-250ms
- Performance thresholds: SLOW_RENDER_MS=50, SLOW_OP_MS=250, SLOW_UI_EVENT_MS=50
- Breadcrumb capacity: 64 entries (circular buffer)

### Platform Handling
- Terminal features gated with `#[cfg(target_os = "macos")]`
- Non-macOS builds stub out terminal functionality

## Dependencies

- **floem** (git): UI framework from lapce/floem
- **alacritty_terminal** (0.25.1, macOS): Terminal emulation engine
- **portable-pty** (0.9.0, macOS): Cross-platform PTY interface
- **arboard** (3.x, macOS): Clipboard operations
- **dispatch** (0.2, macOS): GCD (Grand Central Dispatch) support for direct terminal resize

## Known Issues & Workarounds

- macOS window zoom animation captures a snapshot; app cannot repaint during animation
- Animation detection uses fixed 1.2s timer from resize burst start (macOS animation ~1s, events delayed ~2s)
- Background timer thread bypasses event queue delay to ensure timely repaint after animation
- Canvas paint may not be called during resize animation despite `request_paint()` calls
- RIGHT_MIN_WIDTH reduced to 260.0 to prevent layout overflow in windowed mode (fixes splitter hit-testing)

## Critical: floem Version Pin (b215faa)

**Problem**: New floem versions cause 2s delay after macOS zoom animation completes.

**Root Cause**: floem's internal `size()` function executes expensive `style() + layout() + process_update() + schedule_repaint()` on every WindowResized event BEFORE app's event handler is called. App cannot skip these operations.

**Solution**: Pin floem to Lapce's version (e0dd862) which doesn't have this issue.

**Cargo.toml**:
```toml
floem = { git = "https://github.com/lapce/floem", rev = "e0dd862564e3afbad5cba8ebe60df166a7a41e56" }
```

**API Compatibility (Lapce floem vs newer floem)**:
| Lapce floem (e0dd862) | Newer floem |
|-----------------------|-------------|
| `create_effect(...)` | `Effect::new(...)` |
| `label(\|\| "text")` | `Label::new("text")` |
| `container(child)` | `Container::new(child)` |
| `empty()` | `Empty::new()` |
| `Event::KeyDown(KeyEvent)` | `Event::Key(KeyboardEvent)` |
| `key_event.key.logical_key` | `key_event.key` |
| `Event::PointerDown/Move/Up` | `Event::Pointer(PointerEvent::Down/Move/Up)` |
| `.keyboard_navigable()` | `.focusable(true)` |
| `Event::PointerWheel(wheel).delta.y` | `event.pixel_scroll_delta_vec2()` |
| Custom `is_selecting` RwSignal | `canvas_id.is_active()` |

**Upgrade Path Options**:
1. Fork floem, modify `size()` to throttle layout (recommended for stability)
2. Submit upstream PR with resize throttling option
3. Wait for upstream performance fix

**Verification**: See `swap_memory/tech_spec_window_resize.md` for full technical spec and verification checklist.
