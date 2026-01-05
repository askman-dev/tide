//! Terminal component constants.

#[cfg(target_os = "macos")]
use floem::text::FamilyOwned;

/// Base font size for terminal cells.
#[cfg(target_os = "macos")]
pub const TERMINAL_FONT_SIZE: f32 = 13.0;

/// Width of the splitter handle between terminal panes.
#[cfg(target_os = "macos")]
pub const SPLITTER_WIDTH: f64 = 6.0;

/// Padding around terminal content (each side).
#[cfg(target_os = "macos")]
pub const CELL_PADDING: f64 = 8.0;

/// Debounce delay for PTY resize operations (ms).
#[cfg(target_os = "macos")]
pub const PTY_RESIZE_DEBOUNCE_MS: u64 = 50;

/// Duration to show resize overlay (ms).
#[cfg(target_os = "macos")]
pub const OVERLAY_SHOW_DURATION_MS: u64 = 1000;

/// Minimum visible time before hiding overlay (ms).
#[cfg(target_os = "macos")]
pub const OVERLAY_MIN_VISIBLE_MS: u64 = 900;

/// Delay before triggering pane resize after split (ms).
#[cfg(target_os = "macos")]
pub const SPLIT_TRIGGER_DELAY_MS: u64 = 100;

/// Second wave trigger delay after split (ms).
#[cfg(target_os = "macos")]
pub const SPLIT_SECOND_WAVE_MS: u64 = 150;

/// Terminal font families in preference order.
#[cfg(target_os = "macos")]
pub fn terminal_font_families() -> [FamilyOwned; 1] {
    [FamilyOwned::Name("Menlo".into())]
}
