# Window Resize Delay Fix - Technical Specification

## Problem Statement

macOS window zoom animation causes a 2-second delay before the terminal becomes clear after the animation completes.

**Symptoms**:
- Double-click titlebar to zoom → window content becomes blurry
- Animation completes (~1s) → content stays blurry for additional 2s
- Finally becomes clear after ~3s total

**Acceptable behavior** (Lapce reference):
- Animation blur during zoom: OK (macOS captures snapshot)
- Immediate refresh after animation: Required (no 2s wait)

## Root Cause Analysis

### 1. macOS Behavior
- `full_size_content_view(true)` causes macOS to capture window snapshot during animation
- floem's `show_titlebar(false)` internally enables `full_size_content_view`
- During animation, macOS sends many `WindowResized` events

### 2. New floem (eff3b83) Issue
```rust
// floem/src/window_handle.rs - size() function
fn size(&mut self, size: Size) {
    self.style();           // Expensive: style calculation
    self.layout();          // Expensive: layout tree rebuild
    self.process_update();  // Expensive: process pending updates
    self.schedule_repaint(); // Schedules repaint
}
```

**Critical**: This executes BEFORE app's `event_before_children()` is called, so the app cannot skip these expensive operations.

### 3. Event Queue Backup
- macOS sends ~50+ resize events during 1s animation
- Each event triggers expensive floem internal processing
- Event queue backs up → takes ~2s to drain after animation

## Solution Architecture

### Approach: Pin to Lapce's floem version (e0dd862)

Lapce's version doesn't have the aggressive `size()` processing issue.

### API Compatibility Changes Required

| Old API (Lapce floem) | New API (current floem) |
|-----------------------|-------------------------|
| `create_effect(...)` | `Effect::new(...)` |
| `label(\|\| "text")` | `Label::new("text")` |
| `container(child)` | `Container::new(child)` |
| `empty()` | `Empty::new()` |
| `Event::KeyDown(KeyEvent)` | `Event::Key(KeyboardEvent)` |
| `key_event.key.logical_key` | `key_event.key` |
| `Event::PointerDown/Move/Up` | `Event::Pointer(PointerEvent::Down/Move/Up)` |
| `.keyboard_navigable()` | `.focusable(true)` |
| `Event::PointerWheel(wheel).delta.y` | `event.pixel_scroll_delta_vec2()` |
| Custom `is_selecting` RwSignal | `canvas_id.is_active()` |

### Additional Optimizations (in our code)

#### 1. Animation Burst Detection (`layout.rs`)
```rust
static ANIMATION_TIMER_ACTIVE: AtomicBool = AtomicBool::new(false);
static RESIZE_BURST_START_MS: AtomicU64 = AtomicU64::new(0);

// Skip expensive operations during animation burst
let in_animation_burst = now.saturating_sub(burst_start) < 1500;
if in_animation_burst {
    return EventPropagation::Continue; // SKIP
}
```

#### 2. Fixed-Delay Animation Timer
```rust
fn start_animation_timer() {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(1200)); // Wait for animation
        direct_terminal_resize(window_w, window_h);      // Resize on background thread
    });
}
```

#### 3. Direct Terminal Resize (`terminal.rs`)
```rust
// Bypass floem event queue - resize PTY directly
pub fn direct_terminal_resize(window_width: f64, window_height: f64) {
    // Use cached cell dimensions
    // Resize PTY on background thread (thread-safe)
    // Trigger force repaint
}
```

## Future Upgrade Path

### Option 1: Fork floem (Recommended for stability)
- Fork floem, modify `size()` to only update dimensions without layout
- Maintain fork with periodic upstream merges

### Option 2: Upstream PR
- Submit PR to floem adding resize throttling option
- Risk: May not be accepted or take time

### Option 3: Wait for upstream fix
- Monitor floem for performance improvements
- Test periodically with new versions

## Verification Checklist

When upgrading floem in the future, verify:

- [ ] Window zoom animation blur is acceptable
- [ ] No 2s delay after animation completes
- [ ] Terminal renders immediately after animation
- [ ] Log shows `WindowResized: SKIPPED (animation burst)` during animation
- [ ] Log shows `animation timer: 1.2s elapsed` after burst
- [ ] Splitter drag works smoothly
- [ ] Keyboard input works in terminal
- [ ] Scroll works in terminal
- [ ] Mouse selection works in terminal

## Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | floem version pin |
| `src/components/layout.rs` | Animation burst detection, timer, WindowResized handling |
| `src/components/terminal.rs` | Direct resize, keyboard/pointer/scroll events |
| `src/services/terminal.rs` | Global session cache, force repaint trigger |

## Commit Reference

```
b215faa fix: pin floem to Lapce version (e0dd862) to eliminate 2s resize delay
```
