//! Terminal instance state - bundles all reactive signals for a single terminal pane.

#[cfg(target_os = "macos")]
use floem::{
    ext_event::ExtSendTrigger,
    reactive::RwSignal,
};

#[cfg(target_os = "macos")]
use std::time::Instant;

/// Bundles all reactive signals for a single terminal instance.
/// This consolidates the state that was previously scattered across terminal_pane_view.
#[cfg(target_os = "macos")]
pub struct TerminalInstanceState {
    /// Error message to display (e.g., session spawn failure)
    pub error_msg: RwSignal<Option<String>>,
    /// Last confirmed grid size (cols, rows)
    pub last_size: RwSignal<(u16, u16)>,
    /// Pending grid size waiting to be applied
    pub pending_size: RwSignal<(u16, u16)>,
    /// Timestamp of last resize request (for debouncing)
    pub last_resize_request: RwSignal<Instant>,
    /// Trigger for canvas repaint from external resize events
    pub resize_trigger: ExtSendTrigger,
    /// Cell dimensions in pixels (width, height)
    pub cell_size: RwSignal<(f64, f64)>,
    /// Y offset for text baseline within cell
    pub cell_y_offset: RwSignal<f64>,
    /// Timestamp of last PTY resize (for debouncing)
    pub last_pty_resize_at: RwSignal<Instant>,
    /// Whether IME is currently active
    pub ime_focused: RwSignal<bool>,
    /// Tick counter for forcing IME cursor updates
    pub ime_update_tick: RwSignal<u64>,
    /// Last IME cursor area (position, size) for candidate window placement
    pub last_ime_cursor_area: RwSignal<Option<(floem::kurbo::Point, floem::kurbo::Size)>>,
    /// Last canvas size (width, height) for change detection
    pub last_canvas_size: RwSignal<(f64, f64)>,
    /// Scroll accumulator for smooth touchpad scrolling (sub-line deltas)
    pub scroll_accumulator: RwSignal<f64>,
    /// Whether resize overlay is visible
    pub resize_overlay_visible: RwSignal<bool>,
    /// Text to display in resize overlay (e.g., "80x24")
    pub resize_overlay_text: RwSignal<String>,
    /// Timestamp when overlay was last shown (prevents premature hiding)
    pub overlay_show_time: RwSignal<Instant>,
    /// Trigger for hiding overlay after delay
    pub overlay_hide_trigger: ExtSendTrigger,
}

#[cfg(target_os = "macos")]
impl TerminalInstanceState {
    /// Creates a new terminal instance state with default signal values.
    pub fn new() -> Self {
        Self {
            error_msg: RwSignal::new(None),
            last_size: RwSignal::new((0, 0)),
            pending_size: RwSignal::new((0, 0)),
            last_resize_request: RwSignal::new(Instant::now()),
            resize_trigger: ExtSendTrigger::new(),
            cell_size: RwSignal::new((0.0, 0.0)),
            cell_y_offset: RwSignal::new(0.0),
            last_pty_resize_at: RwSignal::new(Instant::now()),
            ime_focused: RwSignal::new(false),
            ime_update_tick: RwSignal::new(0),
            last_ime_cursor_area: RwSignal::new(None),
            last_canvas_size: RwSignal::new((0.0, 0.0)),
            scroll_accumulator: RwSignal::new(0.0),
            resize_overlay_visible: RwSignal::new(false),
            resize_overlay_text: RwSignal::new(String::new()),
            overlay_show_time: RwSignal::new(Instant::now()),
            overlay_hide_trigger: ExtSendTrigger::new(),
        }
    }
}

#[cfg(target_os = "macos")]
impl Default for TerminalInstanceState {
    fn default() -> Self {
        Self::new()
    }
}
