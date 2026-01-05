//! Terminal panel - multi-pane orchestration and splitter handling.

#[cfg(target_os = "macos")]
use floem::peniko::Color;

/// Drag state for terminal splitters: (pane_id_left, last_x in container coords)
/// We track last_x to calculate incremental deltas, avoiding coordinate system mismatches.
/// A value of -1.0 for last_x indicates the first move hasn't happened yet.
#[cfg(target_os = "macos")]
pub type SplitterDragState = Option<(usize, f64)>;

/// Sentinel value indicating first move hasn't happened yet after PointerDown.
#[cfg(target_os = "macos")]
pub const DRAG_STATE_SENTINEL: f64 = -1.0;

/// Minimum flex ratio for a pane (prevents panes from becoming too small).
/// A ratio of 0.05 allows panes as small as ~50px in a 1000px container.
#[cfg(target_os = "macos")]
pub const MIN_PANE_RATIO: f64 = 0.05;

/// Calculates new flex ratios after a splitter drag.
/// Returns (new_left_ratio, new_right_ratio) or None if the drag should be ignored.
#[cfg(target_os = "macos")]
pub fn calculate_splitter_drag(
    delta_x: f64,
    left_ratio: f64,
    right_ratio: f64,
    container_width: f64,
) -> Option<(f64, f64)> {
    // Skip tiny movements to avoid jitter
    if delta_x.abs() < 1.0 {
        return None;
    }

    let total_ratio = left_ratio + right_ratio;

    // Convert pixel delta to ratio delta
    // Formula: ratio_delta = delta_x * total_ratio / container_width
    // This makes 1px mouse movement = 1px splitter movement
    let ratio_delta = delta_x * total_ratio / container_width.max(100.0);

    // Apply minimum ratio constraints
    let new_left = (left_ratio + ratio_delta)
        .max(MIN_PANE_RATIO)
        .min(total_ratio - MIN_PANE_RATIO);
    let new_right = total_ratio - new_left;

    Some((new_left, new_right))
}

/// Returns the splitter background color based on drag state.
#[cfg(target_os = "macos")]
pub fn splitter_background_color(is_dragging: bool, accent: Color, border_subtle: Color) -> Color {
    if is_dragging {
        accent
    } else {
        border_subtle
    }
}

/// Returns the splitter hover background color.
#[cfg(target_os = "macos")]
pub fn splitter_hover_color(accent: Color) -> Color {
    accent.with_alpha(0.5)
}

/// Panel header style function.
#[cfg(target_os = "macos")]
pub fn panel_header_style() -> impl Fn(floem::style::Style) -> floem::style::Style {
    move |s| {
        s.font_size(12.0)
            .font_bold()
    }
}

/// Panel container style function.
#[cfg(target_os = "macos")]
pub fn panel_container_style(bg_color: Color, border_color: Color) -> impl Fn(floem::style::Style) -> floem::style::Style {
    move |s| {
        s.width_full()
            .flex_grow(1.0)
            .padding(8.0)
            .background(bg_color)
            .border(1.0)
            .border_color(border_color)
    }
}

/// Checks if a pane is the last in the list (no splitter needed after it).
#[cfg(target_os = "macos")]
pub fn is_last_pane<T>(pane_id: usize, panes: &[T], get_id: impl Fn(&T) -> usize) -> bool {
    panes.last().map_or(true, |last| get_id(last) == pane_id)
}

/// Finds the index of a pane by its ID.
#[cfg(target_os = "macos")]
pub fn find_pane_index<T>(pane_id: usize, panes: &[T], get_id: impl Fn(&T) -> usize) -> Option<usize> {
    panes.iter().position(|p| get_id(p) == pane_id)
}
