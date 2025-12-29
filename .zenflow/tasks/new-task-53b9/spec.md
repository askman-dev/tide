# Tide Terminal Technical Specification (macOS, alacritty_terminal + portable-pty)

## 1. Technical Context

- **Language & toolchain**
  - Rust 2024 edition application (`tide` crate).
  - UI built with `floem` (from Git, see `Cargo.toml`).
  - Desktop app entrypoint in `src/main.rs`, top-level layout in `src/app.rs`.
- **Existing UI structure**
  - `app::app_view` builds the three-column layout via `components::main_layout`:
    - Left: file tree, git status, history (`panel_view`, `file_tree_view`, `git_status_view`).
    - Center: currently a chat-style workspace (`chat_workspace_view`) occupying the main work area.
    - Right: editor placeholder (`editor_workspace_view`).
  - `UiTheme` (`src/theme.rs`) defines a dark-only palette (surface, panel_bg, element_bg, border_subtle, accent, text, text_muted, text_soft).
- **Model & services**
  - `WorkspaceTab` (in `src/model.rs`) holds `id`, `name`, `root` (workspace root path), `file_tree`, and `git_status`.
  - `services::fs` and `services::git` encapsulate filesystem tree loading and `git status` querying, always parameterized by the workspace root.
  - Logging is centralized in `logging` (used in `app_view` to log file path on startup).
- **New dependencies (planned)**
  - `alacritty_terminal` (local path): terminal state, VT parsing, grid, scrollback, selection, event system.
    - Local path: `swap_memory/alacritty/alacritty_terminal/`.
  - `portable-pty` (local path): PTY allocation and shell process management.
    - Local path: `swap_memory/wezterm/pty/`.
  - Both are MIT/Apache-compatible and suitable for bundling into a desktop app.
- **Platform scope**
  - macOS only for PTY/shell integration in this iteration.
  - Non-macOS builds should compile but use a stub/placeholder terminal view.

## 2. Existing Architecture & Reuse

- **Center workspace column**
  - Currently implemented via `chat_workspace_view` and `editor_workspace_view` in `src/app.rs`, using `main_layout(left, center, right, theme)`.
  - The app already uses monospace font styling in the right editor placeholder (`font_family("SF Mono, Menlo, Monaco")`) and theme colors for backgrounds and borders.
- **Terminal placeholder component**
  - `src/components/terminal.rs` defines `terminal_placeholder(theme: UiTheme, workspace: String) -> impl IntoView`, but this is not yet wired into the layout.
  - This file is the natural home for a real terminal view, preserving the responsibility boundary that “components render UI; services manage IO/state”.
- **Workspace root concept**
  - `build_tab` in `src/app.rs` determines a tab’s root directory and passes it to `services::fs::build_tree_entries` and `services::git::git_status_entries`.
  - The same `WorkspaceTab.root` will be reused as the terminal working directory, ensuring the terminal always operates at the same root as the file explorer and git status.
- **Reusable patterns**
  - **Services modules** in `src/services` encapsulate non-UI work (filesystem, git) and expose simple functions called from UI components.
  - **Theming** is done by injecting a `UiTheme` copy into components, which then apply styling via `style` closures.
  - **Platform-specific behavior** is already handled in `main.rs` for window config using `cfg!(target_os = "macos")`; terminal integration will follow the same pattern.

## 3. Proposed Architecture

### 3.1 New Modules and File Changes

- **New service module**: `src/services/terminal.rs`
  - Responsibilities:
    - Manage PTY + shell process lifecycle (create, resize, shutdown).
    - Own `alacritty_terminal::Term` and its scrollback/selection state.
    - Run the IO thread that reads PTY output and feeds it into the VT parser.
    - Provide a small, thread-safe API surface used by the Floem view.
- **Update components module**:
  - Replace `terminal_placeholder` in `src/components/terminal.rs` with a real `terminal_view`:
    - Signature: `pub fn terminal_view(theme: UiTheme, workspace: WorkspaceTab) -> impl IntoView`.
    - Internally:
      - Lazily create a `TerminalSession` tied to `workspace.root`.
      - Expose a drawing surface that renders `Term` content.
      - Wire keyboard/mouse/clipboard events to the session.
  - Export the new terminal view from `src/components/mod.rs` when building on macOS.
- **Layout integration**:
  - Adjust `chat_workspace_view` / `workspace_view` in `src/app.rs` to include the terminal in the center column, in place of (or in addition to) the current chat placeholder.
  - For non-macOS targets, keep using the existing placeholder (or a reduced terminal placeholder) behind a `#[cfg(not(target_os = "macos"))]` gate.

### 3.2 TerminalSession State Object

- **Struct layout (service layer)**
  - `pub struct TerminalSession {`
    - `term: Arc<FairMutex<Term<TideEventListener>>>`
    - `pty_master: Box<dyn MasterPty + Send>`
    - `pty_writer: Box<dyn Write + Send>`
    - `pty_reader: Box<dyn Read + Send>`
    - `scrollback: usize` (min 500; configurable later).
    - `dimensions: Arc<FairMutex<(u16, u16)>>` (cols, rows).
    - `io_thread: Option<JoinHandle<()>>`
    - `alive: Arc<AtomicBool>`
  - Selection is stored within `Term` itself (via alacritty_terminal APIs).
- **Public API (called from UI)**
  - `fn new(root_dir: &Path, theme: UiTheme, cell_size: CellSize, on_wakeup: impl Fn() + Send + 'static) -> Result<Arc<Self>>`
  - `fn write(&self, bytes: &[u8]) -> io::Result<()>`
  - `fn resize(&self, cols: u16, rows: u16) -> io::Result<()>`
  - `fn renderable_content(&self) -> RenderableContent<'_>` (wrapped to handle locking).
  - `fn scroll_display(&self, delta: i32)` (delegates to `term.scroll_display`).
  - `fn handle_mouse_selection(&self, event: MouseSelectionEvent)` (construct/update `Selection` in `Term`).
  - `fn shutdown(&self)` (signal IO thread & shell process to exit cleanly).
  - `fn copy_selection(&self) -> Option<String>` (wraps `term.selection_to_string()`).
- **Event listener integration**
  - Implement `TideEventListener` wired into `alacritty_terminal`:
    - `Event::Wakeup` → call provided `on_wakeup` closure to request a Floem redraw.
    - `Event::ClipboardStore` → forward text to a clipboard handler (see 4.3).
    - `Event::ClipboardLoad` → request clipboard content and return to terminal.
    - `Event::PtyWrite` → write directly to `pty_writer`.
    - `Event::Exit` → set `alive` to false and notify UI to show “process exited” state.

### 3.3 Threads & Data Flow

- **UI thread (Floem)**
  - Owns the `Arc<TerminalSession>` handle.
  - For each frame:
    - Acquires a read lock on `term` to obtain `RenderableContent` via `term.renderable_content()`.
    - Renders glyphs, cursor, and selection into a dedicated terminal canvas view.
  - Handles:
    - Key events → call `TerminalSession::write` with encoded bytes.
    - Scroll events → `TerminalSession::scroll_display`.
    - Mouse down/move/up → build `Selection` in `Term`.
    - Clipboard shortcuts → call `copy_selection` and `write` as needed.
- **IO thread**
  - Reads from `pty_reader` into a buffer in a blocking loop.
  - For each chunk of bytes:
    - Instantiates/uses `vte::ansi::Processor`.
    - Executes `processor.advance(&mut term, bytes)` while holding a short-lived lock on `term`.
    - The `Term` uses `TideEventListener` to signal wakeups and clipboard requests.
  - Handles EOF and error conditions by:
    - Marking `alive = false`.
    - Sending an `Event::Exit` to the listener.
    - Exiting the thread gracefully.

### 3.4 PTY & Shell Initialization

- **PTY creation**
  - Use `portable_pty::native_pty_system()` (macOS only).
  - Compute `PtySize` from the initial Floem view size:
    - `cols = (view_width / cell_width).floor()` (min 1).
    - `rows = (view_height / cell_height).floor()` (min 1).
  - Create master/slave PTY pair via `pty_system.openpty(PtySize { rows, cols, .. })`.
- **Shell process**
  - Build `CommandBuilder::new_default_prog()` to pick the user’s default shell.
  - Set current working directory via `.cwd(workspace_root)`, using `WorkspaceTab.root`.
  - Inherit environment from the app process; optionally set:
    - `TERM` to a reasonable value (e.g., `xterm-256color`).
    - `COLORTERM` to indicate color support.
  - Spawn the child process attached to the PTY slave and retain a handle for resizing and exit status.
- **Resize handling**
  - Floem view tracks its pixel size and cell metrics; when either changes:
    - Recompute `cols` and `rows`.
    - Call `Term::resize` with new dimensions, preserving scrollback.
    - Call `child.resize(PtySize { rows, cols, .. })` on the PTY child.

## 4. Rendering & Interaction (Floem)

### 4.1 Terminal Canvas View

- **View structure**
  - In `components::terminal_view`, define a terminal container:
    - Uses a Floem drawing view (e.g., a custom `paint`/canvas view) to render content.
    - Expands to fill available space within the center workspace column.
  - The canvas `paint` callback:
    - Locks `TerminalSession.term` briefly to obtain `RenderableContent`.
    - Iterates `display_iter` to draw cells.
    - Draws cursor and selection overlays last.
- **Cell rendering logic**
  - Compute cell pixel positions:
    - `x = col * cell_width`.
    - `y = row * cell_height`.
  - For each visible cell:
    - Resolve background color:
      - Prefer `RenderableCell.bg` (from `RenderableContent.colors` if present).
      - Fallback to theme-derived palette (see 5.1).
    - Resolve foreground color similarly.
    - Draw background rectangle for the cell.
    - Draw the glyph using the configured monospace font:
      - Handle zero-width combining marks by overlaying them on the previous base cell.
  - After cells:
    - Draw the cursor based on `RenderableContent.cursor`:
      - Use a filled rectangle or outline with theme accent color.
    - Draw selection background for all selected cells:
      - Use a slightly brighter or inverted background derived from `UiTheme`.

### 4.2 Input Mapping

- **Keyboard**
  - Floem key events are mapped to bytes:
    - Regular character keys → UTF-8 bytes to PTY via `TerminalSession::write`.
    - Enter ↦ `\r` (carriage return), unless the shell requires otherwise.
    - Backspace ↦ `\x7f`.
    - Tab ↦ `\t`.
    - Arrow keys (Up/Down/Left/Right) ↦ standard ANSI escape sequences.
    - Optional minimal navigation keys (Home, End, PageUp, PageDown) mapped to typical terminal sequences.
  - `Cmd+C` and `Cmd+V` are handled specially (see 4.3), not passed through directly.
- **Mouse & scroll**
  - Mouse wheel/trackpad:
    - Convert scroll delta into `Scroll::Delta(n)` calls on `Term` through `TerminalSession::scroll_display`.
    - When scrollback offset is zero, new output auto-scrolls; when offset > 0, keep viewport pinned until user scrolls back down.
  - Selection:
    - On mouse down:
      - Compute terminal grid coordinates from pixel position.
      - Create `Selection::new(SelectionType::Simple, point, Side::Left)` on `Term`.
    - On mouse move with button pressed:
      - Call `selection.update(point, side)` to extend selection.
    - On mouse up:
      - Finalize selection; no additional action required until `Cmd+C`.

### 4.3 Clipboard Integration

- **Copy (`Cmd+C`)**
  - If `Term` has an active selection:
    - Use `term.selection_to_string()` to retrieve the selected text (respecting line breaks).
    - Write text to system clipboard via Floem’s clipboard utilities or a thin wrapper service (`services::clipboard` if needed).
    - Do **not** send `^C` to PTY.
  - If no selection:
    - Send `^C` (ETX, `0x03`) to PTY via `TerminalSession::write`, matching common terminal behavior.
- **Paste (`Cmd+V`)**
  - Retrieve string from system clipboard.
  - Normalize line endings to `\n` and write to PTY.
  - Let the shell handle input semantics (multi-line commands, etc.).
- **alacritty_terminal events**
  - Where `alacritty_terminal` emits `ClipboardStore`/`ClipboardLoad` events, integrate them with the same clipboard abstraction used for `Cmd+C` / `Cmd+V`, ensuring consistent behavior across both keyboard shortcuts and terminal-driven clipboard actions.

## 5. Colors, Theme, and Fonts

### 5.1 ANSI Palette Mapping

- **Palette design**
  - Define a `TerminalPalette` mapping from `UiTheme` to:
    - Normal colors (indices 0–7).
    - Bright colors (indices 8–15).
    - Default foreground/background.
  - Implementation:
    - Introduce a helper function in `src/theme.rs`, e.g., `fn terminal_palette(theme: UiTheme) -> TerminalPalette`.
    - Map:
      - Background ↦ `theme.surface` or `theme.panel_bg`.
      - Foreground ↦ `theme.text`.
      - Muted/soft variants ↦ `theme.text_muted`, `theme.text_soft`.
      - Accent color ↦ `theme.accent` (used for cursor/selection and bright ANSI colors).
- **alacritty_terminal integration**
  - Convert `TerminalPalette` into whatever color configuration structure `alacritty_terminal` expects.
  - When calling `Term::new`, pass:
    - `Config { scrolling_history: 500, colors: palette, ..Default::default() }`.
  - Honor per-cell color overrides from `RenderableContent.colors` when present, falling back to the palette only when no override is specified.

### 5.2 Theme Toggle Readiness

- Implementation remains dark-theme-only for now but:
  - `TerminalPalette` is computed from `UiTheme`, not global statics.
  - When a future app-level theme toggle exists, the terminal view:
    - Receives the updated `UiTheme`.
    - Recomputes `TerminalPalette`.
    - Triggers a redraw, applying the new colors without reinitializing `Term` or PTY.

### 5.3 Font and Metrics

- **Font selection**
  - Use a monospace font family consistent with the existing editor placeholder:
    - `"SF Mono, Menlo, Monaco"`.
  - Configure the terminal canvas view with this font via Floem styling.
- **Cell sizing**
  - Use Floem text measurement utilities to determine:
    - Cell width: width of a typical glyph (e.g., `'W'` or `'M'`).
    - Cell height: line height from the current font and font size.
  - Store `CellSize { width, height }` alongside `TerminalSession`.
  - Use `CellSize` for:
    - Row/column calculation (for PTY size).
    - Mapping mouse events from pixel coordinates to grid positions.
    - Aligning background rectangles and cursor rendering.

## 6. Platform Gating and Error Handling

- **macOS vs. non-macOS**
  - Wrap all `portable-pty` usage in `#[cfg(target_os = "macos")]`.
  - Provide a minimal, no-op `TerminalSession` stub and a simple placeholder view for non-macOS builds that shows a message like “Terminal is only available on macOS”.
- **Error surfaces**
  - PTY creation failure:
    - Return a user-visible error state from `TerminalSession::new`.
    - Terminal view renders a panel with the error and suggests checking system permissions.
  - Shell spawn failure:
    - Show exit message including non-zero status or error string.
  - IO errors:
    - Log details using `logging::log_line`.
    - Transition to “session ended” state in the UI, displaying “Process exited” with an exit code if available.
  - Resilience:
    - Ensure UI never panics due to `Term` lock poisoning; handle lock acquisition errors gracefully (e.g., reset session, show error).

## 7. Delivery Phases (Incremental, Testable Milestones)

1. **Phase 1 – Dependency wiring and scaffolding**
   - Add path dependencies for `alacritty_terminal` and `portable-pty` in `Cargo.toml`, gated by `cfg(target_os = "macos")` where necessary.
   - Create `src/services/terminal.rs` with `TerminalSession` skeleton and unit tests for basic construction (without PTY yet).
   - Ensure app still builds and runs with the existing placeholder terminal view.
2. **Phase 2 – PTY and Term integration (headless)**
   - Implement PTY creation and shell spawning in `TerminalSession::new`, using a fixed rows/cols for initial testing.
   - Integrate `Term::new` and an IO thread that feeds PTY bytes into `vte::ansi::Processor`.
   - Add simple log-based verification (e.g., echo command output to logs) to confirm PTY ↔ Term wiring without rendering.
3. **Phase 3 – Rendering and input**
   - Replace `terminal_placeholder` with `terminal_view` that renders `RenderableContent` into a Floem canvas.
   - Implement core keyboard input and scrollback behavior.
   - Validate that `scrolling_history >= 500` and that scrollback does not auto-snap when not at bottom.
4. **Phase 4 – Selection, clipboard, and colors**
   - Implement mouse-based selection in the grid and `Term` selection APIs.
   - Wire `Cmd+C` / `Cmd+V` to system clipboard integration.
   - Implement `TerminalPalette` in `theme.rs` and apply ANSI color mapping to rendering.
5. **Phase 5 – Lifecycle, error handling, and macOS gating**
   - Add resize handling based on view size changes.
   - Implement clear “process exited” states and optional restart action.
   - Finalize platform gating and non-macOS placeholders.
   - Perform performance checks with typical workloads (e.g., running `cargo test`, viewing logs).

## 8. Verification Approach

- **Automated commands**
  - `cargo check` – primary command to validate type correctness across all platforms.
  - `cargo test` – currently there are no tests; we can add focused unit tests in `services::terminal` for:
    - Configuration and cell-dimension calculations.
    - Mapping between pixel coordinates and grid positions.
  - `cargo build --release` (on macOS) – ensure release builds succeed with the new dependencies and gating.
- **Manual verification (macOS)**
  - Launch Tide and open a workspace:
    - Confirm a terminal appears in the center workspace column.
    - Run interactive commands (`ls`, `git status`, `cargo test`) and confirm output matches a regular terminal.
  - Scrollback:
    - Produce >500 lines of output and verify scrollback works and is smooth.
  - Selection and clipboard:
    - Select multiple lines with the mouse, press `Cmd+C`, and paste into an external editor.
    - Press `Cmd+V` with content on the clipboard and verify it appears in the terminal and executes as expected.
  - Theme consistency:
    - Check that terminal background, foreground, accent, and selection colors visually align with existing panels and text styles in Tide.
  - Lifecycle:
    - Exit the shell (e.g., `exit`) and confirm the UI shows a “process exited” message and remains responsive.
    - Restart the terminal if a restart affordance is implemented.

