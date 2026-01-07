# Project Context

## Purpose
Tide is a terminal-based IDE built with Rust and the Floem UI framework. It features a three-pane resizable layout with interactive terminal emulation (macOS only), comprehensive logging/diagnostics, and a watchdog system for detecting UI hangs.

## Tech Stack
- **Language**: Rust (Edition 2021)
- **UI Framework**: Floem (git dependency from lapce/floem, pinned to rev `e0dd862`)
- **Terminal Emulation**: `alacritty_terminal` (0.25.1)
- **PTY Management**: `portable-pty` (0.9.0)
- **Clipboard**: `arboard` (3.x)
- **File Dialog**: `rfd` (0.15)

## Project Conventions

### Code Style
- **Formatting**: Rust standard formatting (`rustfmt`).
- **Debug Output**: Use `eprintln!` for immediate stderr feedback in resize effects (bypasses logging system for low-latency diagnostics).
- **Logging**:
    - Use `logging::log_line(level, msg)` for persistent logging.
    - Use `logging::breadcrumb(msg)` for lightweight event tracking (circular buffer).
    - `logging::init()` must be called at startup.

### Architecture Patterns
- **Reactive UI**: Extensive use of Floem's `RwSignal`, `create_effect`, `dyn_container`, and `dyn_stack`.
- **Custom View Implementation**: Views implement `View` trait with `id()`, `view_style()`, `event_before_children()`.
- **Terminal Rendering**: Custom rendering flow with `terminal_view`, `SplitterDragState`, and `TerminalInstanceState`.
- **Animation Detection**: Background thread timer (1.2s) to handle macOS window zoom animation delays.
- **Split Pane Architecture**: Independent canvas size calculation per pane; no global session state for cell sizes.

### Testing Strategy
- Standard `cargo test`.
- Manual verification for UI interactions (e.g., split panes, resize overlays).
- Verify `cargo build` and `cargo build --release`.

### Git Workflow
- Follow standard feature branch workflow.
- Commit messages should be clear and descriptive.

## Domain Context
- **Terminal Emulation**: Understanding of PTYs, ANSI escape codes, and grid rendering is crucial.
- **Floem Framework**: Knowledge of signals, effects, and view composition in Floem is required.
- **macOS Specifics**: The project has significant macOS-specific handling for window animations and terminal features.

## Important Constraints
- **Platform**: Terminal features are currently `#[cfg(target_os = "macos")]` only.
- **Floem Version**: Must stay pinned to `e0dd862` to avoid performance regression (2s delay after zoom).
- **Performance**: Strict budgets for UI events (<50ms) and rendering (<50ms).
- **UI Layout**: All pane minimums set to 100px to prevent layout overflow in windowed mode.

## External Dependencies
- **Lapce/Floem**: The UI framework source.