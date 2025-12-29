# Full SDD workflow

## Configuration
- **Artifacts Path**: {@artifacts_path} → `.zenflow/tasks/{task_id}`

---

## Workflow Steps

### [x] Step: Requirements
<!-- chat-id: 7cc91629-f401-44fe-ae65-cf487be13f44 -->

Create a Product Requirements Document (PRD) based on the feature description.

1. Review existing codebase to understand current architecture and patterns
2. Analyze the feature definition and identify unclear aspects
3. Ask the user for clarifications on aspects that significantly impact scope or user experience
4. Make reasonable decisions for minor details based on context and conventions
5. If user can't clarify, make a decision, state the assumption, and continue

Save the PRD to `{@artifacts_path}/requirements.md`.

### [x] Step: Technical Specification
<!-- chat-id: 26035415-8c28-4bd2-9b28-82608dbc69b9 -->

Create a technical specification based on the PRD in `{@artifacts_path}/requirements.md`.

1. Review existing codebase architecture and identify reusable components
2. Define the implementation approach

Save to `{@artifacts_path}/spec.md` with:
- Technical context (language, dependencies)
- Implementation approach referencing existing code patterns
- Source code structure changes
- Data model / API / interface changes
- Delivery phases (incremental, testable milestones)
- Verification approach using project lint/test commands

### [x] Step: Planning
<!-- chat-id: 7b27a435-0629-43fe-8104-15e7de51ee31 -->

Create a detailed implementation plan based on `{@artifacts_path}/spec.md`.

1. Break down the work into concrete tasks
2. Each task should reference relevant contracts and include verification steps
3. Replace the Implementation step below with the planned tasks

Rule of thumb for step size: each step should represent a coherent unit of work (e.g., implement a component, add an API endpoint, write tests for a module). Avoid steps that are too granular (single function) or too broad (entire feature).

If the feature is trivial and doesn't warrant full specification, update this workflow to remove unnecessary steps and explain the reasoning to the user.

Save to `{@artifacts_path}/plan.md`.

### [x] Step: Wire terminal dependencies and platform gating
<!-- chat-id: 6264d9a2-f60c-4a27-bce3-af708db9b49f -->

- Update `Cargo.toml` to add path dependencies for `alacritty_terminal` and `portable-pty` (`swap_memory/alacritty/alacritty_terminal`, `swap_memory/wezterm/pty`) as described in spec §1 and §3.1.
- Introduce macOS-only `cfg(target_os = "macos")` wiring for terminal services/components and lightweight non-macOS stubs that compile but show a placeholder message (spec §2, §6).
- Verification: run `cargo check` on macOS and ensure the project also compiles for non-macOS targets (at least via `cargo check --target` where available).

### [x] Step: Implement core TerminalSession, PTY, and Term wiring
<!-- chat-id: fa711579-61f7-4899-91e5-0d1ccfd398d6 -->

- Create `src/services/terminal.rs` with the `TerminalSession` struct owning `Term<TideEventListener>`, PTY handles, scrollback configuration, and basic lifecycle as outlined in spec §3.2–3.4.
- Implement PTY creation via `portable_pty::native_pty_system`, shell spawning with `CommandBuilder::new_default_prog()`, and an IO thread that feeds PTY output into `vte::ansi::Processor` and `Term` (spec §3.4).
- Expose a small, thread-safe API (`new`, `write`, `resize`, `scroll_display`, `snapshot_renderable`/equivalent) for the UI layer; add unit tests for configuration (scrollback ≥ 500) and basic construction (spec §3.2, §7.2).
- Verification: `cargo test` for `services::terminal` unit tests and log-based manual check that shell output flows into `Term` (e.g., echo commands logged via `logging` as per spec §2, §3.3).

### [x] Step: Build Floem terminal view and integrate into center workspace
<!-- chat-id: e84ff0f7-ff4d-4033-b5f6-c2dae4de5e4e -->

- Replace `terminal_placeholder` in `src/components/terminal.rs` with a real `terminal_view(theme: UiTheme, workspace: WorkspaceTab) -> impl IntoView` that creates/owns a `TerminalSession` and renders its content (spec §3.1, §4.1).
- Implement a canvas-style view that, for each frame, obtains `RenderableContent` from `Term`, iterates `display_iter`, and draws cells, cursor, and selection using the app monospace font (spec §4.1–4.2, §5.3).
- Export the terminal view from `src/components/mod.rs` and update the center column layout in `src/app.rs` to show the terminal for macOS builds while retaining a placeholder for non-macOS (spec §2, §3.1, §6).
- Verification: run the app on macOS, confirm the terminal appears in the center workspace column, displays shell prompts/output, and updates as commands run.

### [x] Step: Implement keyboard, mouse, and scrollback behavior
<!-- chat-id: a55f0f04-d24f-4945-a851-d73b236a78a6 -->

- Map Floem key events to terminal input: printable characters, Enter (`\r`), Backspace (`\x7f`), Tab, arrow keys, and minimal navigation keys using ANSI sequences, while reserving `Cmd+C`/`Cmd+V` for clipboard handling (spec §4.2, §4.3).
- Implement scrollback and viewport control via `Term::scroll_display(Scroll::Delta(..))`, ensuring scrollback history is at least 500 lines and that manual scroll offsets are respected (spec §3.3, §4.2, §7.3).
- Wire mouse down/move/up events to `Term` selection APIs (`Selection::new`, `selection.update`) using grid coordinates derived from cell metrics (spec §3.3, §4.2, §5.3).
- Verification: manual testing in the running app to confirm typing, navigation keys, scroll wheel/trackpad behavior, and text selection all behave like a typical terminal with sufficient scrollback.

### [x] Step: Add clipboard integration (Cmd+C/Cmd+V) and alacritty clipboard events
<!-- chat-id: e22bd658-b078-410b-9189-97dea4c4b996 -->

- Implement a clipboard abstraction (using Floem utilities or a small `services::clipboard` module) that can be called from both UI shortcuts and `alacritty_terminal` clipboard events (spec §4.3).
- For `Cmd+C`, copy current selection from `Term` via `selection_to_string()` to the system clipboard; when no selection is active, send `^C` (`0x03`) to the PTY instead (spec §4.3).
- For `Cmd+V`, read text from the system clipboard, normalize line endings, and write to the PTY writer, letting the shell handle multi-line inputs (spec §4.3).
- Integrate `Event::ClipboardStore`/`ClipboardLoad` from `TideEventListener` with the same clipboard abstraction to keep behavior consistent (spec §3.3, §4.3).
- Verification: manual tests copying from the terminal into an external editor and pasting into the terminal, plus confirming `Cmd+C` sends SIGINT when no selection is present.

### [x] Step: Implement theme-aware ANSI palette, font metrics, and rendering polish
<!-- chat-id: ca892dea-b792-4b23-900c-829739101297 -->

- Define a `TerminalPalette` in `src/theme.rs` derived from `UiTheme`, mapping normal/bright colors and default foreground/background for the terminal (spec §5.1–5.2).
- Pass the palette into `Term::new` via its config and honor per-cell color overrides from `RenderableContent` when rendering, falling back to the palette when needed (spec §3.3, §5.1).
- Ensure the terminal canvas uses the monospace font family `"SF Mono, Menlo, Monaco"` and compute `CellSize { width, height }` via Floem text measurement to drive PTY sizing, cursor placement, and mouse hit-testing (spec §5.3).
- Verification: `cargo check` and manual visual inspection that terminal colors align with the app’s dark theme, glyphs are aligned to a consistent grid, and resizing preserves correct row/column mapping.


### [x] Step: Finalize lifecycle, error handling, and platform gating behavior
<!-- chat-id: e25984a0-eb1f-4557-bf7e-01319c10847e -->

- Handle PTY and shell process shutdown: respond to EOF and `Event::Exit` by marking the session inactive and updating the UI with a “process exited” message and optional restart affordance (spec §3.3, §3.4, §6).
- Implement robust error handling for PTY creation, shell spawn, IO errors, and lock failures, logging via `logging::log_line` and ensuring the UI remains responsive with a visible error state instead of panicking (spec §2, §6).
- Verify that macOS and non-macOS builds follow their respective paths (real terminal vs. placeholder) and that the app handles workspace changes and window resizes correctly (spec §3.4, §6, §7.5).
- Verification: `cargo test` (including any added tests for coordinate/size helpers), `cargo build --release` on macOS, and manual scenarios such as exiting the shell, resizing the window, and checking non-macOS compilation.
