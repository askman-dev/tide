use crate::theme::UiTheme;
use floem::prelude::*;
use floem::style::{CustomStyle, CustomStylable};
use floem::views::Empty;

pub fn tab_bar<T: IntoView + 'static, A: IntoView + 'static>(
    tabs: T,
    actions: A,
    theme: UiTheme,
) -> impl IntoView {
    let left_padding = if cfg!(target_os = "macos") { 72.0 } else { 8.0 };
    let tabs = tabs.into_view().style(|s| s.flex_row().col_gap(6.0));
    h_stack((
        tabs,
        Empty::new().style(|s| s.flex_grow(1.0)),
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
    let left = Container::new(left).style(move |s| {
        s.width(260.0)
            .min_width(200.0)
            .height_full()
            .background(theme.panel_bg)
    });
    let center = Container::new(center).style(move |s| {
        s.flex_grow(1.0)
            .min_width(360.0)
            .height_full()
            .background(theme.surface)
    });
    let right = Container::new(right).style(move |s| {
        s.width(520.0)
            .min_width(420.0)
            .height_full()
            .background(theme.panel_bg)
    });

    resizable::resizable((left, center, right))
        .style(|s| s.size_full())
        .custom_style(move |s| {
            s.handle_color(theme.border_subtle)
                .handle_thickness(6.0)
                .active(|s| s.handle_color(theme.accent))
        })
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
