# Task: Sidebar Blank Area Right of File Tree

## Problem Description
When running the app, the Files panel shows a large blank area to the right of the file list (see user screenshots). Expectation: the sidebar and file list should be flush to the right edge of the window; list rows should fill the sidebar width with no empty area.

## Current Layout Context
- Left: Terminal placeholder (main work)
- Center preview: removed from layout (hidden)
- Right: Sidebar (fixed 260px) with Files + Git panels

## What I Tried
1) Removed center preview from layout
- Changed `main_layout` to accept `show_center` and when false, only render left + right + a single splitter.
- Updated `src/app.rs` to call `main_layout(..., false)`.

2) Forced top bar to width_full
- Added `.width_full()` to `tab_bar` styling to ensure the shell uses full width.

3) Removed sidebar padding
- Removed `padding(8.0)` in `sidebar_stack` so the sidebar is flush to the edge.

4) Forced list container width
- Added `.width_full()` to `file_tree_view` and `git_status_view` containers so rows should fill sidebar width.

Despite these, blank space still appears to the right of the file tree (see latest screenshot).

## Relevant Files
- `src/components/layout.rs`
  - `main_layout` now conditionally renders center preview.
  - `right_sidebar` fixed width 260.
  - `sidebar_stack` no padding, row_gap 0.
- `src/components/panels.rs`
  - `file_tree_view` / `git_status_view` use `dyn_stack` and `.width_full()`
- `src/app.rs`
  - `main_layout(..., false)`

## Hypotheses
- Some parent container is not stretching to full width (maybe `panel_view` body container, or `v_stack` sizing defaults).
- `scroll()` might be wrapping content with its own sizing constraints; scroll container may not be width_full or may be capping size.
- `right_sidebar` content might not be set to width_full; in `panel_view`, the body is wrapped in `Container::new(body)` but that container might not stretch to width_full for the scroll view.
- The blank area might be coming from `main_layout` left side using `flex_grow(2)` and right sidebar not aligned, but screenshot suggests the blank area is inside the sidebar region.

## Next Steps (Not Implemented)
- Inspect `panel_view` container sizes: set `width_full()` on the `panel_view` body container AND the scroll view container itself.
- Wrap `file_tree_view(...).scroll()` in a `Container` styled with `width_full()` to force the scroll area to fill width.
- Add `debug_name()` or temporary colored backgrounds to isolate which container is not filling width.
- Verify if `scroll()` introduces a fixed width (check Floem `ScrollExt` behavior).

