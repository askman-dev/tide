//! Terminal instance renderer - canvas paint helpers and cell rendering.

#[cfg(target_os = "macos")]
use super::super::colors::{TerminalColorList, resolve_bg_color, resolve_fg_color};

#[cfg(target_os = "macos")]
use super::super::constants::{CELL_PADDING, TERMINAL_FONT_SIZE, terminal_font_families};

#[cfg(target_os = "macos")]
use alacritty_terminal::{
    term::{
        cell::Flags,
        color::Colors as TermColors,
    },
    vte::ansi::{Color as AnsiColor, NamedColor},
};

#[cfg(target_os = "macos")]
use floem::{
    peniko::Color,
    text::{Attrs, AttrsList, TextLayout},
};

/// Measures cell dimensions from font metrics.
/// Returns (cell_width, cell_height, y_offset).
#[cfg(target_os = "macos")]
pub fn measure_cell_size() -> (f64, f64, f64) {
    let font_families = terminal_font_families();
    let attrs = Attrs::new()
        .font_size(TERMINAL_FONT_SIZE)
        .family(&font_families);
    let base_attrs_list = AttrsList::new(attrs);
    let mut metrics_layout = TextLayout::new();
    metrics_layout.set_text("m", base_attrs_list, None);
    let metrics_size = metrics_layout.size();

    let cell_width = metrics_size.width.max(1.0);
    let cell_height = (metrics_size.height * 1.25).max(1.0);
    let y_offset = (cell_height - metrics_size.height) / 2.0;

    (cell_width, cell_height, y_offset)
}

/// Calculates terminal grid dimensions from canvas size and cell metrics.
/// Returns (cols, rows).
#[cfg(target_os = "macos")]
pub fn calculate_grid_size(canvas_width: f64, canvas_height: f64, cell_width: f64, cell_height: f64) -> (u16, u16) {
    // Subtract padding (CELL_PADDING on each side)
    let padding_total = CELL_PADDING * 2.0;
    let available_width = (canvas_width - padding_total).max(1.0);
    let available_height = (canvas_height - padding_total).max(1.0);

    let cols = (available_width / cell_width).floor().max(1.0) as u16;
    let rows = (available_height / cell_height).floor().max(1.0) as u16;

    (cols, rows)
}

/// Context for rendering a single cell.
#[cfg(target_os = "macos")]
pub struct CellRenderContext<'a> {
    pub cell_width: f64,
    pub cell_height: f64,
    pub y_offset: f64,
    pub palette: &'a TerminalColorList,
    pub term_colors: &'a TermColors,
    pub default_bg: Color,
    pub has_selection: bool,
}

/// Calculates the position for a cell given grid coordinates.
#[cfg(target_os = "macos")]
pub fn cell_position(col: f64, row: f64, cell_width: f64, cell_height: f64) -> (f64, f64) {
    let x = CELL_PADDING + col * cell_width;
    let y = CELL_PADDING + row * cell_height;
    (x, y)
}

/// Checks if a cell position is within the visible canvas area.
#[cfg(target_os = "macos")]
pub fn is_cell_visible(x: f64, y: f64, cell_width: f64, cell_height: f64, canvas_width: f64, canvas_height: f64) -> bool {
    // Use small tolerance (1.0) to avoid floating point precision issues
    let tolerance = CELL_PADDING - 1.0;
    x + cell_width <= canvas_width - tolerance && y + cell_height <= canvas_height - tolerance
}

/// Resolves foreground and background colors for a cell.
#[cfg(target_os = "macos")]
pub fn resolve_cell_colors(
    fg_color: &AnsiColor,
    bg_color: &AnsiColor,
    flags: Flags,
    is_selected: bool,
    is_inverted: bool,
    palette: &TerminalColorList,
    term_colors: &TermColors,
    default_bg: Color,
) -> (Color, Color) {
    let mut fg = resolve_fg_color(term_colors, palette, fg_color, flags);
    let mut bg = resolve_bg_color(term_colors, palette, bg_color);

    // Handle inverse video
    if is_inverted {
        std::mem::swap(&mut fg, &mut bg);
    }

    // Handle selection highlighting
    if is_selected {
        fg = Color::from_rgb8(40, 42, 54);
        bg = Color::from_rgb8(248, 248, 242);
    }

    // Skip background if it matches default (optimization)
    if !is_selected && !is_inverted && matches!(bg_color, AnsiColor::Named(NamedColor::Background)) {
        bg = default_bg;
    }

    (fg, bg)
}

/// Creates the resize overlay view showing grid dimensions.
#[cfg(target_os = "macos")]
pub fn create_grid_overlay_style() -> impl Fn(floem::style::Style) -> floem::style::Style {
    move |s| {
        s.position(floem::style::Position::Absolute)
            .inset_top(8.0)
            .inset_right(8.0)
            .padding(8.0)
            .background(Color::from_rgba8(0, 0, 0, 200))
            .color(Color::WHITE)
            .font_size(12.0)
            .border_radius(8.0)
    }
}

/// Debug border color based on pane ID (for visualizing pane boundaries).
#[cfg(target_os = "macos")]
pub fn debug_border_color(pane_id: usize) -> Color {
    match pane_id % 4 {
        0 => Color::from_rgba8(255, 0, 0, 200),   // Red
        1 => Color::from_rgba8(0, 255, 0, 200),   // Green
        2 => Color::from_rgba8(0, 0, 255, 200),   // Blue
        _ => Color::from_rgba8(255, 255, 0, 200), // Yellow
    }
}
