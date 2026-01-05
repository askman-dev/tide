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
- rfd 0.15 (macOS) - Native file dialog for folder picker in tab dropdown menu

Debug environment variables:
- Default (unset): Hidden titlebar (like Lapce) - tabs at top, may have brief blur during zoom
- `TIDE_WINDOW_STYLE=fullsize` - Transparent titlebar + full content view (tabs in titlebar, has zoom blur)
- `TIDE_WINDOW_STYLE=hidden` - Hidden titlebar via floem's show_titlebar(false) - WARNING: causes zoom blur
- `TIDE_DISALLOW_HIDPI=1` - Disable HiDPI backing scale (may reduce resize/GPU pressure)

Note: floem's `show_titlebar(false)` on macOS internally enables `fullsize_content_view`, causing zoom animation blur.

## Architecture

```
src/
├── main.rs           # Entry point, window configuration
├── app.rs            # App view composition, tab management with context menus (open folder, reveal in Finder, close), UI watchdog, reactive view wrappers (file_tree_view_reactive, git_status_view_reactive), VSCode-style collapsible panels in left column (files_expanded, changes_expanded, history_expanded RwSignals)
├── logging.rs        # Diagnostics: breadcrumbs, performance tracking, heartbeat monitoring
├── model.rs          # Data models: TerminalPane struct (id, session: RwSignal<Option<Arc<TerminalSession>>>, trigger: ExtSendTrigger, flex_ratio: RwSignal<f64>) represents single terminal pane with flexible width distribution, WorkspaceTab with RwSignal fields (name, root, file_tree, git_status, terminal_panes: Vec<TerminalPane>, next_pane_id) for reactive UI updates and split pane support
├── theme.rs          # UiTheme with color definitions
├── components/
│   ├── mod.rs        # Public exports: layout views (app_shell, main_layout, tab_bar, get_last_window_size), terminal functions (terminal_view, force_terminal_repaint on macOS - triggers repaint for all panes), UI atoms (tab_button, tab_button_with_menu, collapsible_panel_header), panel views (collapsible_panel_view, panel_view), icons
│   ├── layout.rs     # SplitDragCapture: custom three-pane resizable layout, WindowResized event processing with breadcrumb logging, animation timer (1.2s fixed delay spawned via std::thread), calls force_terminal_repaint when timer expires (each pane's canvas recalculates its own grid size), window size tracking (LAST_WINDOW_WIDTH/HEIGHT atomics exported via get_last_window_size), debounced resize clamping (100ms intervals via last_clamp_at), main_layout columns use height_full() and Overflow::Hidden, h_stack uses items_stretch(), tab_bar uses flex_shrink(0.0) to prevent shrinking from content pressure
│   ├── terminal/     # Terminal component (modular structure, macOS only)
│   │   ├── mod.rs    # Module orchestration: terminal_view() entry point with multi-pane support via dyn_stack, FORCE_REPAINT_TRIGGER global, register_force_repaint_trigger/force_terminal_repaint functions
│   │   ├── constants.rs # Configuration constants: TERMINAL_FONT_SIZE=13.0, CELL_PADDING=8.0, SPLITTER_WIDTH=6.0, PTY_RESIZE_DEBOUNCE_MS=50, overlay timing (OVERLAY_SHOW_DURATION_MS, OVERLAY_MIN_VISIBLE_MS), split pane timing (SPLIT_TRIGGER_DELAY_MS, SPLIT_SECOND_WAVE_MS), terminal_font_families() helper
│   │   ├── colors.rs # Color palette: TerminalColorList struct with 256-color palette generation (fill_named, fill_cube, fill_gray_ramp), resolve_fg_color/resolve_bg_color functions with ANSI color mapping, background_brush/cursor_brush theme helpers
│   │   ├── panel.rs  # Panel orchestration: SplitterDragState type (Option<(pane_id, last_x)>), calculate_splitter_drag function with MIN_PANE_RATIO=0.05, splitter styling helpers (splitter_background_color, splitter_hover_color), panel header/container style functions, utility functions (is_last_pane, find_pane_index)
│   │   └── instance/ # Terminal instance implementation (single pane)
│   │       ├── mod.rs # Instance submodule exports: TerminalInstanceState, renderer, input
│   │       ├── state.rs # TerminalInstanceState struct: Bundles 14 reactive signals (error_msg, last_size, pending_size, last_resize_request, resize_trigger, cell_size, cell_y_offset, last_pty_resize_at, ime_focused, ime_update_tick, last_ime_cursor_area, last_canvas_size, scroll_accumulator, resize_overlay state), provides new() constructor with default signal values
│   │       ├── renderer.rs # Canvas rendering helpers: measure_cell_size (font metrics, cell dimensions), calculate_grid_size (canvas to grid conversion), cell_position/is_cell_visible (coordinate helpers), resolve_cell_colors (fg/bg color resolution with selection/inverse support), CellRenderContext struct for render state
│   │       └── input.rs # Input handling: pointer_to_grid_point (mouse to grid coords), key_to_pty_bytes (keyboard to ANSI sequences), named_key_to_bytes/char_to_pty_bytes (key conversion with modifier support for Ctrl/Alt/Shift), scroll calculation helpers
│   ├── atoms.rs      # Basic UI components: tab_button, tab_button_with_menu (h_stack with label + chevron dropdown arrow, uses .popout_menu() for Open Folder/Reveal in Finder/Close actions, reactive tab_label via RwSignal<String>), collapsible_panel_header (clickable header with chevron toggle, flex_shrink(0.0) to prevent shrinking), list items, panel headers, splitters, chevron SVG constants (COLLAPSE_CHEVRON, EXPAND_CHEVRON, CHEVRON_DOWN)
│   ├── panels.rs     # Panel view components: collapsible_panel_view (VSCode-style collapsible panels with internal scrolling, flex_basis(0) pattern for content height isolation - prevents File Explorer content size from affecting sibling panel layouts, flex_grow/shrink with min_height(0.0) for proper flex layout, OverflowX::Hidden to prevent horizontal scrollbar), panel_view (static panels), file_tree_view, git_status_view
│   ├── icons.rs      # Icon definitions (FILE, FOLDER, GIT)
│   ├── file_tree.rs  # File explorer tree view
│   └── git_status.rs # Git status display
└── services/
    ├── mod.rs        # Public exports: clipboard (get/set_clipboard_string), fs (build_tree_entries, list_dir_entries), git (git_status_entries), terminal (TerminalSession)
    ├── terminal.rs   # TerminalSession: PTY management, IO thread, TideEventListener with on_title_change callback for OSC title events (Event::Title)
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
- Timer calls force_terminal_repaint() after 1.2s to trigger canvas repaints - each pane independently calculates its own correct grid size from canvas dimensions
- Split pane architecture: Each terminal pane measures its own canvas size and calculates appropriate PTY grid dimensions independently
- No global session or cached cell size - eliminates PTY mismatch issues across split panes
- Only one timer thread active at a time (via atomic swap check)
- LAST_WINDOW_WIDTH/HEIGHT atomics store latest window size from WindowResized events (f64::to_bits for atomic storage)
- get_last_window_size() exports window size atomics for external access (public API via components/mod.rs, uses f64::from_bits to reconstruct)
- WindowResized event handler: logs breadcrumb with size, updates atomics immediately via LAST_WINDOW_WIDTH.store(size.width.to_bits()), starts animation timer
- During animation burst (within 1.5s of burst start), SKIPS expensive clamp_widths operations to drain event queue faster - animation timer handles resize after burst settles
- Logs "WindowResized {w}x{h}: SKIPPED (animation burst) total={ms}ms" at DEBUG level during burst (throttled to 200ms intervals)
- Animation timer logs "animation timer: 1.2s elapsed, triggering repaint for {w}x{h}" at DEBUG level
- Uses current_time_ms() helper (SystemTime since UNIX_EPOCH) for millisecond-precision timing

**Terminal Rendering Flow** (terminal/mod.rs, macOS only):
1. terminal_view() uses dyn_stack to preserve pane views by ID when adding/removing panes
   - dyn_stack(move || terminal_panes.get(), |pane| pane.id, move |pane| ...) preserves views vs dyn_container rebuilding
   - Each pane's flex_ratio tracked reactively in style closure: .style(move |s| s.flex_basis(0.0).flex_grow(pane_flex_ratio.get()))
   - Layout updates without rebuilding pane views when flex_ratio changes
   - Each pane+splitter wrapped in h_stack, dyn_stack uses flex_row layout for horizontal arrangement
2. Splitter drag system with dynamic sensitivity (integrated into terminal_view):
   - Each pane includes splitter on right side (SPLITTER_WIDTH=6.0px), hidden for last pane
   - panes_stack_id captured via .id() before attaching event handlers to enable container width access during drag
   - drag_state: RwSignal<SplitterDragState> tracks (pane_id_left, last_x) at parent level to avoid view rebuilds during drag
   - PointerDown: Captures initial state with last_x = -1.0 sentinel (first move will set actual position)
   - PointerMove: Dynamic sensitivity calculation for 1:1 mouse-to-splitter pixel mapping:
     - Gets container_rect via panes_stack_id.layout_rect(), extracts width with .max(100.0) fallback
     - Calculates delta_x = current_x - last_x (incremental pixel delta from last position)
     - Formula: ratio_delta = delta_x * total_ratio / container_width
     - Rationale: Makes 1px mouse movement = 1px splitter movement, splitter follows mouse exactly
     - Previous fixed sensitivity (delta_x / 100.0) caused splitter to move faster than mouse
     - Applies min_ratio = 0.05 clamp (~50px minimum pane size), updates left/right flex_ratio
   - PointerUp: Clears drag_state, logs "splitter drag end" breadcrumb
   - Splitter visual feedback: theme.accent on hover/drag, theme.border_subtle otherwise, col-resize cursor
3. Split pane resize detection and grid overlay:
   - After splitting, collects existing pane triggers before spawning thread: let triggers: Vec<_> = terminal_panes.get_untracked().iter().map(|p| p.trigger).collect()
   - Thread safety fix: Spawns background thread with cloned triggers instead of accessing signals inside thread
   - Spawns 50ms delayed trigger to force repaint: register_ext_trigger() for each collected trigger
   - Also calls view.request_layout() immediately to trigger resize detection
   - Pattern: let triggers = collect_triggers(); view.request_layout(); std::thread::spawn(move || { sleep(50ms); for t in triggers { register_ext_trigger(t); } })
   - Grid size overlay (Ghostty-style visual feedback):
     - Displays "cols x rows" centered on terminal canvas during resize
     - Triggered by canvas size changes detected via prev_canvas_size.get() != current_size
     - Auto-hides after 1 second via background thread timer
     - resize_overlay_visible (RwSignal<bool>) controls visibility, resize_overlay_text (RwSignal<String>) stores "80 x 24" format
     - Overlay rendered via dyn_container checking resize_overlay_visible signal
     - Visual style: Font size 18.0 (increased from 14.0 for better visibility), theme.text color with theme.accent border (1.0px), semi-transparent background (theme.panel_bg @ 95% opacity), border_radius(8.0)
     - Positioning: Uses absolute() + inset_left_pct(50.0) + inset_top_pct(50.0) + margin_left/top offsets for centering (only on inner container when visible)
     - Event handling fix: Outer dyn_container has NO inset positioning, only z_index(100) - prevents blocking terminal input when overlay hidden
     - Empty state: Returns empty().style(|s| s.display(Display::None)) when not visible to ensure no invisible overlay blocks pointer events
     - Previous issue: inset(0.0) on outer container covered entire terminal area, blocking all pointer events even when overlay hidden
     - Background thread: std::thread::spawn(move || { sleep(1s); register_ext_trigger(overlay_hide_trigger); }), only hides if 900ms+ elapsed since last show
     - Debugging logs: "Pane {id}: showing overlay '{text}' (was {prev_cols}x{prev_rows})" at INFO level when PTY resize triggers overlay, "Pane {id}: overlay visible with text '{text}'" breadcrumb when overlay renders
4. focused_pane_id (RwSignal<Option<usize>>) tracks which pane has focus; cursor only shows on focused pane
5. terminal_pane_view() renders individual pane:
   - `canvas()` closure receives paint context and size
   - Calculate grid dimensions from canvas size and cell metrics
   - Build color palette from theme using `TerminalColorList::from_palette()`
   - Debounce PTY resize (50ms threshold) via `ExtSendTrigger` + background thread:
     - Only updates pending_size when grid dimensions actually change (prevents debounce race condition)
     - Spawns 60ms timer (must exceed 50ms threshold) to trigger resize effect
     - Fixed race condition: Previously updated pending_size on every canvas paint even when unchanged, causing debounce check to always fail
     - Debug output via eprintln!: tracks pending vs last sizes, skip reasons (zero size, no change, debounce), execution timing
   - Render using `last_size` (not `pending_size`) to avoid flicker during resize
5. Track render performance via `logging::record_terminal_render()` (per pane)
6. IME cursor positioning: Effect tracks cell_size/cursor changes, anchors IME candidate window at terminal caret using canvas-relative coordinates
7. Force repaint trigger (FORCE_REPAINT_TRIGGER) registered during initialization for external repaint requests
8. Cell rendering loop (per pane):
   - **Padding offset**: All terminal content rendered with 8px padding on each side
     - Cell position: x = 8.0 + col * cell_width, y = 8.0 + row * cell_height
     - Bounds check: x + cell_width > size.width - 7.0 || y + cell_height > size.height - 7.0 (skip cells outside padded area)
     - Relaxed tolerance from 8.0 to 7.0 to fix floating point precision issues causing last row to be cut off
     - Cursor position also uses 8px offset for alignment with content
     - Fixes issue where content rendered at (0,0) instead of accounting for padding
   - **Debug output**: eprintln! used for immediate stderr feedback, bypassing logging system
     - Canvas paint: "[Pane {id}] canvas paint: {w}x{h}px -> {cols}x{rows} grid (was {prev_cols}x{prev_rows}) changed={bool}"
     - Logged on every canvas paint to diagnose resize timing issues
   - **Visual debug borders**: Colored 3px borders drawn on each pane for boundary visualization
     - Colors cycle by pane_id % 4: red (0), green (1), blue (2), yellow (3)
     - Border drawn at top/bottom/left/right edges of canvas using Color::from_rgba8 with alpha=200
     - Helps diagnose pane layout and splitter positioning issues
   - Resolve fg/bg colors from ANSI palette via resolve_fg_color/resolve_bg_color
   - Apply INVERSE flag (swap fg/bg) if set
   - Selection rendering: Uses consistent white background (#FFFFFF) and dark text (#1E1E1E) for seamless appearance
     - Selection rectangles extend by 1px (Rect::new(x, y, x + cell_display_width + 1.0, y + cell_height + 1.0)) to overlap and eliminate sub-pixel gaps between cells
     - Creates unified white selection highlight with no visible grid lines
   - Cursor rendering: Only renders if focused_pane_id.get() == Some(pane.id)
   - Non-selected cells: Only fill background if bg_color differs from default_bg (performance optimization)
   - Wide character support: CJK/emoji use cell_width * 2.0 for proper display
   - Text layout: Uses floem TextLayout with Menlo font, 13px size
9. Scroll handling (PointerWheel events, per pane):
   - scroll_accumulator RwSignal<f64> accumulates sub-line scroll deltas for smooth touchpad scrolling
   - Accumulation: accumulated = scroll_accumulator.get_untracked() + dy
   - Lines calculation: lines = (accumulated / cell_height).trunc() as i32
   - Remainder tracking: accumulated -= (lines as f64) * cell_height after scrolling
   - Scroll direction: Negative dy (scroll up gesture) maps to positive delta (show earlier content) via -lines
   - Logs: "terminal scroll: dy={dy:.1} acc={accumulated:.1} lines={scroll_delta}" when scrolling occurs
   - Requests canvas repaint after scroll via canvas_id.request_paint()
10. Keyboard input with control character support:
   - Regular keys: Matches Key::Character(ch) and writes string directly to PTY via session.write()
   - Control character mapping: Ctrl+key combinations generate ASCII control codes (0x01-0x1A)
     - Ctrl+A = 0x01, Ctrl+B = 0x02, ..., Ctrl+Z = 0x1A
     - Ctrl+C (0x03), Ctrl+D (0x04), Ctrl+L (0x0C), Ctrl+Z (0x1A) commonly used for interrupt/EOF/clear/suspend
     - Implementation: Checks modifiers.control() && ch.len() == 1, converts to uppercase, calculates code = (ch_upper as u8 - b'A' + 1)
     - Writes single control byte to PTY: session.write(&[code]) bypasses normal string encoding
   - Space key support: Key::Named(NamedKey::Space) writes " " string to PTY
   - Special keys: Enter/Backspace handled via Key::Named variants
   - Thread-safe PTY write: session.write() locks internal pty_writer mutex
   - Logs "Terminal write error: {err}" at ERROR level on write failure

**Force Repaint System** (terminal/mod.rs, macOS only):
- FORCE_REPAINT_TRIGGER global (OnceLock<Mutex<Option<ExtSendTrigger>>>) enables external repaint requests
- register_force_repaint_trigger() stores trigger during terminal view initialization (called from terminal_view)
- force_terminal_repaint() clones and registers trigger to force repaint, called by animation timer in layout.rs
- Public export via components/mod.rs (#[cfg(target_os = "macos")] guard, exported alongside terminal_view)
- Logs "force_terminal_repaint: triggered from animation timer" when activated with DEBUG level
- Integration flow: layout.rs WindowResized → start_animation_timer → 1.2s delay → force_terminal_repaint → ExtSendTrigger → terminal canvas repaint for all panes
- Each pane's canvas paint calculates its own grid size from actual canvas dimensions and triggers PTY resize independently
- Thread-safe access via mutex lock with graceful handling if trigger not registered
- Eliminates PTY mismatch issues: Each split pane manages its own session resize based on measured canvas size, no shared global state

**Terminal Selection Color Strategy** (terminal/mod.rs, macOS only):
- Uses consistent selection colors instead of swapping fg/bg colors (eliminates grid/cell effect)
- Selection background: Color::from_rgb8(255, 255, 255) - solid white (#FFFFFF)
- Selection foreground: Color::from_rgb8(30, 30, 30) - dark text (#1E1E1E)
- Implementation: if is_selected block sets fg_color and fills extended background rect with selection_bg before text rendering
  - Rectangle extends by 1px: Rect::new(x, y, x + cell_display_width + 1.0, y + cell_height + 1.0)
  - Overlap prevents sub-pixel gaps between cells, creating seamless white selection highlight
- Rationale: Previous approach (std::mem::swap fg/bg) created visual grid artifacts; consistent colors with 1px overlap provide cleaner UX
- Background fill optimization: Non-selected cells only fill if bg_color != default_bg to reduce overdraw

**Terminal Event Listener** (services/terminal.rs, macOS only):
- TideEventListener implements alacritty_terminal::event::EventListener for terminal event handling
- Constructor: TideEventListener::new(pty_writer, alive, on_title_change) takes Arc<dyn Fn(String) + Send + Sync> callback
- Event handlers:
  - Event::Wakeup: Logs "Terminal wakeup event" at DEBUG level
  - Event::Title(title): Logs title change at DEBUG level, invokes on_title_change callback with OSC title string
  - Event::Exit: Sets alive flag to false, logs exit at INFO level
  - Event::ChildExit(code): Sets alive flag to false, logs exit code at INFO level
  - Event::PtyWrite(text): Writes text to PTY via pty_writer mutex
  - Event::ClipboardStore(_, text): Calls set_clipboard_string() to store clipboard data
  - Event::ClipboardLoad(_, formatter): Retrieves clipboard via get_clipboard_string(), formats, writes to PTY
- Title change callback: Currently no-op (Arc::new(|_| {})), ready for future tab title sync implementation
- Thread-safe: pty_writer uses Arc<Mutex<Box<dyn Write + Send>>>, alive uses Arc<AtomicBool>

**Heartbeat Watchdog** (app.rs):
- Background thread pings UI every 500ms via `ExtSendTrigger`
- Detects stale heartbeat after 2s, dumps breadcrumbs for debugging

**Reactive UI Pattern** (app.rs, model.rs):
- WorkspaceTab fields use RwSignal<T> for automatic UI updates: name, root, file_tree, git_status, terminal_panes (Vec<TerminalPane>), next_pane_id
- TerminalPane struct: id (usize), session (RwSignal<Option<Arc<TerminalSession>>>), trigger (ExtSendTrigger), flex_ratio (RwSignal<f64>)
- Signal reading: Use .get() to read current value (e.g., tab.name.get() returns String)
- Signal writing: Use .set(value) to update and trigger reactive UI updates (e.g., tab_name_signal.set(new_name))
- Reactive view wrappers: file_tree_view_reactive() and git_status_view_reactive() wrap static views in dyn_container
- dyn_container pattern: dyn_container(move || signal.get(), |data| view(data)) watches signal and rebuilds view on change
- dyn_stack pattern: dyn_stack(move || vec_signal.get(), |item| item.id, move |item| view(item)) preserves views by ID, avoids rebuilding unchanged items (use for lists with expensive views like terminal panes)
- dyn_stack vs dyn_container: dyn_stack preserves view identity by key (better for stateful views), dyn_container rebuilds all children on change (simpler for stateless views)
- Reactive style updates: Track RwSignal in style closure (.style(move |s| s.flex_grow(signal.get()))) to update layout without rebuilding view structure
- View functions accepting signals: project_header_view(name: RwSignal<String>, root: RwSignal<PathBuf>) uses move || name.get() in label closures
- Tab button with reactive title: tab_button_with_menu(tab_label: RwSignal<String>, ...) uses move || tab_label.get() for live title updates
- Terminal pane management: dyn_stack preserves terminal_panes views by ID (vs dyn_container rebuilding all), each pane has independent session/trigger/flex_ratio, sessions only created once when None, persist across workspace changes
- Terminal width distribution: flex_ratio defaults to 1.0, adjusted by splitter drag, width = (pane.flex_ratio / total_ratio) * available_width
- Folder picker updates workspace signals: name, root, file_tree (build_tree_entries), git_status (git_status_entries) - terminal panes persist unchanged
- Requires: floem::reactive::{RwSignal, create_effect}

**Context Menu System** (.context_menu() and .popout_menu() patterns):
- **Tab Dropdown Menu** (app.rs, atoms.rs):
  - tab_button_with_menu() uses h_stack with two interactive areas: label text (selects tab) + chevron arrow (shows menu)
  - Label text: Uses .on_click_stop() to handle tab selection without propagating to parent
  - Chevron arrow: CHEVRON_DOWN SVG constant (10x10px), uses .popout_menu() to show dropdown menu on click
  - Arrow color: theme.accent when active, theme.text_muted when inactive
  - Menu items: "打开文件夹" (Open Folder), "在 Finder 中定位" (Reveal in Finder), "关闭" (Close)
  - Open Folder: Uses rfd::FileDialog.pick_folder() to select new workspace, updates tab.root/name/file_tree/git_status dynamically
  - Reveal in Finder: Calls Command::new("open").arg(&root).spawn() to open workspace in macOS Finder
  - Close: Removes tab from tabs vector via tabs.update(), switches active_tab to first remaining tab if closing active
  - All folder operations log to INFO level via logging::log_line()
- **Terminal Context Menu** (terminal/mod.rs, macOS only):
  - Right-click on terminal pane triggers .context_menu() with conditional menu building
  - Menu construction pattern: menu = Menu::new("").entry(...) allows conditional additions
  - Conditional Copy item: Only shows "复制" (Copy) when there's a selection (checks session.with_term(|term| term.selection_to_string()).is_some())
  - Menu items:
    - "复制" (Copy, conditional): session.with_term(|term| term.selection_to_string()) + crate::services::set_clipboard_string(), logs "Terminal: copied selection to clipboard" at INFO level
    - "粘贴" (Paste): crate::services::get_clipboard_string() + sess.write(), normalizes line endings (\r\n → \n, \r → \n), logs "Terminal paste failed: {err}" at ERROR level on failure
    - "向右分割" (Split Right): Creates new pane after current pane, logs "Terminal: Split right from pane {id}", triggers repaint on all panes
    - "向左分割" (Split Left): Creates new pane before current pane, logs "Terminal: Split left from pane {id}", triggers repaint on all panes
    - "重置终端" (Reset Terminal, after separator): sets session signal to None + error_msg signal to None, logs "Terminal: Reset requested" at INFO level
  - Split pane implementation:
    - Increments next_pane_id counter, creates new TerminalPane with default flex_ratio=1.0
    - Inserts new_pane via terminal_panes.update(|panes| panes.insert(idx, new_pane))
    - Calls register_ext_trigger(pane.trigger) for all existing panes to force repaint and recalculate grid sizes
    - Forces existing terminal content to resize and fill available width after split
  - Closure capture: session_for_copy (get_untracked), session_for_reset (clone), error_msg_for_reset for menu actions
  - Each pane has independent context menu; operations apply to clicked pane only
- **Pattern**: Menu::new("").entry(MenuItem::new("label").action(callback)).separator()
- **Requires**: floem::menu::{Menu, MenuItem}, std::process::Command (for tab menu)
- **Note**: .context_menu() for right-click context menus, .popout_menu() for left-click dropdown menus

## Coding Conventions

### Debug Output
- Terminal resize debug: Uses eprintln! for immediate stderr feedback in resize effect (bypasses logging system for low-latency diagnostics)
  - Canvas paint: "[Pane {id}] canvas paint: {w}x{h}px -> {cols}x{rows} grid (was {prev_cols}x{prev_rows}) changed={bool}"
  - Resize effect skip: "[Pane {id}] resize effect: SKIP {reason} ..." (reasons: zero size, no change from last_size, debounce wait < 50ms)
  - Resize effect execute: "[Pane {id}] resize effect: EXECUTING {prev_cols}x{prev_rows} -> {cols}x{rows} (waited {ms}ms)"
  - Scroll events: "[Pane {id}] scroll event: dy={dy:.1}" (raw scroll delta), "[Pane {id}] scroll execute: lines={lines} delta={delta}" (calculated line scroll)
  - Logged on every relevant event to diagnose resize timing and debounce issues

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
  - Usage patterns: UI state transitions (overlay visibility, drag events), context-rich debugging (e.g., "Pane {id}: overlay visible with text '{text}'" tracks which pane shows what grid size)
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
- Pane minimums: LEFT=100, CENTER=100, RIGHT=100 (reduced to 100px to prevent layout overflow in windowed mode, fixes splitter hit-testing issues)
- Pane initial widths: LEFT=200, RIGHT=260 (separate from minimums to allow flexible layout)
- Handle width: HANDLE_WIDTH=1.0px (reduced from 10px for cleaner visual appearance, commit 2a3f96d)
- Terminal splitter width: SPLITTER_WIDTH=6.0px (draggable splitter between terminal panes, wider for easier grabbing)
- Terminal pane flex ratio: Default 1.0 (equal width distribution), min 0.05 during drag (~50px minimum pane size)
- Terminal splitter sensitivity: Dynamic calculation (ratio_delta = delta_x * total_ratio / container_width) for 1:1 pixel mapping, replaces fixed 200px sensitivity from terminal_splitter() function
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
- **rfd** (0.15, macOS): Native file dialog for folder picker in tab context menu

## Known Issues & Workarounds

- macOS window zoom animation captures a snapshot; app cannot repaint during animation
- Animation detection uses fixed 1.2s timer from resize burst start (macOS animation ~1s, events delayed ~2s)
- Background timer thread bypasses event queue delay to ensure timely repaint after animation
- Canvas paint may not be called during resize animation despite `request_paint()` calls
- All pane minimums reduced to 100px to prevent layout overflow in windowed mode (fixes splitter hit-testing)
- Grid overlay event blocking: Fixed by removing inset(0.0) from outer dyn_container and using display:none for empty state - absolute positioning only applied to inner container when visible
- File Explorer panel expand/collapse affects terminal height: Minor visual artifact (±2 rows) due to floem's cross-axis recalculation in flex layouts - attempted fixes broke layout, accepted as tradeoff

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

## Tab Folder Switching Feature (验收清单)

**Location**: Tab bar at top of window

**Interaction**:
1. Each tab shows folder name + dropdown arrow (▼)
2. Click folder name = select tab
3. Click dropdown arrow (▼) = show menu with:
   - "打开文件夹" (Open Folder) - opens native file picker
   - "在 Finder 中定位" (Reveal in Finder) - opens folder in macOS Finder
   - "关闭" (Close) - closes the tab

**Open Folder Behavior**:
- Opens rfd::FileDialog native folder picker
- After selecting new folder:
  - Tab title updates to new folder name (reactive via RwSignal)
  - Left panel file tree updates to show new folder contents
  - Project header (name + path) updates
  - Git status updates
  - Terminal session PRESERVES - does NOT reinitialize (session stored in signal, only created once)

**Implementation Files**:
- src/components/atoms.rs: tab_button_with_menu() with RwSignal<String> for reactive title
- src/app.rs: on_open_folder callback sets signals directly
- src/model.rs: WorkspaceTab with RwSignal fields (name, root, file_tree, git_status)

**Collapsible Panel System** (VSCode-style):
- Each panel (File Explorer, Changes, History) is independently collapsible via collapsible_panel_view()
- Collapse state tracked via RwSignal<bool> per panel (files_expanded, changes_expanded, history_expanded)
- Header always visible with clickable chevron: COLLAPSE_CHEVRON (▼ down arrow) when expanded, EXPAND_CHEVRON (▶ right arrow) when collapsed
- Expanded panels share available vertical space via flex-grow/flex-shrink
- Internal scrolling: each expanded panel scrolls independently, no outer scrollbars
- Layout constraints:
  - project_header: flex_shrink(0.0) - fixed height, never shrinks (prevents tab bar height interference, commit 8453665)
  - Panel header: flex_shrink(0.0) - prevents shrinking when scrollbar appears (commit e42be7d)
  - Expanded panels: min_height(HEADER_HEIGHT + 20.0) ensures header + some content always visible
  - Collapsed panels: height(HEADER_HEIGHT) - only header visible
  - Body container: min_height(0.0) allows shrinking below content height for proper flex layout
  - Outer container: OverflowX/OverflowY::Hidden prevents outer scrollbars, isolates child content overflow
  - flex_basis(0) pattern: Applied to both outer v_stack (line 86) and body_container (line 61) when expanded
    - This is CSS "flex: 1 1 0" - distributes space purely by flex-grow ratios, ignoring content size
    - Prevents File Explorer content height from affecting sibling panel layouts (especially terminal panel)
    - Without flex_basis(0), large file lists cause flex engine to adjust other panels' dimensions
- Implementation: collapsible_panel_view() in src/components/panels.rs, workspace_view() in src/app.rs

**Layout Cross-Axis Coupling** (known floem behavior):
- File Explorer panel expand/collapse causes terminal height to change slightly (e.g., 30 rows → 32 rows)
- Root cause: floem's layout engine recalculates cross-axis dimensions when sibling content changes in flex containers
- Attempted fixes with flex_basis(0) and items_start() broke layout completely (reverted in commit fixing layout isolation)
- Current approach: Accept this minor visual artifact as acceptable tradeoff for stable layout
- Tab bar uses flex_shrink(0.0) to prevent content pressure from child panels affecting its height
- Main layout columns use height_full() and Overflow::Hidden for consistent behavior
- Main layout h_stack uses items_stretch() for proper vertical alignment
