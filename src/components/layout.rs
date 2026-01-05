use crate::theme::UiTheme;
use crate::logging;
use crate::components::terminal::direct_terminal_resize;
use floem::context::EventCx;
use floem::event::{Event, EventPropagation};
use floem::WindowIdExt;
use floem::prelude::*;
use floem::style::CursorStyle;
use floem::style::Style;
use floem::views::drag_window_area;
use floem::views::Empty;
use floem::ViewId;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const LEFT_MIN_WIDTH: f64 = 200.0;
const CENTER_MIN_WIDTH: f64 = 300.0;
// Right pane default used to be large enough to make the min-sum exceed a typical window width,
// which caused the layout engine to overflow and our splitter hit-testing to miss in windowed mode.
const RIGHT_MIN_WIDTH: f64 = 260.0;
const HANDLE_WIDTH: f64 = 10.0;

/// Animation detection: track when resize burst started.
/// We use a fixed 1.2s timer from first event, because WindowResized events themselves
/// are delayed by macOS during zoom animation (real animation is ~1s, but events arrive over 3s).
static ANIMATION_TIMER_ACTIVE: AtomicBool = AtomicBool::new(false);
static RESIZE_BURST_START_MS: AtomicU64 = AtomicU64::new(0);

/// Last known window size - updated by WindowResized events.
/// Terminal can read this to get the latest size without waiting for canvas paint.
static LAST_WINDOW_WIDTH: AtomicU64 = AtomicU64::new(0);
static LAST_WINDOW_HEIGHT: AtomicU64 = AtomicU64::new(0);

/// Get the last known window size.
pub fn get_last_window_size() -> (f64, f64) {
    let w = f64::from_bits(LAST_WINDOW_WIDTH.load(Ordering::SeqCst));
    let h = f64::from_bits(LAST_WINDOW_HEIGHT.load(Ordering::SeqCst));
    (w, h)
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Start a fixed-delay timer that directly resizes terminal 1.2s after resize starts.
/// Executes resize directly on background thread (PTY resize is thread-safe).
/// Key insight: macOS visual animation is ~1s, but WindowResized events are delayed ~2s.
fn start_animation_timer() {
    let now = current_time_ms();
    let burst_start = RESIZE_BURST_START_MS.load(Ordering::SeqCst);

    // If this is a new resize burst (>500ms since last burst started), start new timer
    let is_new_burst = now.saturating_sub(burst_start) > 500;

    if is_new_burst {
        RESIZE_BURST_START_MS.store(now, Ordering::SeqCst);

        // Only start one timer per burst
        if ANIMATION_TIMER_ACTIVE.swap(true, Ordering::SeqCst) {
            return;
        }

        std::thread::spawn(move || {
            // Wait 1.2s from burst start - this is when real animation should be done
            std::thread::sleep(Duration::from_millis(1200));

            // Get window size from our atomic storage
            let (window_w, window_h) = get_last_window_size();

            logging::log_line("DEBUG", &format!(
                "animation timer: 1.2s elapsed, executing resize directly for {:.0}x{:.0}",
                window_w, window_h
            ));

            // Execute resize directly on this background thread
            // PTY resize and term.lock() are thread-safe
            direct_terminal_resize(window_w, window_h);

            ANIMATION_TIMER_ACTIVE.store(false, Ordering::SeqCst);
        });
    }
}

#[derive(Clone, Copy)]
enum SplitHandle {
    Left,
    Right,
}

#[derive(Clone, Copy)]
struct DragState {
    handle: SplitHandle,
    start_x: f64,
    start_left: f64,
    start_right: f64,
}

struct SplitDragCapture {
    id: ViewId,
    style: Style,
    left_width: RwSignal<f64>,
    right_width: RwSignal<f64>,
    drag: Option<DragState>,
    last_resize_log_at: Instant,
    last_clamp_at: Instant,
}

impl SplitDragCapture {
    fn new(
        child: impl IntoView + 'static,
        left_width: RwSignal<f64>,
        right_width: RwSignal<f64>,
    ) -> Self {
        let id = ViewId::new();
        id.set_children_vec(vec![child.into_any()]);

        Self {
            id,
            style: Style::new().width_full().height_full().items_stretch(),
            left_width,
            right_width,
            drag: None,
            last_resize_log_at: Instant::now(),
            last_clamp_at: Instant::now(),
        }
    }

    fn total_width(&self) -> f64 {
        self.id
            .get_size()
            .map(|size| size.width)
            .unwrap_or_default()
    }

    fn handle_hit_test(&self, x: f64) -> Option<SplitHandle> {
        let total_width = self.total_width();
        let left_width = self.left_width.get_untracked();
        let right_width = self.right_width.get_untracked();

        let left_x0 = left_width;
        let left_x1 = left_width + HANDLE_WIDTH;

        let right_x0 = (total_width - right_width - HANDLE_WIDTH).max(0.0);
        let right_x1 = (total_width - right_width).max(0.0);

        if x >= left_x0 && x <= left_x1 {
            Some(SplitHandle::Left)
        } else if x >= right_x0 && x <= right_x1 {
            Some(SplitHandle::Right)
        } else {
            None
        }
    }

    fn clamp_widths(&self) {
        let total_width = self.total_width();
        let mut left = self.left_width.get_untracked();
        let mut right = self.right_width.get_untracked();

        let max_left = (total_width - CENTER_MIN_WIDTH - right - HANDLE_WIDTH * 2.0)
            .max(LEFT_MIN_WIDTH);
        left = left.clamp(LEFT_MIN_WIDTH, max_left);

        let max_right = (total_width - CENTER_MIN_WIDTH - left - HANDLE_WIDTH * 2.0)
            .max(RIGHT_MIN_WIDTH);
        right = right.clamp(RIGHT_MIN_WIDTH, max_right);

        if (left - self.left_width.get_untracked()).abs() > f64::EPSILON {
            self.left_width.set(left);
        }
        if (right - self.right_width.get_untracked()).abs() > f64::EPSILON {
            self.right_width.set(right);
        }
    }
}

impl View for SplitDragCapture {
    fn id(&self) -> ViewId {
        self.id
    }

    fn view_style(&self) -> Option<Style> {
        Some(self.style.clone())
    }

    fn event_before_children(&mut self, _cx: &mut EventCx, event: &Event) -> EventPropagation {
        match event {
            Event::WindowResized(size) => {
                let event_start = Instant::now();
                logging::breadcrumb(format!(
                    "WindowResized event: {:.0}x{:.0}",
                    size.width, size.height
                ));

                // Update last known window size
                LAST_WINDOW_WIDTH.store(size.width.to_bits(), Ordering::SeqCst);
                LAST_WINDOW_HEIGHT.store(size.height.to_bits(), Ordering::SeqCst);

                // Start animation timer if this is a new resize burst
                start_animation_timer();

                // Check if we're in animation burst mode
                let now = current_time_ms();
                let burst_start = RESIZE_BURST_START_MS.load(Ordering::SeqCst);
                let in_animation_burst = now.saturating_sub(burst_start) < 1500; // Within 1.5s of burst start

                // During animation burst, SKIP expensive operations to let event queue drain faster
                // The animation timer will handle resize after burst settles
                if in_animation_burst {
                    // Minimal processing - just update size tracking
                    let total_ms = event_start.elapsed().as_micros() as f64 / 1000.0;
                    if self.last_resize_log_at.elapsed() >= Duration::from_millis(200) {
                        self.last_resize_log_at = Instant::now();
                        logging::log_line(
                            "DEBUG",
                            &format!(
                                "WindowResized {:.0}x{:.0}: SKIPPED (animation burst) total={:.2}ms",
                                size.width, size.height, total_ms
                            ),
                        );
                    }
                    return EventPropagation::Continue;
                }

                // Not in animation burst - do full processing
                let should_clamp = self.last_clamp_at.elapsed() >= Duration::from_millis(100);

                if should_clamp {
                    let clamp_start = Instant::now();
                    self.clamp_widths();
                    self.last_clamp_at = Instant::now();
                    let clamp_ms = clamp_start.elapsed().as_micros() as f64 / 1000.0;
                    if clamp_ms > 1.0 {
                        logging::log_line("DEBUG", &format!("clamp_widths took {clamp_ms:.2}ms"));
                    }
                }

                // Force immediate layout and repaint
                let layout_start = Instant::now();
                self.id.request_layout();
                let layout_ms = layout_start.elapsed().as_micros() as f64 / 1000.0;

                let paint_start = Instant::now();
                self.id.request_paint();
                let paint_ms = paint_start.elapsed().as_micros() as f64 / 1000.0;

                let repaint_start = Instant::now();
                if let Some(window_id) = self.id.window_id() {
                    let _ = window_id.force_repaint();
                }
                let repaint_ms = repaint_start.elapsed().as_micros() as f64 / 1000.0;

                let total_ms = event_start.elapsed().as_micros() as f64 / 1000.0;

                // Log timing for resize events
                if self.last_resize_log_at.elapsed() >= Duration::from_millis(200) || total_ms > 5.0 {
                    self.last_resize_log_at = Instant::now();
                    logging::log_line(
                        "DEBUG",
                        &format!(
                            "WindowResized {:.0}x{:.0}: total={:.2}ms (layout={:.2}ms paint={:.2}ms repaint={:.2}ms)",
                            size.width, size.height, total_ms, layout_ms, paint_ms, repaint_ms
                        ),
                    );
                }
                EventPropagation::Continue
            }
            Event::PointerDown(pointer_event) => {
                let pos = pointer_event.pos;

                // Only start drag when the pointer is over a handle.
                let Some(handle) = self.handle_hit_test(pos.x) else {
                    return EventPropagation::Continue;
                };

                logging::breadcrumb(format!("split drag start: {pos:?}"));

                self.drag = Some(DragState {
                    handle,
                    start_x: pos.x,
                    start_left: self.left_width.get_untracked(),
                    start_right: self.right_width.get_untracked(),
                });

                EventPropagation::Stop
            }
            Event::PointerMove(pointer_event) => {
                let Some(drag) = self.drag else {
                    return EventPropagation::Continue;
                };

                let pos = pointer_event.pos;

                let total_width = self.total_width();
                let delta = pos.x - drag.start_x;
                match drag.handle {
                    SplitHandle::Left => {
                        let max_left = (total_width
                            - CENTER_MIN_WIDTH
                            - drag.start_right
                            - HANDLE_WIDTH * 2.0)
                            .max(LEFT_MIN_WIDTH);
                        let next_left = (drag.start_left + delta).clamp(LEFT_MIN_WIDTH, max_left);
                        self.left_width.set(next_left);
                    }
                    SplitHandle::Right => {
                        let max_right = (total_width
                            - CENTER_MIN_WIDTH
                            - drag.start_left
                            - HANDLE_WIDTH * 2.0)
                            .max(RIGHT_MIN_WIDTH);
                        let next_right =
                            (drag.start_right - delta).clamp(RIGHT_MIN_WIDTH, max_right);
                        self.right_width.set(next_right);
                    }
                }

                self.id.request_layout();
                EventPropagation::Stop
            }
            Event::PointerUp(_) => {
                if self.drag.is_some() {
                    logging::breadcrumb("split drag end".to_string());
                    self.drag = None;
                    return EventPropagation::Stop;
                }
                EventPropagation::Continue
            }
            _ => EventPropagation::Continue,
        }
    }
}

pub fn tab_bar<T: IntoView + 'static, A: IntoView + 'static>(
    tabs: T,
    actions: A,
    theme: UiTheme,
) -> impl IntoView {
    let left_padding = if cfg!(target_os = "macos") { 72.0 } else { 8.0 };
    let tabs = tabs.into_view().style(|s| s.flex_row().col_gap(6.0));

    h_stack((
        tabs,
        drag_window_area(empty())
            .style(|s| s.flex_grow(1.0).height_full()),
        actions.into_view(),
    ))
    .style(move |s| {
        s.width_full()
            .height(32.0)
            .items_center()
            .padding_left(left_padding)
            .padding_right(8.0)
            .background(theme.surface)
            .border_bottom(1.0)
            .border_color(theme.border_subtle)
    })
}

pub fn app_shell<V: IntoView + 'static>(body: V, theme: UiTheme) -> impl IntoView {
    container(body).style(move |s| {
        s.size_full()
            .items_stretch()
            .background(theme.surface)
            .font_family("SF Pro Text, Avenir Next, Helvetica Neue".to_string())
            .font_size(13.0)
            .color(theme.text)
    })
}

pub fn main_layout<L: IntoView + 'static, C: IntoView + 'static, R: IntoView + 'static>(
    left: L,
    center: C,
    right: R,
    theme: UiTheme,
) -> impl IntoView {
    let left_width = RwSignal::new(LEFT_MIN_WIDTH);
    let right_width = RwSignal::new(RIGHT_MIN_WIDTH);

    let left = container(left).style(move |s| {
        s.width(left_width.get())
            .min_width(LEFT_MIN_WIDTH)
            .height_full()
            .background(theme.panel_bg)
    });
    let center = container(center).style(move |s| {
        s.flex_grow(1.0)
            .min_width(CENTER_MIN_WIDTH)
            .height_full()
            .background(theme.surface)
    });
    let right = container(right).style(move |s| {
        s.width(right_width.get())
            .min_width(RIGHT_MIN_WIDTH)
            .height_full()
            .background(theme.panel_bg)
    });

    let make_handle = move || {
        container(empty()).style(move |s| {
            s.width(HANDLE_WIDTH)
                .height_full()
                .flex_shrink(0.0)
                .cursor(CursorStyle::ColResize)
                .pointer_events_auto()
                .background(theme.accent.with_alpha(0.15))
                .hover(|s| s.background(theme.accent.with_alpha(0.45)))
        })
    };

    let root = h_stack((left, make_handle(), center, make_handle(), right))
        .style(|s| s.size_full().items_stretch());

    SplitDragCapture::new(root, left_width, right_width)
}

pub fn right_sidebar<V: IntoView + 'static>(content: V, theme: UiTheme) -> impl IntoView {
    container(content).style(move |s| {
        s.width(260.0)
            .height_full()
            .items_stretch()
            .background(theme.panel_bg)
    })
}

pub fn sidebar_stack<V: ViewTuple + 'static>(content: V, theme: UiTheme) -> impl IntoView {
    v_stack(content).style(move |s| {
        s.width_full()
            .row_gap(0.0)
            .background(theme.panel_bg)
    })
}

pub fn main_work<V: IntoView + 'static>(content: V, theme: UiTheme) -> impl IntoView {
    container(content).style(move |s| {
        s.flex_grow(2.0)
            .height_full()
            .padding(8.0)
            .background(theme.surface)
    })
}

pub fn center_preview<V: IntoView + 'static>(content: V, theme: UiTheme) -> impl IntoView {
    container(content).style(move |s| {
        s.flex_grow(0.0)
            .height_full()
            .width(0.0)
            .background(theme.surface)
    })
}
