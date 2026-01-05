mod colors;
mod constants;
mod instance;
#[cfg(target_os = "macos")]
mod panel;

use crate::components::atoms::meta_text;
use crate::model::{TerminalPane, WorkspaceTab};
use crate::services::TerminalSession;
use crate::theme::UiTheme;

#[cfg(target_os = "macos")]
use crate::theme::TerminalPalette;
use floem::prelude::*;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
use colors::{
    TerminalColorList, background_brush, cursor_brush, resolve_bg_color, resolve_fg_color,
};

#[cfg(target_os = "macos")]
use instance::TerminalInstanceState;

#[cfg(target_os = "macos")]
use panel::{SplitterDragState, calculate_splitter_drag, DRAG_STATE_SENTINEL};

#[cfg(target_os = "macos")]
use constants::{
    CELL_PADDING, OVERLAY_MIN_VISIBLE_MS, OVERLAY_SHOW_DURATION_MS, PTY_RESIZE_DEBOUNCE_MS,
    SPLITTER_WIDTH, SPLIT_SECOND_WAVE_MS, SPLIT_TRIGGER_DELAY_MS, TERMINAL_FONT_SIZE,
    terminal_font_families,
};

#[cfg(target_os = "macos")]
use crate::logging;

#[cfg(target_os = "macos")]
use alacritty_terminal::{
    grid::{Dimensions, Indexed},
    index::{Column, Side},
    selection::{Selection, SelectionType},
    term::{
        cell::Flags,
        point_to_viewport,
        viewport_to_point,
        RenderableContent,
    },
    vte::ansi::{Color as AnsiColor, CursorShape as AnsiCursorShape, NamedColor},
};

#[cfg(target_os = "macos")]
use floem::{
    event::{Event, EventListener, EventPropagation},
    ext_event::{register_ext_trigger, ExtSendTrigger},
    keyboard::{Key, NamedKey},
    menu::{Menu, MenuItem},
    peniko::{
        kurbo::Rect,
        Brush, Color,
    },
    reactive::{create_effect, RwSignal},
    text::{Attrs, AttrsList, TextLayout},
};

#[cfg(target_os = "macos")]
use std::sync::Arc;

#[cfg(target_os = "macos")]
use std::time::{Duration, Instant};

#[cfg(target_os = "macos")]
use std::sync::{Mutex, OnceLock};

/// Global trigger for forcing terminal repaint from WindowResized events.
/// This allows layout.rs to bypass the normal canvas paint flow during macOS animations.
#[cfg(target_os = "macos")]
static FORCE_REPAINT_TRIGGER: OnceLock<Mutex<Option<ExtSendTrigger>>> = OnceLock::new();

/// Register a trigger that can be used to force terminal repaint from external code.
#[cfg(target_os = "macos")]
pub fn register_force_repaint_trigger(trigger: ExtSendTrigger) {
    let mutex = FORCE_REPAINT_TRIGGER.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = mutex.lock() {
        *guard = Some(trigger);
    }
}

/// Force a terminal repaint by triggering the registered ExtSendTrigger.
/// Called from layout.rs when animation timer expires.
/// Each terminal pane's canvas will recalculate its own grid size correctly.
#[cfg(target_os = "macos")]
pub fn force_terminal_repaint() {
    if let Some(mutex) = FORCE_REPAINT_TRIGGER.get() {
        if let Ok(guard) = mutex.lock() {
            if let Some(ref trigger) = *guard {
                let t: ExtSendTrigger = trigger.clone();
                register_ext_trigger(t);
                logging::log_line("DEBUG", "force_terminal_repaint: triggered from animation timer");
            }
        }
    }
}

/// Platform-gated terminal view entry point.
///
/// On macOS this hosts the real PTY-backed terminal backed by
/// `alacritty_terminal` and `portable-pty`.
/// On non-macOS platforms it shows a simple placeholder message.
#[cfg(target_os = "macos")]
pub fn terminal_view(theme: UiTheme, workspace: WorkspaceTab) -> impl IntoView {
    use floem::style::CursorStyle;

    let workspace_name = workspace.name;
    let workspace_root = workspace.root;
    let terminal_panes = workspace.terminal_panes;
    let next_pane_id = workspace.next_pane_id;

    // Track which pane is focused (for cursor visibility)
    let focused_pane_id: RwSignal<Option<usize>> = RwSignal::new(None);

    // Track splitter drag state at parent level (not inside dyn_stack)
    // This prevents pane views from being rebuilt when drag state changes
    let drag_state: RwSignal<SplitterDragState> = RwSignal::new(None);

    // Use dyn_stack to preserve pane views when adding/removing panes
    // Each pane includes a splitter on its right side (handled in pane view)
    let panes_stack = dyn_stack(
        move || terminal_panes.get(),
        |pane| pane.id,
        move |pane| {
            let pane_id = pane.id;
            let pane_flex_ratio = pane.flex_ratio;

            // Check if this is the last pane (no splitter needed after it)
            let is_last = move || {
                let panes = terminal_panes.get();
                panes.last().map_or(true, |last| last.id == pane_id)
            };

            let pane_view = terminal_pane_view(
                theme,
                pane.clone(),
                workspace_root,
                terminal_panes,
                next_pane_id,
                focused_pane_id,
            );

            // Splitter element (only visible if not last pane)
            let splitter = container(empty())
                .style(move |s| {
                    let show = !is_last();
                    let is_dragging = drag_state.get().map_or(false, |(id, _)| id == pane_id);
                    s.display(if show { floem::style::Display::Flex } else { floem::style::Display::None })
                        .width(SPLITTER_WIDTH)
                        .height_full()
                        .background(if is_dragging { theme.accent } else { theme.border_subtle })
                        .cursor(CursorStyle::ColResize)
                        .hover(|s| s.background(theme.accent.with_alpha(0.5)))
                })
                .on_event(EventListener::PointerDown, move |event| {
                    if let Event::PointerDown(pointer_event) = event {
                        if pointer_event.button.is_primary() {
                            // Check that there's a next pane to resize with
                            let panes = terminal_panes.get_untracked();
                            let idx = panes.iter().position(|p| p.id == pane_id);

                            if let Some(i) = idx {
                                if panes.get(i + 1).is_some() {
                                    // Use sentinel to indicate first move hasn't happened yet
                                    drag_state.set(Some((pane_id, DRAG_STATE_SENTINEL)));
                                    logging::breadcrumb(format!("splitter drag start: pane {pane_id}"));
                                    return EventPropagation::Stop;
                                }
                            }
                        }
                    }
                    EventPropagation::Continue
                });

            // Combine pane and splitter in a horizontal stack
            h_stack((pane_view, splitter))
                .style(move |s| {
                    let flex = pane_flex_ratio.get();
                    s.flex_basis(0.0).flex_grow(flex as f32).height_full()
                })
        },
    )
    .style(|s| s.flex_row().width_full().height_full());

    // Capture the panes_stack ViewId so we can get its width during drag
    let panes_stack_id = panes_stack.id();

    // Add event handlers for drag
    let panes_stack = panes_stack
        .on_event(EventListener::PointerMove, move |event| {
            if let Event::PointerMove(pointer_event) = event {
                if let Some((left_pane_id, last_x)) = drag_state.get_untracked() {
                    let current_x = pointer_event.pos.x;

                    // First move after PointerDown: just record position, don't resize yet
                    if last_x == DRAG_STATE_SENTINEL {
                        drag_state.set(Some((left_pane_id, current_x)));
                        return EventPropagation::Stop;
                    }

                    // Calculate incremental delta (same coordinate system)
                    let delta_x = current_x - last_x;

                    // Update last_x for next move
                    drag_state.set(Some((left_pane_id, current_x)));

                    let panes = terminal_panes.get_untracked();
                    if let Some(idx) = panes.iter().position(|p| p.id == left_pane_id) {
                        if let Some(right_pane) = panes.get(idx + 1) {
                            let left_ratio = panes[idx].flex_ratio.get_untracked();
                            let right_ratio = right_pane.flex_ratio.get_untracked();

                            // Get actual container width for accurate sensitivity
                            let container_rect = panes_stack_id.layout_rect();
                            let container_width = container_rect.width();

                            // Use panel helper for drag calculation
                            if let Some((new_left, new_right)) = calculate_splitter_drag(
                                delta_x,
                                left_ratio,
                                right_ratio,
                                container_width,
                            ) {
                                panes[idx].flex_ratio.set(new_left);
                                right_pane.flex_ratio.set(new_right);
                            }
                        }
                    }

                    return EventPropagation::Stop;
                }
            }
            EventPropagation::Continue
        })
        .on_event(EventListener::PointerUp, move |_| {
            if drag_state.get_untracked().is_some() {
                logging::breadcrumb("splitter drag end".to_string());
                drag_state.set(None);
                // Collect triggers before spawning thread (RwSignal is not Send)
                let triggers: Vec<_> = terminal_panes.get_untracked()
                    .iter()
                    .map(|p| p.trigger.clone())
                    .collect();
                // Delay trigger to allow layout to settle after drag ends
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(PTY_RESIZE_DEBOUNCE_MS));
                    for trigger in triggers {
                        register_ext_trigger(trigger);
                    }
                });
                return EventPropagation::Stop;
            }
            EventPropagation::Continue
        });

    v_stack((
        label(|| "Terminal").style(move |s| {
            s.font_size(12.0)
                .font_bold()
                .color(theme.text_muted)
        }),
        meta_text(format!("Workspace: {}", workspace_name.get()), theme),
        container(panes_stack).style(move |s| {
            s.width_full()
                .flex_grow(1.0)  // Fill remaining height
                .padding(8.0)
                .background(theme.panel_bg)
                .border(1.0)
                .border_color(theme.border_subtle)
        }),
    ))
    .style(|s| s.width_full().height_full().row_gap(8.0))
}


/// Render a single terminal pane
#[cfg(target_os = "macos")]
fn terminal_pane_view(
    theme: UiTheme,
    pane: TerminalPane,
    workspace_root: RwSignal<PathBuf>,
    terminal_panes: RwSignal<Vec<TerminalPane>>,
    next_pane_id: RwSignal<usize>,
    focused_pane_id: RwSignal<Option<usize>>,
) -> impl IntoView {
    let session = pane.session;
    let term_update_trigger = pane.trigger;
    let pane_id = pane.id;

    // Bundle all instance state signals
    let state = TerminalInstanceState::new();

    // Create local bindings for frequently used signals (avoids repetitive `state.` prefix)
    let error_msg = state.error_msg;
    let last_size = state.last_size;
    let pending_size = state.pending_size;
    let last_resize_request = state.last_resize_request;
    let resize_trigger = state.resize_trigger.clone();
    let cell_size = state.cell_size;
    let cell_y_offset = state.cell_y_offset;
    let last_pty_resize_at = state.last_pty_resize_at;
    let ime_focused = state.ime_focused;
    let ime_update_tick = state.ime_update_tick;
    let last_ime_cursor_area = state.last_ime_cursor_area;
    let last_canvas_size = state.last_canvas_size;
    let scroll_accumulator = state.scroll_accumulator;
    let resize_overlay_visible = state.resize_overlay_visible;
    let resize_overlay_text = state.resize_overlay_text;
    let overlay_show_time = state.overlay_show_time;
    let overlay_hide_trigger = state.overlay_hide_trigger.clone();

    // Register the resize trigger globally so layout.rs can force repaint after animation
    register_force_repaint_trigger(resize_trigger.clone());

    let terminal_canvas = canvas({
        move |cx, size| {
            let workspace_root = workspace_root.get_untracked();
            let font_families = terminal_font_families();
            let render_start = Instant::now();
            let mut rendered_cells: usize = 0;

            // 1. Draw background
            let bg_rect = Rect::new(0.0, 0.0, size.width, size.height);
            let bg_brush = background_brush(theme);
            cx.fill(&bg_rect, &bg_brush, 0.0);

            // 2. Check for error state
            if let Some(err) = error_msg.get_untracked() {
                let attrs = Attrs::new()
                    .color(Color::from_rgb8(235, 87, 87))
                    .font_size(14.0)
                    .family(&font_families);
                let mut layout = TextLayout::new();
                layout.set_text(&format!("Error: {}", err), AttrsList::new(attrs), None);
                let text_size = layout.size();
                let x = (size.width - text_size.width) / 2.0;
                let y = (size.height - text_size.height) / 2.0;
                cx.draw_text(&layout, floem::kurbo::Point::new(x, y));
                return;
            }

            // 3. Initialize session if needed
            let mut current_session = session.get_untracked();
            if current_session.is_none() {
                // Callback to trigger repaint from IO thread
                let notify = {
                    let term_update_trigger = term_update_trigger;
                    Arc::new(move || {
                        register_ext_trigger(term_update_trigger);
                    })
                };

                match TerminalSession::new(&workspace_root, notify) {
                    Ok(new_session) => {
                        current_session = Some(new_session.clone());
                        session.set(Some(new_session));
                    }
                    Err(err) => {
                        let msg = format!("Failed to start terminal session: {err}");
                        crate::logging::log_line("ERROR", &msg);
                        error_msg.set(Some(msg));
                        // Force repaint to show error
                        return; // Will repaint next frame due to signal change if tracked, or we rely on events.
                    }
                }
            }

            let Some(session) = current_session else {
                return;
            };

            // 4. Measure (and cache) cell size.
            // Recalculate on every canvas size change to ensure accurate layout.
            // TextLayout measurement is fast enough to do this every frame during resize.
            let (prev_canvas_width, prev_canvas_height) = last_canvas_size.get_untracked();
            let canvas_size_changed = (size.width - prev_canvas_width).abs() > 1.0
                || (size.height - prev_canvas_height).abs() > 1.0;
            
            let (mut cell_width, mut cell_height) = cell_size.get_untracked();
            
            // Ensure cell metrics are always up-to-date
            if canvas_size_changed || cell_width <= 0.0 || cell_height <= 0.0 {
                let attrs = Attrs::new()
                    .font_size(TERMINAL_FONT_SIZE)
                    .family(&font_families);
                let base_attrs_list = AttrsList::new(attrs);
                let mut metrics_layout = TextLayout::new();
                metrics_layout.set_text("m", base_attrs_list, None);
                let metrics_size = metrics_layout.size();
                cell_width = metrics_size.width.max(1.0);
                cell_height = (metrics_size.height * 1.25).max(1.0);

                cell_size.set((cell_width, cell_height));
                cell_y_offset.set((cell_height - metrics_size.height) / 2.0);
                last_canvas_size.set((size.width, size.height));
            }

            let y_offset = cell_y_offset.get_untracked();

            // 5. Calculate terminal grid size and trigger resize if needed
            // Subtract padding (8px on each side = 16px total)
            let available_width = (size.width - 16.0).max(1.0);
            let available_height = (size.height - 16.0).max(1.0);
            let cols = (available_width / cell_width).floor().max(1.0) as u16;
            let rows = (available_height / cell_height).floor().max(1.0) as u16;

            // Check if grid size changed
            let (last_cols, last_rows) = last_size.get_untracked();
            let size_changed = cols != last_cols || rows != last_rows;

            // DEBUG: Log canvas size on every paint to diagnose resize issues
            logging::log_line(
                "DEBUG",
                &format!(
                    "[Pane {}] canvas paint: {:.0}x{:.0}px -> {}x{} grid (was {}x{}) changed={}",
                    pane_id, size.width, size.height, cols, rows, last_cols, last_rows, size_changed
                ),
            );

            if size_changed {
                // Only update pending_size and spawn timer if the pending size is actually changing
                // This prevents debounce race condition where repeated paints keep resetting last_resize_request
                let (pending_cols, pending_rows) = pending_size.get_untracked();
                let pending_changed = pending_cols != cols || pending_rows != rows;

                if pending_changed {
                    pending_size.set((cols, rows));
                    last_resize_request.set(Instant::now());

                    logging::breadcrumb(format!(
                        "grid size changed: {}x{} -> {}x{} (canvas {:.0}x{:.0})",
                        last_cols, last_rows, cols, rows, size.width, size.height
                    ));

                    // Spawn a timer to trigger PTY resize later (debounce)
                    // Wait 60ms to ensure debounce check (50ms) passes
                    let trigger = resize_trigger.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(60));
                        register_ext_trigger(trigger);
                    });
                }
            }

            // Always render with current calculated size (not last_size)
            // This ensures immediate visual update even before PTY resize completes

            // 6. Render content
            let palette = TerminalPalette::for_theme(theme);
            let palette_list = TerminalColorList::from_palette(&palette);

            session.with_term(|term| {
                // DEBUG: Log PTY's actual grid size vs canvas calculated size
                let pty_cols = term.columns();
                let pty_rows = term.screen_lines();
                if pty_cols != cols as usize || pty_rows != rows as usize {
                    logging::log_line(
                        "WARN",
                        &format!(
                            "[Pane {}] MISMATCH: PTY={}x{} vs Canvas={}x{}",
                            pane_id, pty_cols, pty_rows, cols, rows
                        ),
                    );
                }

                let mut content: RenderableContent<'_> = term.renderable_content();
                let selection = content.selection;
                let cursor = content.cursor;
                let term_colors = content.colors;
                let default_bg = theme.panel_bg;
                let mut text = String::with_capacity(8);
                let mut cell_layout = TextLayout::new();
                let has_selection = selection.is_some();

                for indexed in content.display_iter.by_ref() {
                    rendered_cells += 1;
                    let viewport_point =
                        match point_to_viewport(content.display_offset, indexed.point) {
                            Some(p) => p,
                            None => continue,
                        };

                    let col = viewport_point.column.0 as f64;
                    let row = viewport_point.line as f64;
                    // Add padding offset
                    let x = CELL_PADDING + col * cell_width;
                    let y = CELL_PADDING + row * cell_height;

                    // Skip cells outside the available area (accounting for padding)
                    // Use small tolerance (1.0) to avoid floating point precision issues cutting off last row
                    if x + cell_width > size.width - (CELL_PADDING - 1.0) || y + cell_height > size.height - (CELL_PADDING - 1.0) {
                        continue;
                    }

                    let cell = indexed.cell;
                    let flags = cell.flags;
                    
                    // Skip wide char spacer cells - they are just placeholders
                    if flags.contains(Flags::WIDE_CHAR_SPACER) {
                        continue;
                    }

                    // Fast-path: default background + whitespace with no selection.
                    // This is the common case (especially after a big resize), and
                    // skipping it avoids per-cell color resolution and text layout.
                    if !has_selection
                        && !flags.contains(Flags::INVERSE)
                        && cell.zerowidth().is_none()
                        && cell.c.is_whitespace()
                        && matches!(cell.bg, AnsiColor::Named(NamedColor::Background))
                    {
                        continue;
                    }

                    let is_selected = selection.as_ref().map_or(false, |range| {
                        range.contains_cell(
                            &Indexed { point: indexed.point, cell },
                            cursor.point,
                            cursor.shape,
                        )
                    });

                    let mut fg_color =
                        resolve_fg_color(term_colors, &palette_list, &cell.fg, flags);
                    let mut bg_color =
                        resolve_bg_color(term_colors, &palette_list, &cell.bg);

                    if flags.contains(Flags::INVERSE) {
                        std::mem::swap(&mut fg_color, &mut bg_color);
                    }

                    // Check if this is a wide character (CJK, emoji, etc.)
                    let is_wide = flags.contains(Flags::WIDE_CHAR);
                    let cell_display_width = if is_wide { cell_width * 2.0 } else { cell_width };

                    // Selection uses consistent colors for clean appearance (no grid effect)
                    if is_selected {
                        // White background, dark text for selection
                        // Extend rect by 1px to eliminate sub-pixel gaps between cells
                        let selection_bg = Color::from_rgb8(255, 255, 255);
                        let selection_fg = Color::from_rgb8(30, 30, 30);
                        fg_color = selection_fg;
                        let bg_brush = Brush::from(selection_bg);
                        let cell_rect = Rect::new(x, y, x + cell_display_width + 1.0, y + cell_height + 1.0);
                        cx.fill(&cell_rect, &bg_brush, 0.0);
                    } else if bg_color != default_bg {
                        // Non-selected cells: only fill if bg differs from default
                        let bg_brush = Brush::from(bg_color);
                        let cell_rect = Rect::new(x, y, x + cell_display_width, y + cell_height);
                        cx.fill(&cell_rect, &bg_brush, 0.0);
                    }

                    // Skip text layout/draw for empty cells.
                    if cell.c.is_whitespace() && cell.zerowidth().is_none() {
                        continue;
                    }

                    text.clear();
                    text.push(cell.c);
                    if let Some(extra) = cell.zerowidth() {
                        for ch in extra {
                            text.push(*ch);
                        }
                    }

                    let attrs = Attrs::new()
                        .color(fg_color)
                        .font_size(TERMINAL_FONT_SIZE)
                        .family(&font_families);
                    let attrs_list = AttrsList::new(attrs);

                    cell_layout.set_text(&text, attrs_list, None);
                    
                    // For wide characters, ensure we don't clip the text
                    if is_wide {
                        cx.save();
                        cx.draw_text(&cell_layout, floem::kurbo::Point::new(x, y + y_offset));
                        cx.restore();
                    } else {
                        cx.draw_text(&cell_layout, floem::kurbo::Point::new(x, y + y_offset));
                    }
                }

                // Cursor - only show on focused pane
                let is_focused = focused_pane_id.get_untracked() == Some(pane_id);
                if cursor.shape != AnsiCursorShape::Hidden && session.is_active() && is_focused {
                    if let Some(viewport_cursor) =
                        point_to_viewport(content.display_offset, cursor.point)
                    {
                        let col = viewport_cursor.column.0 as f64;
                        let row = viewport_cursor.line as f64;
                        // Add padding offset
                        let x = CELL_PADDING + col * cell_width;
                        let y = CELL_PADDING + row * cell_height;
                        let cursor_rect = Rect::new(x, y, x + cell_width, y + cell_height);
                        let brush = cursor_brush(theme);
                        cx.fill(&cursor_rect, &brush, 0.0);
                    }
                }
            });

            // 7. Render "Session Ended" overlay if inactive
            if !session.is_active() {
                let overlay_rect = Rect::new(0.0, 0.0, size.width, size.height);
                let overlay_color = Color::from_rgba8(0, 0, 0, 150); // Semi-transparent black
                cx.fill(&overlay_rect, &Brush::from(overlay_color), 0.0);

                let attrs = Attrs::new()
                    .color(Color::WHITE)
                    .font_size(16.0)
                    .weight(floem::text::Weight::BOLD)
                    .family(&font_families);
                let mut layout = TextLayout::new();
                layout.set_text("Session Ended", AttrsList::new(attrs), None);
                let text_size = layout.size();
                let x = (size.width - text_size.width) / 2.0;
                let y = (size.height - text_size.height) / 2.0 - 10.0;
                cx.draw_text(&layout, floem::kurbo::Point::new(x, y));

                let attrs_sub = Attrs::new()
                    .color(Color::from_rgb8(200, 200, 200))
                    .font_size(12.0)
                    .family(&font_families);
                let mut layout_sub = TextLayout::new();
                layout_sub.set_text("Press Enter to Restart", AttrsList::new(attrs_sub), None);
                let sub_size = layout_sub.size();
                let sx = (size.width - sub_size.width) / 2.0;
                let sy = y + text_size.height + 8.0;
                cx.draw_text(&layout_sub, floem::kurbo::Point::new(sx, sy));
            }

            logging::record_terminal_render(
                render_start.elapsed(),
                rendered_cells,
                cols,
                rows,
            );
        }
    });

    let canvas_id = terminal_canvas.id();

    // Track if we're in selection mode (primary button held)
    let is_selecting = RwSignal::new(false);

    // Wrap canvas to track window_origin
    let terminal_wrapper = terminal_canvas
        .on_event_cont(EventListener::WindowGotFocus, move |_| {
            // Dummy event to trigger update and capture window_origin
        })
        .style(|s| s.width_full().height_full());

    // Custom update to track window_origin
    let terminal_wrapper_id = terminal_wrapper.id();
    
    // Effect to trigger repaint when session/error state or focus changes
    create_effect(move |_| {
        session.track();
        error_msg.track();
        term_update_trigger.track();
        focused_pane_id.track();  // Repaint when focus changes to show/hide cursor
        // Request layout first so canvas can detect new size, then repaint
        canvas_id.request_layout();
        canvas_id.request_paint();
    });

    // Effect to update IME cursor position based on terminal cursor
    create_effect(move |_| {
        cell_size.track();
        session.track();
        term_update_trigger.track();
        ime_focused.track();
        ime_update_tick.track();
        
        let (cell_width, cell_height) = cell_size.get_untracked();
        let canvas_rect = terminal_wrapper_id.layout_rect();
        
        if !ime_focused.get_untracked() {
            last_ime_cursor_area.set(None);
            return;
        }

        if cell_width <= 0.0 || cell_height <= 0.0 || canvas_rect.width() <= 0.0 {
            return;
        }

        let Some(sess) = session.get_untracked() else {
            return;
        };
        if !sess.is_active() {
            return;
        }

        let next = sess.with_term(|term| {
            let content = term.renderable_content();
            let cursor = content.cursor;
            let display_offset = content.display_offset;

            let viewport_cursor = point_to_viewport(display_offset, cursor.point)?;
            let col = viewport_cursor.column.0 as f64;
            let row = viewport_cursor.line as f64;

            // `canvas_rect` is window-relative; anchor the IME at the caret cell rect.
            let x = canvas_rect.x0 + col * cell_width;
            let y = canvas_rect.y0 + row * cell_height;
            Some((
                floem::kurbo::Point::new(x, y),
                floem::kurbo::Size::new(cell_width, cell_height),
            ))
        });

        let Some((pos, size)) = next else {
            return;
        };

        let should_send = match last_ime_cursor_area.get_untracked() {
            None => true,
            Some((prev_pos, prev_size)) => {
                let moved = (prev_pos.x - pos.x).abs() >= 0.5 || (prev_pos.y - pos.y).abs() >= 0.5;
                let resized = (prev_size.width - size.width).abs() >= 0.5
                    || (prev_size.height - size.height).abs() >= 0.5;
                moved || resized
            }
        };

        if should_send {
            floem::action::set_ime_cursor_area(pos, size);
            last_ime_cursor_area.set(Some((pos, size)));
        }
    });

    // Effect to handle PTY resize asynchronously (debounced)
    create_effect(move |_| {
        let effect_start = Instant::now();
        resize_trigger.track();
        let (pending_cols, pending_rows) = pending_size.get_untracked();
        let (last_cols, last_rows) = last_size.get_untracked();

        // DEBUG: Log effect entry
        logging::log_line(
            "DEBUG",
            &format!(
                "[Pane {}] resize effect: pending={}x{} last={}x{}",
                pane_id, pending_cols, pending_rows, last_cols, last_rows
            ),
        );

        if pending_cols == 0 || pending_rows == 0 {
            logging::log_line("DEBUG", &format!("[Pane {}] resize effect: SKIP (pending is zero)", pane_id));
            return;
        }

        // If nothing changed, don't bother
        if pending_cols == last_cols && pending_rows == last_rows {
            logging::log_line("DEBUG", &format!("[Pane {}] resize effect: SKIP (no change)", pane_id));
            return;
        }

        // Debounce check
        let last_request = last_resize_request.get_untracked();
        let debounce_wait = last_request.elapsed();
        if debounce_wait < Duration::from_millis(PTY_RESIZE_DEBOUNCE_MS) {
            logging::log_line(
                "DEBUG",
                &format!(
                    "[Pane {}] resize effect: SKIP debounce (waited {}ms < {}ms)",
                    pane_id, debounce_wait.as_millis(), PTY_RESIZE_DEBOUNCE_MS
                ),
            );
            return;
        }

        logging::log_line(
            "DEBUG",
            &format!(
                "[Pane {}] resize effect: EXECUTING {}x{} -> {}x{} (waited {}ms)",
                pane_id, last_cols, last_rows, pending_cols, pending_rows, debounce_wait.as_millis()
            ),
        );

        if let Some(sess) = session.get_untracked() {
            logging::breadcrumb(format!(
                "terminal pty resize call (debounced): {pending_cols}x{pending_rows} (was {last_cols}x{last_rows})"
            ));
            let resize_start = Instant::now();
            if let Err(err) = sess.resize(pending_cols, pending_rows) {
                crate::logging::log_line(
                    "ERROR",
                    &format!("Terminal resize failed: {err}"),
                );
            } else {
                let resize_ms = resize_start.elapsed().as_micros() as f64 / 1000.0;
                last_size.set((pending_cols, pending_rows));
                last_pty_resize_at.set(Instant::now());
                let effect_ms = effect_start.elapsed().as_micros() as f64 / 1000.0;
                crate::logging::log_line(
                    "DEBUG",
                    &format!(
                        "PTY resized: {}x{} (was {}x{}) resize={:.2}ms effect_total={:.2}ms",
                        pending_cols, pending_rows, last_cols, last_rows, resize_ms, effect_ms
                    ),
                );

                // Show grid size overlay (like Ghostty)
                let overlay_text = format!("{} x {}", pending_cols, pending_rows);
                logging::log_line("INFO", &format!(
                    "Pane {}: showing overlay '{}' (was {}x{})",
                    pane_id, overlay_text, last_cols, last_rows
                ));
                resize_overlay_text.set(overlay_text);
                resize_overlay_visible.set(true);
                overlay_show_time.set(Instant::now());

                // Hide overlay after timeout (unless another resize happens)
                let hide_trigger = overlay_hide_trigger.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(OVERLAY_SHOW_DURATION_MS));
                    register_ext_trigger(hide_trigger);
                });
            }
        }
    });

    // Effect to hide overlay when triggered (only if minimum visible time has passed)
    create_effect(move |_| {
        overlay_hide_trigger.track();
        let show_time = overlay_show_time.get_untracked();
        // Only hide if minimum visible time has passed since last show
        // This prevents premature hiding when multiple resize events happen
        if show_time.elapsed() >= std::time::Duration::from_millis(OVERLAY_MIN_VISIBLE_MS) {
            resize_overlay_visible.set(false);
        }
    });

    let terminal_wrapper = terminal_wrapper
        .keyboard_navigable()
        .style(move |s| {
            s.width_full()
                .height_full()
                .background(theme.panel_bg)
        })
        .on_event_cont(EventListener::WindowResized, move |_| {
            ime_update_tick.update(|tick| *tick = tick.wrapping_add(1));
            // Force canvas repaint on window resize to avoid stale content during animation
            canvas_id.request_layout();
            canvas_id.request_paint();
        })
        .on_event(EventListener::FocusGained, move |_| {
            logging::breadcrumb(format!("terminal pane {} focus gained", pane_id));
            ime_focused.set(true);
            ime_update_tick.update(|tick| *tick = tick.wrapping_add(1));
            floem::action::set_ime_allowed(true);
            focused_pane_id.set(Some(pane_id));
            EventPropagation::Continue
        })
        .on_event(EventListener::FocusLost, move |_| {
            logging::breadcrumb(format!("terminal pane {} focus lost", pane_id));
            ime_focused.set(false);
            floem::action::set_ime_allowed(false);
            // Only clear if we were the focused pane
            if focused_pane_id.get_untracked() == Some(pane_id) {
                focused_pane_id.set(None);
            }
            EventPropagation::Continue
        })
        .on_event_cont(EventListener::ImePreedit, move |_| {
            ime_update_tick.update(|tick| *tick = tick.wrapping_add(1));
        })
        .on_event(EventListener::KeyDown, move |event| {
            logging::measure_ui_event("terminal keydown", || {
                if let Event::KeyDown(key_event) = event {
                    let key = &key_event.key.logical_key;
                    match key {
                        Key::Named(named) => {
                            logging::breadcrumb(format!("terminal keydown: {named:?}"));
                        }
                        Key::Character(text) => {
                            logging::breadcrumb(format!(
                                "terminal keydown: char len={}",
                                text.len()
                            ));
                        }
                        _ => {}
                    }
                    ime_update_tick.update(|tick| *tick = tick.wrapping_add(1));

                    // Check for restart if session is inactive
                    if let Some(session_arc) = session.get_untracked() {
                        if !session_arc.is_active() {
                            if matches!(key, Key::Named(NamedKey::Enter)) {
                                logging::breadcrumb("terminal restart".to_string());
                                session.set(None);
                                error_msg.set(None);
                                return EventPropagation::Stop;
                            }
                            return EventPropagation::Stop; // Consume keys when dead
                        }
                    }

                    // Check for restart if we have an error (e.g. failed to start)
                    if error_msg.get_untracked().is_some() {
                         if matches!(key, Key::Named(NamedKey::Enter)) {
                            logging::breadcrumb("terminal restart (error)".to_string());
                            session.set(None);
                            error_msg.set(None);
                            return EventPropagation::Stop;
                        }
                    }

                    let Some(session) = session.get_untracked() else {
                        return EventPropagation::Continue;
                    };

                    let modifiers = key_event.modifiers;

                    // Handle Cmd+C / Cmd+V for clipboard integration.
                    if modifiers.meta() {
                        if let Key::Character(ch) = key {
                            if ch.eq_ignore_ascii_case("c") {
                                let selection =
                                    session.with_term(|term| term.selection_to_string());

                                if let Some(text) = selection {
                                    crate::services::set_clipboard_string(&text);
                                } else if let Err(err) =
                                    session.write(&[0x03])
                                {
                                    crate::logging::log_line(
                                        "ERROR",
                                        &format!(
                                            "Terminal write failed for Cmd+C: {err}"
                                        ),
                                    );
                                }

                                return EventPropagation::Stop;
                            } else if ch.eq_ignore_ascii_case("v") {
                                if let Some(text) =
                                    crate::services::get_clipboard_string()
                                {
                                    let normalized = text
                                        .replace("\r\n", "\n")
                                        .replace('\r', "\n");

                                    if !normalized.is_empty() {
                                        if let Err(err) =
                                            session.write(normalized.as_bytes())
                                        {
                                            crate::logging::log_line(
                                                "ERROR",
                                                &format!(
                                                    "Terminal write failed for Cmd+V: {err}"
                                                ),
                                            );
                                        }
                                    }
                                }

                                return EventPropagation::Stop;
                            }
                        }
                    }

                    // Handle Ctrl+key for control characters (Ctrl+C=0x03, Ctrl+Z=0x1A, etc.)
                    if modifiers.control() {
                        if let Key::Character(ch) = key {
                            // Get first char and convert to control code
                            if let Some(c) = ch.chars().next() {
                                let upper = c.to_ascii_uppercase();
                                if upper >= 'A' && upper <= 'Z' {
                                    let ctrl_code = (upper as u8) - b'A' + 1;
                                    logging::breadcrumb(format!(
                                        "terminal ctrl+{}: sending 0x{:02x}",
                                        c, ctrl_code
                                    ));
                                    if let Err(err) = session.write(&[ctrl_code]) {
                                        crate::logging::log_line(
                                            "ERROR",
                                            &format!(
                                                "Terminal write failed for Ctrl+{}: {err}",
                                                c
                                            ),
                                        );
                                    }
                                    return EventPropagation::Stop;
                                }
                            }
                        }
                    }

                    let mut handled = false;

                    match key {
                        Key::Character(text) => {
                            if !text.is_empty() {
                                if let Err(err) = session.write(text.as_bytes()) {
                                    crate::logging::log_line(
                                        "ERROR",
                                        &format!("Terminal write failed: {err}"),
                                    );
                                }
                                handled = true;
                            }
                        }
                        Key::Named(named) => {
                            let bytes: Option<&[u8]> = match named {
                                NamedKey::Enter => Some(b"\r"),
                                NamedKey::Space => Some(b" "),
                                NamedKey::Tab => Some(b"\t"),
                                NamedKey::Backspace => Some(&[0x7f]),
                                NamedKey::Escape => Some(b"\x1b"),
                                NamedKey::ArrowUp => Some(b"\x1b[A"),
                                NamedKey::ArrowDown => Some(b"\x1b[B"),
                                NamedKey::ArrowRight => Some(b"\x1b[C"),
                                NamedKey::ArrowLeft => Some(b"\x1b[D"),
                                NamedKey::Home => Some(b"\x1b[H"),
                                NamedKey::End => Some(b"\x1b[F"),
                                NamedKey::PageUp => Some(b"\x1b[5~"),
                                NamedKey::PageDown => Some(b"\x1b[6~"),
                                NamedKey::Delete => Some(b"\x1b[3~"),
                                _ => None,
                            };

                            if let Some(bytes) = bytes {
                                if let Err(err) = session.write(bytes) {
                                    crate::logging::log_line(
                                        "ERROR",
                                        &format!("Terminal write failed: {err}"),
                                    );
                                }
                                handled = true;
                            }
                        }
                        _ => {}
                    }

                    if handled {
                        EventPropagation::Stop
                    } else {
                        EventPropagation::Continue
                    }
                } else {
                    EventPropagation::Continue
                }
            })
        })
        .on_event(EventListener::ImeCommit, move |event| {
            logging::measure_ui_event("terminal ime commit", || {
                if let Event::ImeCommit(text) = event {
                    logging::breadcrumb(format!("terminal ime commit: len={}", text.len()));
                    ime_update_tick.update(|tick| *tick = tick.wrapping_add(1));
                    
                    let Some(session) = session.get_untracked() else {
                        return EventPropagation::Continue;
                    };
                    
                    if !session.is_active() {
                        return EventPropagation::Stop;
                    }
                    
                    if !text.is_empty() {
                        if let Err(err) = session.write(text.as_bytes()) {
                            crate::logging::log_line(
                                "ERROR",
                                &format!("Terminal IME write failed: {err}"),
                            );
                        }
                    }
                    
                    return EventPropagation::Stop;
                }
                EventPropagation::Continue
            })
        })
        .on_event(EventListener::PointerWheel, move |event| {
            logging::measure_ui_event("terminal scroll", || {
                let Some(session) = session.get_untracked() else {
                    return EventPropagation::Continue;
                };
                if !session.is_active() { return EventPropagation::Continue; }

                let (_, cell_height) = cell_size.get_untracked();
                if cell_height <= 0.0 {
                    return EventPropagation::Continue;
                }

                if let Event::PointerWheel(wheel_event) = event {
                    let dy = wheel_event.delta.y;
                    logging::log_line("DEBUG", &format!("[Pane {}] scroll event: dy={:.1}", pane_id, dy));

                    // Accumulate scroll delta for smooth touchpad scrolling
                    // Small gestures build up until they reach a full line
                    let mut accumulated = scroll_accumulator.get_untracked() + dy;

                    // Calculate whole lines from accumulated scroll
                    let lines = (accumulated / cell_height).trunc() as i32;

                    if lines != 0 {
                        // Subtract the lines we're scrolling from accumulator
                        accumulated -= (lines as f64) * cell_height;
                        scroll_accumulator.set(accumulated);

                        // Negate: scrolling up (negative dy) should show earlier content (positive delta)
                        let scroll_delta = -lines;
                        logging::log_line("DEBUG", &format!("[Pane {}] scroll execute: lines={} delta={}", pane_id, lines, scroll_delta));
                        session.scroll_display(scroll_delta);
                        canvas_id.request_paint();
                    } else {
                        // Just accumulate, no scroll yet
                        scroll_accumulator.set(accumulated);
                    }

                    return EventPropagation::Stop;
                }

                EventPropagation::Continue
            })
        })
        .on_event(EventListener::PointerDown, move |event| {
            logging::measure_ui_event("terminal pointer down", || {
                let Some(session) = session.get_untracked() else {
                    return EventPropagation::Continue;
                };

                let (cell_width, cell_height) = cell_size.get_untracked();
                if cell_width <= 0.0 || cell_height <= 0.0 {
                    return EventPropagation::Continue;
                }

                if let Event::PointerDown(pointer_event) = event {
                    if !pointer_event.button.is_primary() {
                        return EventPropagation::Continue;
                    }

                    let pos = pointer_event.pos;
                    {
                        logging::breadcrumb("terminal pointer down".to_string());
                        canvas_id.request_focus();
                        is_selecting.set(true);

                        let x = pos.x;
                        let y = pos.y;

                        session.with_term_mut(|term| {
                            let (cols, lines, display_offset) = {
                                let grid = term.grid();
                                (grid.columns(), grid.screen_lines(), grid.display_offset())
                            };

                            if cols == 0 || lines == 0 {
                                return;
                            }

                            let col = (x / cell_width).floor() as isize;
                            let line = (y / cell_height).floor() as isize;

                            if col < 0 || line < 0 {
                                return;
                            }

                            let mut col = col as usize;
                            let mut line = line as usize;

                            if col >= cols {
                                col = cols - 1;
                            }
                            if line >= lines {
                                line = lines - 1;
                            }

                            let viewport_point =
                                alacritty_terminal::index::Point::<usize, Column>::new(
                                    line,
                                    Column(col),
                                );
                            let term_point =
                                viewport_to_point(display_offset, viewport_point);

                            term.selection = Some(Selection::new(
                                SelectionType::Simple,
                                term_point,
                                Side::Left,
                            ));
                        });

                        canvas_id.request_paint();
                        return EventPropagation::Stop;
                    }
                }

                EventPropagation::Continue
            })
        })
        .on_event(EventListener::PointerMove, move |event| {
            logging::measure_ui_event("terminal pointer move", || {
                let Some(session) = session.get_untracked() else {
                    return EventPropagation::Continue;
                };
                // Allow selection even if inactive? Yes, why not.

                let (cell_width, cell_height) = cell_size.get_untracked();
                if cell_width <= 0.0 || cell_height <= 0.0 {
                    return EventPropagation::Continue;
                }

                if let Event::PointerMove(pointer_event) = event {
                    // Only track selection when we're selecting (primary button was pressed)
                    if !is_selecting.get_untracked() {
                        return EventPropagation::Continue;
                    }

                    let pos = pointer_event.pos;
                    {
                        let x = pos.x;
                        let y = pos.y;

                        session.with_term_mut(|term| {
                            if term.selection.is_none() {
                                return;
                            }

                            let (cols, lines, display_offset) = {
                                let grid = term.grid();
                                (grid.columns(), grid.screen_lines(), grid.display_offset())
                            };

                            if cols == 0 || lines == 0 {
                                return;
                            }

                            let col = (x / cell_width).floor() as isize;
                            let line = (y / cell_height).floor() as isize;

                            if col < 0 || line < 0 {
                                return;
                            }

                            let mut col = col as usize;
                            let mut line = line as usize;

                            if col >= cols {
                                col = cols - 1;
                            }
                            if line >= lines {
                                line = lines - 1;
                            }

                            let viewport_point =
                                alacritty_terminal::index::Point::<usize, Column>::new(
                                    line,
                                    Column(col),
                                );
                            let term_point =
                                viewport_to_point(display_offset, viewport_point);

                            if let Some(selection) = term.selection.as_mut() {
                                selection.update(term_point, Side::Right);
                            }
                        });

                        canvas_id.request_paint();
                        return EventPropagation::Stop;
                    }
                }

                EventPropagation::Continue
            })
        })
        .on_event(EventListener::PointerUp, move |event| {
            logging::measure_ui_event("terminal pointer up", || {
                if let Event::PointerUp(_) = event {
                    is_selecting.set(false);
                }
                EventPropagation::Continue
            })
        })
        .context_menu(move || {
            let session_for_reset = session.clone();
            let error_msg_for_reset = error_msg;

            // Check if there's a selection to show copy option
            let has_selection = session.get_untracked().map_or(false, |sess| {
                sess.with_term(|term| term.selection_to_string().is_some())
            });

            let mut menu = Menu::new("");

            // Only show Copy if there's a selection
            if has_selection {
                let session_for_copy = session.get_untracked();
                menu = menu.entry(MenuItem::new("复制").action(move || {
                    if let Some(ref sess) = session_for_copy {
                        if let Some(text) = sess.with_term(|term| term.selection_to_string()) {
                            crate::services::set_clipboard_string(&text);
                            logging::log_line("INFO", "Terminal: copied selection to clipboard");
                        }
                    }
                }));
            }

            menu = menu.entry(MenuItem::new("粘贴").action(move || {
                if let Some(sess) = session.get_untracked() {
                    if let Some(text) = crate::services::get_clipboard_string() {
                        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                        if !normalized.is_empty() {
                            if let Err(err) = sess.write(normalized.as_bytes()) {
                                logging::log_line("ERROR", &format!("Terminal paste failed: {err}"));
                            }
                        }
                    }
                }
            }));

            // Split actions
            menu = menu
                .separator()
                .entry(MenuItem::new("向右分割").action(move || {
                    logging::log_line("INFO", &format!("Terminal: Split right from pane {pane_id}"));
                    let new_id = next_pane_id.get();
                    next_pane_id.set(new_id + 1);

                    let new_pane = TerminalPane {
                        id: new_id,
                        session: RwSignal::new(None),
                        trigger: floem::ext_event::ExtSendTrigger::new(),
                        flex_ratio: RwSignal::new(1.0),
                    };

                    terminal_panes.update(|panes| {
                        // Find current pane index and insert after it
                        if let Some(idx) = panes.iter().position(|p| p.id == pane_id) {
                            panes.insert(idx + 1, new_pane);
                        } else {
                            panes.push(new_pane);
                        }
                    });
                    // Collect triggers before spawning thread (RwSignal is not Send)
                    let triggers: Vec<_> = terminal_panes.get_untracked()
                        .iter()
                        .map(|p| (p.id, p.trigger.clone()))
                        .collect();
                    let pane_count = triggers.len();
                    // Delay trigger to allow layout to recalculate after pane list changes
                    std::thread::spawn(move || {
                        logging::log_line(
                            "DEBUG",
                            &format!("[Split Right] triggering {} panes after {}ms delay", pane_count, SPLIT_TRIGGER_DELAY_MS),
                        );
                        std::thread::sleep(std::time::Duration::from_millis(SPLIT_TRIGGER_DELAY_MS));
                        for (id, trigger) in triggers.iter() {
                            logging::log_line("DEBUG", &format!("[Split Right] trigger pane {}", id));
                            register_ext_trigger(trigger.clone());
                        }
                        // Second wave trigger to ensure layout is complete
                        logging::log_line("DEBUG", &format!("[Split Right] second wave after {}ms", SPLIT_SECOND_WAVE_MS));
                        std::thread::sleep(std::time::Duration::from_millis(SPLIT_SECOND_WAVE_MS));
                        for (id, trigger) in triggers.iter() {
                            logging::log_line("DEBUG", &format!("[Split Right] trigger pane {} (2nd)", id));
                            register_ext_trigger(trigger.clone());
                        }
                    });
                }))
                .entry(MenuItem::new("向左分割").action(move || {
                    logging::log_line("INFO", &format!("Terminal: Split left from pane {pane_id}"));
                    let new_id = next_pane_id.get();
                    next_pane_id.set(new_id + 1);

                    let new_pane = TerminalPane {
                        id: new_id,
                        session: RwSignal::new(None),
                        trigger: floem::ext_event::ExtSendTrigger::new(),
                        flex_ratio: RwSignal::new(1.0),
                    };

                    terminal_panes.update(|panes| {
                        // Find current pane index and insert before it
                        if let Some(idx) = panes.iter().position(|p| p.id == pane_id) {
                            panes.insert(idx, new_pane);
                        } else {
                            panes.insert(0, new_pane);
                        }
                    });
                    // Collect triggers before spawning thread (RwSignal is not Send)
                    let triggers: Vec<_> = terminal_panes.get_untracked()
                        .iter()
                        .map(|p| (p.id, p.trigger.clone()))
                        .collect();
                    let pane_count = triggers.len();
                    // Delay trigger to allow layout to recalculate after pane list changes
                    std::thread::spawn(move || {
                        logging::log_line("DEBUG", &format!(
                            "[Split Left] triggering {} panes after {}ms delay",
                            pane_count, SPLIT_TRIGGER_DELAY_MS
                        ));
                        std::thread::sleep(std::time::Duration::from_millis(SPLIT_TRIGGER_DELAY_MS));
                        for (id, trigger) in triggers.iter() {
                            logging::log_line("DEBUG", &format!("[Split Left] trigger pane {}", id));
                            register_ext_trigger(trigger.clone());
                        }
                        // Second wave trigger to ensure layout is complete
                        logging::log_line("DEBUG", &format!("[Split Left] second wave after {}ms", SPLIT_SECOND_WAVE_MS));
                        std::thread::sleep(std::time::Duration::from_millis(SPLIT_SECOND_WAVE_MS));
                        for (id, trigger) in triggers.iter() {
                            logging::log_line("DEBUG", &format!("[Split Left] trigger pane {} (2nd)", id));
                            register_ext_trigger(trigger.clone());
                        }
                    });
                }));

            menu = menu
                .separator()
                .entry(MenuItem::new("重置终端").action(move || {
                    logging::log_line("INFO", "Terminal: Reset requested");
                    session_for_reset.set(None);
                    error_msg_for_reset.set(None);
                }));

            menu
        });

    // Grid size overlay (centered, shows during resize)
    // Use dyn_container to conditionally show/hide overlay with proper event handling
    let grid_overlay = dyn_container(
        move || resize_overlay_visible.get(),
        move |visible| {
            if visible {
                let text = resize_overlay_text.get();
                logging::breadcrumb(format!("Pane {}: overlay visible with text '{}'", pane_id, text));
                // Overlay container: absolute + full size + centered content
                container(
                    container(label(move || text.clone()).style(move |s| {
                        s.font_size(18.0)
                            .font_bold()
                            .color(theme.text)
                            .padding(12.0)
                            .padding_horiz(20.0)
                    }))
                    .style(move |s| {
                        s.background(theme.panel_bg.with_alpha(0.95))
                            .border_radius(8.0)
                            .border(1.0)
                            .border_color(theme.accent)
                    })
                )
                .style(|s| {
                    s.absolute()
                        .inset(0.0)
                        .items_center()
                        .justify_center()
                        .pointer_events_none()  // Don't block input to terminal
                })
                .into_any()
            } else {
                // When not visible, return empty with no size to avoid blocking input
                empty().style(|s| s.display(floem::style::Display::None)).into_any()
            }
        },
    )
    .style(|s| s.z_index(100));

    // Return terminal wrapper with overlay on top
    stack((
        terminal_wrapper.style(|s| s.flex_grow(1.0).height_full()),
        grid_overlay,
    ))
    .style(|s| s.flex_grow(1.0).height_full())
}

#[cfg(not(target_os = "macos"))]
pub fn terminal_view(theme: UiTheme, workspace: WorkspaceTab) -> impl IntoView {
    let workspace_name = workspace.name;
    v_stack((
        label(|| "Terminal").style(move |s| {
            s.font_size(12.0)
                .font_bold()
                .color(theme.text_muted)
        }),
        meta_text(format!("Workspace: {}", workspace_name.get()), theme),
        container(label(||
            "Terminal is only available on macOS in this build."
        ))
        .style(move |s| {
            s.width_full()
                .height_full()
                .padding(12.0)
                .background(theme.panel_bg)
                .border(1.0)
                .border_color(theme.border_subtle)
                .color(theme.text_soft)
        }),
    ))
    .style(|s| s.width_full().height_full().row_gap(8.0))
}
