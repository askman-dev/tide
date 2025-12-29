use crate::components::atoms::meta_text;
use crate::model::WorkspaceTab;
use crate::services::TerminalSession;
use crate::theme::{TerminalPalette, UiTheme};
use floem::prelude::*;

#[cfg(target_os = "macos")]
use alacritty_terminal::{
    grid::{Dimensions, Indexed},
    index::{Column, Side},
    selection::{Selection, SelectionType},
    term::{
        cell::Flags,
        color::Colors as TermColors,
        point_to_viewport,
        viewport_to_point,
        RenderableContent,
    },
    vte::ansi::{Color as AnsiColor, CursorShape as AnsiCursorShape, NamedColor},
};

#[cfg(target_os = "macos")]
use floem::{
    event::{Event, EventListener, EventPropagation},
    peniko::{
        kurbo::Rect,
        Brush, Color,
    },
    reactive::{RwSignal, Effect},
    text::{Attrs, AttrsList, FamilyOwned, TextLayout},
};

#[cfg(target_os = "macos")]
use floem::ui_events::pointer::{PointerButton, PointerEvent as UiPointerEvent};

#[cfg(target_os = "macos")]
use std::sync::Arc;

#[cfg(target_os = "macos")]
use std::ops::{Index, IndexMut};

/// Base font size for terminal cells.
#[cfg(target_os = "macos")]
const TERMINAL_FONT_SIZE: f32 = 12.0;

#[cfg(target_os = "macos")]
fn terminal_font_families() -> [FamilyOwned; 4] {
    [
        FamilyOwned::Name("SF Mono".into()),
        FamilyOwned::Name("Menlo".into()),
        FamilyOwned::Name("Monaco".into()),
        FamilyOwned::Monospace,
    ]
}

#[cfg(target_os = "macos")]
fn background_brush(theme: UiTheme) -> Brush {
    Brush::from(theme.panel_bg)
}

#[cfg(target_os = "macos")]
fn cursor_brush(theme: UiTheme) -> Brush {
    Brush::from(theme.accent.with_alpha(0.7))
}

#[cfg(target_os = "macos")]
fn ansi_rgb_to_color(rgb: alacritty_terminal::vte::ansi::Rgb) -> Color {
    Color::from_rgb8(rgb.r, rgb.g, rgb.b)
}

#[cfg(target_os = "macos")]
struct TerminalColorList([alacritty_terminal::vte::ansi::Rgb; alacritty_terminal::term::color::COUNT]);

#[cfg(target_os = "macos")]
impl TerminalColorList {
    fn from_palette(palette: &TerminalPalette) -> Self {
        use alacritty_terminal::term::color::COUNT;

        let mut list = TerminalColorList([alacritty_terminal::vte::ansi::Rgb::default(); COUNT]);

        list.fill_named(palette);
        list.fill_cube();
        list.fill_gray_ramp();

        list
    }

    fn fill_named(&mut self, palette: &TerminalPalette) {
        // Normal ANSI colors.
        self[NamedColor::Black] = palette.normal[0];
        self[NamedColor::Red] = palette.normal[1];
        self[NamedColor::Green] = palette.normal[2];
        self[NamedColor::Yellow] = palette.normal[3];
        self[NamedColor::Blue] = palette.normal[4];
        self[NamedColor::Magenta] = palette.normal[5];
        self[NamedColor::Cyan] = palette.normal[6];
        self[NamedColor::White] = palette.normal[7];

        // Bright ANSI colors.
        self[NamedColor::BrightBlack] = palette.bright[0];
        self[NamedColor::BrightRed] = palette.bright[1];
        self[NamedColor::BrightGreen] = palette.bright[2];
        self[NamedColor::BrightYellow] = palette.bright[3];
        self[NamedColor::BrightBlue] = palette.bright[4];
        self[NamedColor::BrightMagenta] = palette.bright[5];
        self[NamedColor::BrightCyan] = palette.bright[6];
        self[NamedColor::BrightWhite] = palette.bright[7];

        // Foreground and background.
        self[NamedColor::Foreground] = palette.primary_foreground;
        self[NamedColor::Background] = palette.primary_background;
        self[NamedColor::BrightForeground] = palette.bright[7];

        // Dimmed foreground and ANSI colors.
        let dim_fg = {
            let fg = palette.primary_foreground;
            let r = (fg.r as f32 * 0.66) as u8;
            let g = (fg.g as f32 * 0.66) as u8;
            let b = (fg.b as f32 * 0.66) as u8;
            alacritty_terminal::vte::ansi::Rgb { r, g, b }
        };

        self[NamedColor::DimForeground] = dim_fg;

        self[NamedColor::DimBlack] = palette.dim[0];
        self[NamedColor::DimRed] = palette.dim[1];
        self[NamedColor::DimGreen] = palette.dim[2];
        self[NamedColor::DimYellow] = palette.dim[3];
        self[NamedColor::DimBlue] = palette.dim[4];
        self[NamedColor::DimMagenta] = palette.dim[5];
        self[NamedColor::DimCyan] = palette.dim[6];
        self[NamedColor::DimWhite] = palette.dim[7];
    }

    fn fill_cube(&mut self) {
        let mut index: usize = 16;

        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    let red = if r == 0 { 0 } else { r * 40 + 55 };
                    let green = if g == 0 { 0 } else { g * 40 + 55 };
                    let blue = if b == 0 { 0 } else { b * 40 + 55 };

                    self.0[index] = alacritty_terminal::vte::ansi::Rgb {
                        r: red,
                        g: green,
                        b: blue,
                    };

                    index += 1;
                }
            }
        }

        debug_assert!(index == 232);
    }

    fn fill_gray_ramp(&mut self) {
        let mut index: usize = 232;

        for i in 0..24 {
            let value = i * 10 + 8;
            self.0[index] = alacritty_terminal::vte::ansi::Rgb {
                r: value,
                g: value,
                b: value,
            };

            index += 1;
        }

        debug_assert!(index == 256);
    }

    fn color_for_index(&self, index: usize, overrides: &TermColors) -> alacritty_terminal::vte::ansi::Rgb {
        let clamped = index.min(self.0.len().saturating_sub(1));
        overrides[clamped].unwrap_or(self.0[clamped])
    }
}

#[cfg(target_os = "macos")]
impl Index<usize> for TerminalColorList {
    type Output = alacritty_terminal::vte::ansi::Rgb;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

#[cfg(target_os = "macos")]
impl IndexMut<usize> for TerminalColorList {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

#[cfg(target_os = "macos")]
impl Index<NamedColor> for TerminalColorList {
    type Output = alacritty_terminal::vte::ansi::Rgb;

    fn index(&self, idx: NamedColor) -> &Self::Output {
        &self.0[idx as usize]
    }
}

#[cfg(target_os = "macos")]
impl IndexMut<NamedColor> for TerminalColorList {
    fn index_mut(&mut self, idx: NamedColor) -> &mut Self::Output {
        &mut self.0[idx as usize]
    }
}

#[cfg(target_os = "macos")]
fn resolve_fg_color(
    overrides: &TermColors,
    palette: &TerminalColorList,
    color: &AnsiColor,
    flags: Flags,
) -> Color {
    const DRAW_BOLD_TEXT_WITH_BRIGHT_COLORS: bool = true;
    const DIM_FACTOR: f32 = 0.66;

    let rgb = match *color {
        AnsiColor::Spec(spec) => {
            if (flags & Flags::DIM) == Flags::DIM {
                let r = (spec.r as f32 * DIM_FACTOR) as u8;
                let g = (spec.g as f32 * DIM_FACTOR) as u8;
                let b = (spec.b as f32 * DIM_FACTOR) as u8;
                alacritty_terminal::vte::ansi::Rgb { r, g, b }
            } else {
                spec
            }
        }
        AnsiColor::Named(ansi) => {
            let dim_bold = flags & Flags::DIM_BOLD;

            if dim_bold == Flags::DIM_BOLD && ansi == NamedColor::Foreground {
                palette.color_for_index(NamedColor::DimForeground as usize, overrides)
            } else if DRAW_BOLD_TEXT_WITH_BRIGHT_COLORS && dim_bold == Flags::BOLD {
                palette.color_for_index(ansi.to_bright() as usize, overrides)
            } else if dim_bold == Flags::DIM
                || (!DRAW_BOLD_TEXT_WITH_BRIGHT_COLORS && dim_bold == Flags::DIM_BOLD)
            {
                palette.color_for_index(ansi.to_dim() as usize, overrides)
            } else {
                palette.color_for_index(ansi as usize, overrides)
            }
        }
        AnsiColor::Indexed(idx) => {
            let dim_bold = flags & Flags::DIM_BOLD;
            let mut palette_index = idx as usize;

            match (DRAW_BOLD_TEXT_WITH_BRIGHT_COLORS, dim_bold, idx) {
                (true, Flags::BOLD, 0..=7) => {
                    palette_index = idx as usize + 8;
                }
                (false, Flags::DIM, 8..=15) => {
                    palette_index = idx as usize - 8;
                }
                (false, Flags::DIM, 0..=7) => {
                    palette_index = NamedColor::DimBlack as usize + idx as usize;
                }
                _ => {}
            }

            palette.color_for_index(palette_index, overrides)
        }
    };

    ansi_rgb_to_color(rgb)
}

#[cfg(target_os = "macos")]
fn resolve_bg_color(
    overrides: &TermColors,
    palette: &TerminalColorList,
    color: &AnsiColor,
) -> Color {
    let rgb = match *color {
        AnsiColor::Spec(spec) => spec,
        AnsiColor::Named(ansi) => palette.color_for_index(ansi as usize, overrides),
        AnsiColor::Indexed(idx) => palette.color_for_index(idx as usize, overrides),
    };

    ansi_rgb_to_color(rgb)
}

/// Platform-gated terminal view entry point.
///
/// On macOS this hosts the real PTY-backed terminal backed by
/// `alacritty_terminal` and `portable-pty`.
/// On non-macOS platforms it shows a simple placeholder message.
#[cfg(target_os = "macos")]
pub fn terminal_view(theme: UiTheme, workspace: WorkspaceTab) -> impl IntoView {
    let workspace_name = workspace.name.clone();
    let workspace_root = workspace.root.clone();

    let session: RwSignal<Option<Arc<TerminalSession>>> = RwSignal::new(None);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let last_size: RwSignal<(u16, u16)> = RwSignal::new((0, 0));
    let cell_size: RwSignal<(f64, f64)> = RwSignal::new((0.0, 0.0));

    let terminal_canvas = canvas({
        let workspace_root = workspace_root.clone();
        move |cx, size| {
            let font_families = terminal_font_families();

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
                match TerminalSession::new(&workspace_root) {
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

            // 4. Measure cell size
            let attrs = Attrs::new()
                .font_size(TERMINAL_FONT_SIZE)
                .family(&font_families);
            let base_attrs_list = AttrsList::new(attrs);
            let mut metrics_layout = TextLayout::new();
            metrics_layout.set_text("W", base_attrs_list, None);
            let metrics_size = metrics_layout.size();
            let cell_width = metrics_size.width.max(1.0);
            let cell_height = metrics_size.height.max(1.0);

            cell_size.set((cell_width, cell_height));

            // 5. Resize terminal if needed
            let cols = (size.width / cell_width).floor().max(1.0) as u16;
            let rows = (size.height / cell_height).floor().max(1.0) as u16;

            {
                let (prev_cols, prev_rows) = last_size.get_untracked();
                if cols != prev_cols || rows != prev_rows {
                    if let Err(err) = session.resize(cols, rows) {
                        crate::logging::log_line(
                            "ERROR",
                            &format!("Terminal resize failed: {err}"),
                        );
                    } else {
                        last_size.set((cols, rows));
                    }
                }
            }

            // 6. Render content
            let palette = TerminalPalette::for_theme(theme);
            let palette_list = TerminalColorList::from_palette(&palette);

            session.with_term(|term| {
                let mut content: RenderableContent<'_> = term.renderable_content();
                let selection = content.selection;
                let cursor = content.cursor;
                let term_colors = content.colors;

                for indexed in content.display_iter.by_ref() {
                    let viewport_point =
                        match point_to_viewport(content.display_offset, indexed.point) {
                            Some(p) => p,
                            None => continue,
                        };

                    let col = viewport_point.column.0 as f64;
                    let row = viewport_point.line as f64;
                    let x = col * cell_width;
                    let y = row * cell_height;

                    if x >= size.width || y >= size.height {
                        continue;
                    }

                    let cell = indexed.cell;
                    let flags = cell.flags;

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

                    if is_selected {
                        std::mem::swap(&mut fg_color, &mut bg_color);
                    }

                    let bg_brush = Brush::from(bg_color);
                    let cell_rect = Rect::new(x, y, x + cell_width, y + cell_height);
                    cx.fill(&cell_rect, &bg_brush, 0.0);

                    let mut text = String::new();
                    text.push(cell.c);
                    if let Some(extra) = cell.zerowidth() {
                        for ch in extra {
                            text.push(*ch);
                        }
                    }

                    if text.trim().is_empty() {
                        continue;
                    }

                    let attrs = Attrs::new()
                        .color(fg_color)
                        .font_size(TERMINAL_FONT_SIZE)
                        .family(&font_families);
                    let attrs_list = AttrsList::new(attrs);

                    let mut cell_layout = TextLayout::new();
                    cell_layout.set_text(&text, attrs_list, None);
                    cx.draw_text(&cell_layout, floem::kurbo::Point::new(x, y));
                }

                // Cursor
                if cursor.shape != AnsiCursorShape::Hidden && session.is_active() {
                    if let Some(viewport_cursor) =
                        point_to_viewport(content.display_offset, cursor.point)
                    {
                        let col = viewport_cursor.column.0 as f64;
                        let row = viewport_cursor.line as f64;
                        let x = col * cell_width;
                        let y = row * cell_height;
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
        }
    });

    let canvas_id = terminal_canvas.id();

    // Effect to trigger repaint when session/error state changes
    Effect::new(move |_| {
        session.track();
        error_msg.track();
        canvas_id.request_paint();
    });

    let terminal_canvas = terminal_canvas
        .style(move |s| {
            s.width_full()
                .height_full()
                .background(theme.panel_bg)
                .focusable(true)
        })
        .on_event(EventListener::KeyDown, move |event| {
            if let Event::Key(key_event) = event {
                if key_event.state != KeyState::Down {
                    return EventPropagation::Continue;
                }

                // Check for restart if session is inactive
                if let Some(session_arc) = session.get_untracked() {
                    if !session_arc.is_active() {
                        if key_event.key == Key::Named(NamedKey::Enter) {
                            session.set(None);
                            error_msg.set(None);
                            return EventPropagation::Stop;
                        }
                        return EventPropagation::Stop; // Consume keys when dead
                    }
                }

                // Check for restart if we have an error (e.g. failed to start)
                if error_msg.get_untracked().is_some() {
                     if key_event.key == Key::Named(NamedKey::Enter) {
                        session.set(None);
                        error_msg.set(None);
                        return EventPropagation::Stop;
                    }
                }

                let Some(session) = session.get_untracked() else {
                    return EventPropagation::Continue;
                };

                let key = &key_event.key;
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
        .on_event(EventListener::PointerWheel, move |event| {
            let Some(session) = session.get_untracked() else {
                return EventPropagation::Continue;
            };
            if !session.is_active() { return EventPropagation::Continue; }

            let (_, cell_height) = cell_size.get_untracked();
            if cell_height <= 0.0 {
                return EventPropagation::Continue;
            }

            if let Some(delta) = event.pixel_scroll_delta_vec2() {
                let dy = delta.y;
                let lines = (dy / cell_height).round() as i32;
                if lines != 0 {
                    session.scroll_display(lines);
                    return EventPropagation::Stop;
                }
            }

            EventPropagation::Continue
        })
        .on_event(EventListener::PointerDown, move |event| {
            let Some(session) = session.get_untracked() else {
                return EventPropagation::Continue;
            };

            let (cell_width, cell_height) = cell_size.get_untracked();
            if cell_width <= 0.0 || cell_height <= 0.0 {
                return EventPropagation::Continue;
            }

            if let Event::Pointer(UiPointerEvent::Down(button_event)) = event {
                if button_event.button != Some(PointerButton::Primary) {
                    return EventPropagation::Continue;
                }

                if let Some(pos) = event.point() {
                    canvas_id.request_focus();
                    canvas_id.request_active();

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

                    return EventPropagation::Stop;
                }
            }

            EventPropagation::Continue
        })
        .on_event(EventListener::PointerMove, move |event| {
            let Some(session) = session.get_untracked() else {
                return EventPropagation::Continue;
            };
            // Allow selection even if inactive? Yes, why not.

            let (cell_width, cell_height) = cell_size.get_untracked();
            if cell_width <= 0.0 || cell_height <= 0.0 {
                return EventPropagation::Continue;
            }

            if let Event::Pointer(UiPointerEvent::Move(update)) = event {
                if !update.current.buttons.contains(PointerButton::Primary) {
                    return EventPropagation::Continue;
                }

                if let Some(pos) = event.point() {
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

                    return EventPropagation::Stop;
                }
            }

            EventPropagation::Continue
        })
        .on_event(EventListener::PointerUp, move |event| {
            if let Event::Pointer(UiPointerEvent::Up(_)) = event {
                canvas_id.clear_active();
            }
            EventPropagation::Continue
        });

    v_stack((
        Label::new("Terminal").style(move |s| {
            s.font_size(12.0)
                .font_bold()
                .color(theme.text_muted)
        }),
        meta_text(format!("Workspace: {workspace_name}"), theme),
        Container::new(terminal_canvas).style(move |s| {
            s.width_full()
                .height_full()
                .padding(8.0)
                .background(theme.panel_bg)
                .border(1.0)
                .border_color(theme.border_subtle)
        }),
    ))
    .style(|s| s.width_full().height_full().row_gap(8.0))
}

#[cfg(not(target_os = "macos"))]
pub fn terminal_view(theme: UiTheme, workspace: WorkspaceTab) -> impl IntoView {
    let workspace_name = workspace.name.clone();
    v_stack((
        Label::new("Terminal").style(move |s| {
            s.font_size(12.0)
                .font_bold()
                .color(theme.text_muted)
        }),
        meta_text(format!("Workspace: {workspace_name}"), theme),
        Container::new(Label::new(
            "Terminal is only available on macOS in this build.",
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
