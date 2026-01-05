//! Terminal instance input handling - keyboard, mouse, scroll, and IME.

#[cfg(target_os = "macos")]
use super::super::constants::CELL_PADDING;

#[cfg(target_os = "macos")]
use alacritty_terminal::index::{Column, Point};

#[cfg(target_os = "macos")]
use floem::keyboard::{Key, NamedKey};

/// Converts a pointer position to terminal grid coordinates.
/// Returns None if the position is in the padding area.
#[cfg(target_os = "macos")]
pub fn pointer_to_grid_point(
    x: f64,
    y: f64,
    cell_width: f64,
    cell_height: f64,
    cols: usize,
    rows: usize,
) -> Option<Point<usize>> {
    // Check if in padding area
    if x < CELL_PADDING || y < CELL_PADDING {
        return None;
    }

    let adjusted_x = x - CELL_PADDING;
    let adjusted_y = y - CELL_PADDING;

    let col = (adjusted_x / cell_width).floor() as usize;
    let row = (adjusted_y / cell_height).floor() as usize;

    // Clamp to grid bounds
    if col >= cols || row >= rows {
        return None;
    }

    Some(Point::new(row, Column(col)))
}

/// Converts a floem key to an ANSI escape sequence for terminal input.
/// Returns the bytes to send to the PTY.
#[cfg(target_os = "macos")]
pub fn key_to_pty_bytes(key: &Key, modifiers: &floem::keyboard::Modifiers) -> Option<Vec<u8>> {
    let ctrl = modifiers.control();
    let alt = modifiers.alt();
    let shift = modifiers.shift();

    match key {
        Key::Named(named) => named_key_to_bytes(named, ctrl, alt, shift),
        Key::Character(c) => char_to_pty_bytes(c, ctrl, alt),
        _ => None,
    }
}

/// Converts a named key to PTY bytes.
#[cfg(target_os = "macos")]
fn named_key_to_bytes(key: &NamedKey, ctrl: bool, alt: bool, _shift: bool) -> Option<Vec<u8>> {
    let bytes = match key {
        NamedKey::Enter => vec![b'\r'],
        NamedKey::Backspace => vec![0x7f],
        NamedKey::Tab => vec![b'\t'],
        NamedKey::Escape => vec![0x1b],
        NamedKey::Space => vec![b' '],

        // Arrow keys with modifier support
        NamedKey::ArrowUp => {
            if alt {
                b"\x1b\x1b[A".to_vec()
            } else if ctrl {
                b"\x1b[1;5A".to_vec()
            } else {
                b"\x1b[A".to_vec()
            }
        }
        NamedKey::ArrowDown => {
            if alt {
                b"\x1b\x1b[B".to_vec()
            } else if ctrl {
                b"\x1b[1;5B".to_vec()
            } else {
                b"\x1b[B".to_vec()
            }
        }
        NamedKey::ArrowRight => {
            if alt {
                b"\x1b\x1bf".to_vec() // word forward
            } else if ctrl {
                b"\x1b[1;5C".to_vec()
            } else {
                b"\x1b[C".to_vec()
            }
        }
        NamedKey::ArrowLeft => {
            if alt {
                b"\x1b\x1bb".to_vec() // word backward
            } else if ctrl {
                b"\x1b[1;5D".to_vec()
            } else {
                b"\x1b[D".to_vec()
            }
        }

        // Home/End with modifier support
        NamedKey::Home => {
            if ctrl {
                b"\x1b[1;5H".to_vec()
            } else {
                b"\x1b[H".to_vec()
            }
        }
        NamedKey::End => {
            if ctrl {
                b"\x1b[1;5F".to_vec()
            } else {
                b"\x1b[F".to_vec()
            }
        }

        // Page Up/Down
        NamedKey::PageUp => b"\x1b[5~".to_vec(),
        NamedKey::PageDown => b"\x1b[6~".to_vec(),

        // Insert/Delete
        NamedKey::Insert => b"\x1b[2~".to_vec(),
        NamedKey::Delete => b"\x1b[3~".to_vec(),

        // Function keys
        NamedKey::F1 => b"\x1bOP".to_vec(),
        NamedKey::F2 => b"\x1bOQ".to_vec(),
        NamedKey::F3 => b"\x1bOR".to_vec(),
        NamedKey::F4 => b"\x1bOS".to_vec(),
        NamedKey::F5 => b"\x1b[15~".to_vec(),
        NamedKey::F6 => b"\x1b[17~".to_vec(),
        NamedKey::F7 => b"\x1b[18~".to_vec(),
        NamedKey::F8 => b"\x1b[19~".to_vec(),
        NamedKey::F9 => b"\x1b[20~".to_vec(),
        NamedKey::F10 => b"\x1b[21~".to_vec(),
        NamedKey::F11 => b"\x1b[23~".to_vec(),
        NamedKey::F12 => b"\x1b[24~".to_vec(),

        _ => return None,
    };

    Some(bytes)
}

/// Converts a character key press to PTY bytes.
#[cfg(target_os = "macos")]
fn char_to_pty_bytes(c: &str, ctrl: bool, alt: bool) -> Option<Vec<u8>> {
    let ch = c.chars().next()?;

    if ctrl {
        // Control key combinations (e.g., Ctrl+C = 0x03)
        if ch.is_ascii_alphabetic() {
            let ctrl_char = (ch.to_ascii_lowercase() as u8) - b'a' + 1;
            return Some(vec![ctrl_char]);
        }
        // Special control characters
        match ch {
            '[' | '3' => return Some(vec![0x1b]), // Ctrl+[ = Escape
            '\\' | '4' => return Some(vec![0x1c]), // Ctrl+\
            ']' | '5' => return Some(vec![0x1d]), // Ctrl+]
            '^' | '6' => return Some(vec![0x1e]), // Ctrl+^
            '_' | '7' => return Some(vec![0x1f]), // Ctrl+_
            '?' | '8' => return Some(vec![0x7f]), // Ctrl+? = DEL
            '@' | '2' => return Some(vec![0x00]), // Ctrl+@ = NUL
            _ => {}
        }
    }

    if alt {
        // Alt key sends ESC prefix
        let mut bytes = vec![0x1b];
        bytes.extend(c.as_bytes());
        return Some(bytes);
    }

    // Regular character input
    Some(c.as_bytes().to_vec())
}

/// Calculates scroll lines from pixel delta and accumulator.
/// Returns (lines_to_scroll, new_accumulator).
#[cfg(target_os = "macos")]
pub fn calculate_scroll_lines(
    delta_y: f64,
    cell_height: f64,
    current_accumulator: f64,
) -> (i32, f64) {
    // Add delta to accumulator
    let new_acc = current_accumulator + delta_y;

    // Calculate how many full lines to scroll
    let lines = (new_acc / cell_height).trunc() as i32;

    // Update accumulator with remainder
    let remaining_acc = new_acc - (lines as f64 * cell_height);

    (lines, remaining_acc)
}

/// Determines if a key event should be handled by the terminal or passed through.
#[cfg(target_os = "macos")]
pub fn should_handle_key(key: &Key, modifiers: &floem::keyboard::Modifiers) -> bool {
    // Don't handle if Cmd/Meta is pressed (macOS shortcuts)
    if modifiers.meta() {
        return false;
    }

    match key {
        Key::Named(named) => match named {
            // Always handle navigation and editing keys
            NamedKey::Enter
            | NamedKey::Backspace
            | NamedKey::Tab
            | NamedKey::Escape
            | NamedKey::Space
            | NamedKey::ArrowUp
            | NamedKey::ArrowDown
            | NamedKey::ArrowLeft
            | NamedKey::ArrowRight
            | NamedKey::Home
            | NamedKey::End
            | NamedKey::PageUp
            | NamedKey::PageDown
            | NamedKey::Insert
            | NamedKey::Delete
            | NamedKey::F1
            | NamedKey::F2
            | NamedKey::F3
            | NamedKey::F4
            | NamedKey::F5
            | NamedKey::F6
            | NamedKey::F7
            | NamedKey::F8
            | NamedKey::F9
            | NamedKey::F10
            | NamedKey::F11
            | NamedKey::F12 => true,
            _ => false,
        },
        Key::Character(_) => true,
        _ => false,
    }
}
