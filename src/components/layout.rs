use crate::theme::UiTheme;
use crate::logging;
use floem::context::EventCx;
use floem::event::{Event, EventPropagation};
use floem::prelude::*;
use floem::style::CursorStyle;
use floem::style::Style;
use floem::views::drag_window_area;
use floem::views::Empty;
use floem::ViewId;
use std::time::{Duration, Instant};

const LEFT_MIN_WIDTH: f64 = 200.0;
const CENTER_MIN_WIDTH: f64 = 300.0;
// Right pane default used to be large enough to make the min-sum exceed a typical window width,
// which caused the layout engine to overflow and our splitter hit-testing to miss in windowed mode.
const RIGHT_MIN_WIDTH: f64 = 260.0;
const HANDLE_WIDTH: f64 = 10.0;

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
                // Debounce clamp_widths during animations to avoid blocking UI thread.
                // Only clamp when animation settles or at reasonable intervals.
                let should_clamp = self.last_clamp_at.elapsed() >= Duration::from_millis(100);
                
                if should_clamp {
                    self.clamp_widths();
                    self.last_clamp_at = Instant::now();
                }
                
                // Force immediate layout and repaint to avoid macOS showing stale scaled screenshots
                self.id.request_layout();
                
                // Log less frequently to avoid spam
                if self.last_resize_log_at.elapsed() >= Duration::from_millis(500) {
                    self.last_resize_log_at = Instant::now();
                    logging::log_line(
                        "INFO",
                        &format!(
                            "window resized: {:.0}x{:.0} left={:.0} right={:.0}",
                            size.width,
                            size.height,
                            self.left_width.get_untracked(),
                            self.right_width.get_untracked()
                        ),
                    );
                }
                EventPropagation::Continue
            }
            Event::Pointer(PointerEvent::Down(button_event)) => {
                let Some(pos) = event.point() else {
                    return EventPropagation::Continue;
                };

                // Only start drag when the pointer is over a handle.
                let Some(handle) = self.handle_hit_test(pos.x) else {
                    return EventPropagation::Continue;
                };

                logging::breadcrumb(format!("split drag start: {pos:?}"));

                self.id.request_active();
                self.drag = Some(DragState {
                    handle,
                    start_x: pos.x,
                    start_left: self.left_width.get_untracked(),
                    start_right: self.right_width.get_untracked(),
                });

                // Prevent child views from interpreting this as a click.
                let _ = button_event;
                EventPropagation::Stop
            }
            Event::Pointer(PointerEvent::Move(_)) => {
                let Some(drag) = self.drag else {
                    return EventPropagation::Continue;
                };

                let Some(pos) = event.point() else {
                    return EventPropagation::Continue;
                };

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
            Event::Pointer(PointerEvent::Up(_)) => {
                if self.drag.is_some() {
                    logging::breadcrumb("split drag end".to_string());
                    self.drag = None;
                    self.id.clear_active();
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
        drag_window_area(Empty::new())
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
    Container::new(body).style(move |s| {
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

    let left = Container::new(left).style(move |s| {
        s.width(left_width.get())
            .min_width(LEFT_MIN_WIDTH)
            .height_full()
            .background(theme.panel_bg)
    });
    let center = Container::new(center).style(move |s| {
        s.flex_grow(1.0)
            .min_width(CENTER_MIN_WIDTH)
            .height_full()
            .background(theme.surface)
    });
    let right = Container::new(right).style(move |s| {
        s.width(right_width.get())
            .min_width(RIGHT_MIN_WIDTH)
            .height_full()
            .background(theme.panel_bg)
    });

    let make_handle = move || {
        Container::new(Empty::new()).style(move |s| {
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
    Container::new(content).style(move |s| {
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
    Container::new(content).style(move |s| {
        s.flex_grow(2.0)
            .height_full()
            .padding(8.0)
            .background(theme.surface)
    })
}

pub fn center_preview<V: IntoView + 'static>(content: V, theme: UiTheme) -> impl IntoView {
    Container::new(content).style(move |s| {
        s.flex_grow(0.0)
            .height_full()
            .width(0.0)
            .background(theme.surface)
    })
}
