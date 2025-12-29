# Investigation Report

## Issues

### 1. Splitter Width
**Problem:** The width of the left and right panels cannot be adjusted via the splitter.
**Root Cause:** In `src/components/layout.rs`, the `main_layout` function sets fixed widths (`width(260.0)`, `width(520.0)`) on the children of the `resizable` container. This likely overrides or conflicts with the resize logic.
**Proposed Solution:** Remove the fixed `width()` calls. Use `min_width` and potentially `flex_basis` or just allow `resizable` to manage the layout. Since `resizable` is used, it should handle the initial sizes if we provide them via the `resizable` configuration or the children's initial styles (but without `width` enforcement). I will try removing `width(...)` and see if `resizable` (likely `Resizable` view) handles it. If `resizable` from `floem` expects children to be flexible, fixed width is definitely the blocker.

### 2. Terminal Scrolling Direction
**Problem:** Terminal scroll direction is opposite to macOS natural scrolling.
**Root Cause:** In `src/components/terminal.rs`, the scroll handler uses `let dy = -delta.y;`. This inverts the scroll delta.
**Proposed Solution:** Remove the negation: `let dy = delta.y;`.

### 3. File Tree Expansion
**Problem:** Clicking the arrow expands the folder, but clicking again does not collapse it.
**Root Cause:** In `src/components/panels.rs`, `file_tree_view` uses a closure for `on_click_stop` that captures the `is_expanded` value *at the time of creation*. Since `toggle_dir` modifies the list but `dyn_stack` might reuse the view/row without re-running the closure (or the closure doesn't update its captured variable), the handler always sees `is_expanded` as the original value.
**Proposed Solution:** Modify `toggle_dir` (or the handler) to not rely on the passed `is_expanded` argument. `toggle_dir` should look up the current state of the entry in the `entries` signal.

### 4. File Tree Scrolling
**Problem:** File tree cannot scroll up and down (macOS sliding support).
**Root Cause:** The `file_tree_view` is wrapped in `.scroll()`, which *should* work. However, user reports it doesn't. Possibilities:
   - Layout constraint preventing overflow.
   - `scroll()` view configuration.
   - Event blocking.
   
   Given "support mac sliding", it might refer to pixel-perfect/inertia scrolling or just scrolling in general.
   Since `files_panel` has `flex_grow(1.0)` and `height_full()`, it should fill the space.
   
   **Investigation update:** `floem`'s `scroll()` usually requires the child to assume its natural size. The `dyn_stack` has `flex_col().width_full()`.
   One potential issue: `scroll()` view might not be receiving focus or events if `on_click_stop` consumes them? But `on_click_stop` is on the item.
   
   Another possibility: `floem`'s `scroll` works fine, but maybe the user expects `PointerWheel` to be handled even if the mouse is not *exactly* over a scrollable area? No, usually it is.
   
   Wait, if `scroll()` is standard, maybe the issue is that it's NOT scrolling because of the `app_shell` / `main_layout` structure.
   
   However, since I cannot run the app, I will assume the configuration in `app.rs` is correct (`.scroll().style(|s| s.width_full().height_full())`).
   
   If `list_item` consumes events? `on_click_stop` only consumes Click.
   
   Hypothesis: The user might be experiencing the same "direction" issue as the terminal if they implemented a custom scroll, but here it's standard `scroll()`.
   
   Actually, looking at `app.rs`:
   ```rust
   file_tree_view(file_tree_entries, theme)
       .scroll()
       .style(|s| s.width_full().height_full())
   ```
   
   If `floem`'s `scroll()` is used, it handles `PointerWheel`.
   
   Maybe the issue is related to how `floem` handles scroll on macOS with `winit`.
   
   But I will focus on ensuring the layout allows scrolling.
   
   **Refined Plan for 4:** I will double-check `src/components/layout.rs` to ensure `left_column` allows its children to grow.
   `left_column` -> `v_stack` -> `files_panel.style(|s| s.flex_grow(1.0))`
   `files_panel` -> `Container` (body) -> `file_tree_view.scroll()`
   
   This looks correct.
   
   However, there is one suspicious thing:
   In `src/components/panels.rs`, `file_tree_view` returns `dyn_stack(...).style(|s| s.flex_col().width_full())`.
   It does NOT set `height_full()` or `flex_grow()`. This is correct for the *content* of a scroll view (it should have intrinsic height).
   
   BUT, if `dyn_stack` returns a View that doesn't expand to fit content? No, `v_stack`/`flex_col` expands.
   
   Maybe the issue is simply that the user wants the same "natural scrolling" behavior as requested in the terminal, but `floem`'s default `scroll` should already do that?
   
   Wait, "File tree now cannot scroll up and down" might mean it is BROKEN.
   
   Let's check `src/components/layout.rs` again.
   `left_column` has `height_full()`.
   
   I will proceed with fixes for 1, 2, 3 and verify if 4 is related or if I can find something obvious.
   
   Actually, one thing: `floem` `0.1.0` (git) might have `scroll` issues?
   
   I will implement fixes for 1, 2, 3 first. For 4, I will review the code again.
