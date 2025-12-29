# Tide Terminal PRD (macOS, alacritty_terminal + portable-pty)

## 1. Overview

Add a real, interactive terminal to the center workspace column of Tide on macOS using open-source components:

- `alacritty_terminal` for terminal state, VT parsing, grid, scrollback, and selection.
- `portable-pty` for PTY allocation and shell process management.

The new terminal replaces the current placeholder view in `src/components/terminal.rs:terminal_placeholder` and is scoped to macOS only for this iteration.

## 2. Goals

- Provide a fully interactive shell session (backed by a PTY) inside the center workspace column.
- Support scrollback of at least 500 lines (ideally configurable).
- Support mouse-based text selection.
- Support `Cmd+C` to copy selected text to the system clipboard.
- Support `Cmd+V` to paste system clipboard text into the terminal.
- Render ANSI colors consistent with the app’s light/dark theme (default dark theme already exists in `UiTheme`).
- Use a monospace font with fixed cell sizing derived from font metrics.
- Integrate cleanly with existing Floem-based layout and theme, without disrupting other panels.
- Keep the implementation macOS-only for now (Linux/Windows are out-of-scope).

## 3. Non‑Goals

- Cross-platform support (Linux/Windows) in this iteration.
- Advanced terminal features: tabs, splits, multiple concurrent sessions, search, hyperlinking, or complex keybindings beyond a minimal useful subset.
- Full configuration UI for terminal settings (shell selection, color schemes, scrollback tuning, etc.).
- Shell-specific integration (e.g., zsh/bash/fish customization, login shell profiles beyond whatever the OS/config provides).
- Persisting terminal history across app restarts.
- General-purpose text editor behavior; this is strictly an interactive PTY terminal.

## 4. Users and Use Cases

**Primary users**

- Tide users (developers) on macOS who want an integrated terminal anchored to the current workspace.

**Core use cases**

- Run build/test commands (`cargo`, `npm`, etc.) from the workspace root without leaving Tide.
- Run git commands and other CLI tools in context of the current workspace.
- Quickly inspect and manipulate files via CLI (e.g., `ls`, `cat`, `rg`) alongside the Tide UI.
- Copy output from commands (logs, error messages) via mouse selection and `Cmd+C`.
- Paste commands/snippets from clipboard into the terminal via `Cmd+V`.

## 5. UX & Interaction Requirements

### 5.1 Placement & Layout

- The terminal lives in the center workspace column, occupying the existing terminal area where `terminal_placeholder` is currently rendered.
- It should expand to fill the available space in its container, respecting the surrounding layout (panels, headers).
- Default behavior: a single terminal session per workspace view (no tabs or splits).

### 5.2 Text Rendering & Visuals

- Use a monospace font (e.g., “SF Mono, Menlo, Monaco” or system-equivalent) for all terminal glyphs.
- Use fixed cell sizing based on font metrics (cell width/height).
- Render all characters from `alacritty_terminal::Term` via `RenderableContent.display_iter`:
  - Draw per-cell background (using ANSI or theme-mapped colors).
  - Draw glyphs (including zero-width combining characters).
  - Render cursor in the correct position based on `RenderableContent.cursor`.
  - Render text selection as a background overlay (using selection colors).
- The terminal background and foreground colors must visually align with the rest of the Tide UI (dark theme by default).

### 5.3 Scrolling & Scrollback

- Terminal maintains at least 500 lines of scrollback.
- Mouse wheel/trackpad scroll must scroll through scrollback, using `term.scroll_display(Scroll::Delta(n))` or equivalent.
- When new output arrives and the user is not scrolled up (no active scrollback offset), the viewport should follow the cursor (auto-scroll).
- When the user scrolls up into history, new output must not forcibly snap to bottom until the user scrolls back down.

### 5.4 Selection & Clipboard

- Mouse selection:
  - Click and drag to select text in the grid using `Selection::new` and `selection.update`.
  - Selection should be visually obvious (background highlight).
  - Selection can span multiple lines.
- Copy:
  - On `Cmd+C`, if there is an active selection, copy `term.selection_to_string()` to the system clipboard.
  - `Cmd+C` should not send `^C` to the PTY when a selection exists.
  - Behavior when no selection exists:
    - **Assumption (to verify):** Send `^C` to the PTY (typical terminal behavior).
- Paste:
  - On `Cmd+V`, read from system clipboard and write text into the PTY.
  - Pasted text should respect terminal input semantics (e.g., handle newlines correctly).

### 5.5 Keyboard Input

- Map normal text input directly into bytes written to the PTY writer.
- Provide sensible default mappings for:
  - Enter (`\r` or `\n` as appropriate for shell, likely `\r`).
  - Backspace (`\x7f`).
  - Arrow keys and a minimal set of navigation keys via ANSI escape sequences.
- Avoid breaking normal shell behavior; shortcuts not explicitly handled (like `Ctrl+C`) should pass through to the PTY as raw control characters.

### 5.6 Theme & Colors

- Support at least a dark theme, consistent with `UiTheme::new()` (current default).
- Plan for a future app-level theme toggle (light/dark) and ensure the terminal can switch palettes without re-architecting.
- Define an ANSI color palette for:
  - Normal colors (0–7).
  - Bright colors (8–15).
  - Default foreground/background.
- Map terminal colors to app theme colors; when `RenderableContent.colors` provides overrides, respect those overrides.
- The terminal must re-render when theme changes (if/when a theme toggle is added).

### 5.7 Session Lifecycle

- Terminal session is created when the terminal view is instantiated:
  - Create PTY using `portable_pty::native_pty_system()`.
  - Compute `PtySize` based on view size and cell metrics (rows/cols).
  - Spawn default shell with `CommandBuilder::new_default_prog()`.
  - Set working directory to the workspace root so commands run in context.
- Resize:
  - When the UI size changes, compute new rows/cols and:
    - Call `Term::resize()`.
    - Notify PTY of resize (via `child.resize()` or equivalent).
- Exit/Restart:
  - When the shell exits, display a clear indication (e.g., “Process exited with code X”).
  - Provide a simple way to restart the session (e.g., button or automatic respawn) — exact UX can be minimal but must avoid a “dead” blank area.

## 6. Technical Requirements & Constraints

### 6.1 Platform

- macOS only for PTY and shell execution in this iteration.
- Code should be written so that non-macOS platforms can later compile with the terminal feature disabled or stubbed.

### 6.2 Dependencies & Integration

- Use `alacritty_terminal` from local path (planned under `swap_memory/alacritty/alacritty_terminal/`).
- Use `portable-pty` from local path (planned under `swap_memory/wezterm/pty/`).
- Continue using Floem as the UI framework for rendering the terminal grid and capturing input.
- Ensure no licensing conflicts (both are MIT/Apache-compatible).

### 6.3 Architecture & Data Flow

- `TerminalSession` state object:
  - `term: Arc<FairMutex<Term<EventProxy>>>` (or equivalent Tide mutex abstraction).
  - `pty_master: Box<dyn MasterPty>`.
  - `pty_writer: Box<dyn Write + Send>`.
  - `pty_reader: Box<dyn Read + Send>`.
  - `scrollback: usize` (configured to at least 500).
  - Selection stored on `Term`.
- Threads:
  - UI thread (Floem): renders terminal grid, handles input/selection, sends writes to PTY.
  - IO thread: reads from PTY, feeds bytes into `alacritty_terminal::Term` via `vte::ansi::Processor::advance(&mut term, bytes)`.
- Rendering:
  - Use `term.renderable_content()` to obtain `RenderableContent`.
  - Iterate `display_iter` to render cells and background rectangles to the Floem canvas.
  - Render cursor and selection after cell rendering.

### 6.4 Event Handling

- Implement an event proxy/listener for terminal events:
  - `Event::Wakeup` → request a Floem redraw of the terminal view.
  - `Event::ClipboardStore` / `Event::ClipboardLoad` → integrate with macOS clipboard APIs (via existing or new abstraction).
  - `Event::PtyWrite` → write bytes to PTY writer.
  - `Event::Exit` → mark session as finished and trigger UI update (e.g., show status).

### 6.5 Error Handling & Reliability

- If PTY creation fails:
  - Show an error message in the terminal area and avoid crashing the app.
- If shell spawn fails:
  - Show a meaningful error with exit status or error text.
- IO thread must handle read errors gracefully and terminate cleanly on EOF.
- Avoid UI-thread panics due to unexpected terminal states or PTY errors.

## 7. Performance & Quality

- The terminal should feel responsive:
  - Visual updates within ~1 frame of PTY output arriving.
  - Keypress-to-echo latency should be comparable to standalone terminals for typical workloads.
- Floem rendering must handle typical terminal line counts without noticeable lag, even with scrollback up to at least 500 lines.
- Avoid excessive CPU usage when idle (no busy loops in IO or rendering).

## 8. Open Questions & Assumptions

- **Workspace root mapping**: Confirm how “workspace root” is determined in Tide and ensure the terminal’s working directory uses the same concept.
- **Cmd+C without selection**:
  - Assumption: send `^C` to PTY when there is no active selection, and only treat `Cmd+C` as copy when selection exists.
- **Theme toggle**:
  - Currently only a dark theme appears to be defined (`UiTheme::new()`).
  - Assumption: for this iteration, implement a dark-only palette with a design that can extend to light mode later.
- **Multiple terminals**:
  - Assumption: one terminal instance per workspace view is sufficient; no requirement for multiple sessions/tabs in this iteration.
- **Persistence**:
  - Assumption: terminal state (scrollback, running process) does not persist across app restarts or workspace switching.

These assumptions should be confirmed or adjusted before the Technical Specification and implementation planning stages.

