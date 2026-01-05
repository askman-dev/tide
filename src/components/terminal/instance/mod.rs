//! Terminal instance module - single terminal pane with renderer, input handling, and state.

mod state;
#[cfg(target_os = "macos")]
pub mod renderer;
#[cfg(target_os = "macos")]
pub mod input;

#[cfg(target_os = "macos")]
pub use state::TerminalInstanceState;
