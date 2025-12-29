use crate::logging;

/// Set the system clipboard contents to the given text.
///
/// On non-macOS platforms this is currently a no-op, but the function
/// still exists so callers do not need to be platform-gated.
pub fn set_clipboard_string(text: &str) {
    #[cfg(target_os = "macos")]
    {
        use arboard::Clipboard;

        let result = (|| -> Result<(), arboard::Error> {
            let mut clipboard = Clipboard::new()?;
            clipboard.set_text(text.to_owned())
        })();

        if let Err(err) = result {
            logging::log_line(
                "ERROR",
                &format!("Failed to set clipboard contents: {err}"),
            );
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = text;
        logging::log_line(
            "WARN",
            "set_clipboard_string called on unsupported platform",
        );
    }
}

/// Get the current system clipboard contents as UTF-8 text.
///
/// Returns `None` if the clipboard is empty or cannot be read.
pub fn get_clipboard_string() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use arboard::Clipboard;

        let result = (|| -> Result<String, arboard::Error> {
            let mut clipboard = Clipboard::new()?;
            clipboard.get_text()
        })();

        match result {
            Ok(text) => Some(text),
            Err(err) => {
                logging::log_line(
                    "ERROR",
                    &format!("Failed to get clipboard contents: {err}"),
                );
                None
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        logging::log_line(
            "WARN",
            "get_clipboard_string called on unsupported platform",
        );
        None
    }
}

