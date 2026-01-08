# Fix Chinese Input Focus Stealing

## Background
The user reported two issues after the previous fix:
1. When a file is open in the preview pane, Chinese input in the terminal is sometimes redirected to the file preview.
2. The file preview pane accepts text input even though it should be read-only.

## Problem
- The `text_editor` used for file preview is editable by default, causing it to accept input.
- The `text_editor` might be capturing global IME events or retaining focus in a way that interferes with the terminal's IME handling.

## Solution
- Explicitly set the `text_editor` to `.read_only()` in `src/app.rs`.
- This ensures the file preview acts as a viewer, not an editor, and should reject input.
- This change is expected to mitigate the input stealing issue as well.

## Risks
- If `read_only()` doesn't stop IME event capture, the issue might persist (requiring deeper `floem` investigation).
