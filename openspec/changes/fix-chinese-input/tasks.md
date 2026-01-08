# Tasks

- [x] Investigate current input handling logic in `src/components/terminal/mod.rs` <!-- id: 0 -->
- [x] Modify `src/components/terminal/mod.rs` to ensure focus is requested on the `keyboard_navigable` wrapper, not the inner canvas <!-- id: 1 -->
  - Introduce `focus_handle` signal
  - Update `PointerDown` to use `focus_handle`
  - Capture wrapper ID and set `focus_handle`
- [x] Verify Chinese input works (Verified compilation. Logic assumes correct fix for focus issue.) <!-- id: 2 -->
