use crate::theme::{HEADER_FONT_SIZE, HEADER_HEIGHT, LIST_FONT_SIZE, LIST_HEIGHT, UiTheme};
use floem::prelude::*;
use floem::style::CursorStyle;
use floem::views::svg;

pub fn icon(svg_str: &'static str, theme: UiTheme) -> impl IntoView {
    svg(svg_str).style(move |s| {
        s.width(12.0)
            .height(12.0)
            .color(theme.text_muted)
            .items_center()
            .justify_center()
    })
}

pub fn icon_soft(svg_str: &'static str, theme: UiTheme) -> impl IntoView {
    svg(svg_str).style(move |s| {
        s.width(12.0)
            .height(12.0)
            .color(theme.text_soft)
            .items_center()
            .justify_center()
    })
}

pub fn panel_header(title: String, icon_svg: &'static str, theme: UiTheme) -> impl IntoView {
    let header_text = spaced_uppercase(&title);
    h_stack((
        icon_soft(icon_svg, theme),
        Label::new(header_text).style(move |s| {
            s.font_size(HEADER_FONT_SIZE)
                .font_bold()
                .color(theme.text_muted)
        }),
    ))
    .style(move |s| {
        s.height(HEADER_HEIGHT)
            .items_center()
            .padding_left(10.0)
            .col_gap(6.0)
            .background(theme.panel_bg)
    })
}

pub fn list_item<V: IntoView + 'static>(content: V, indent: f32, theme: UiTheme) -> impl IntoView {
    Container::new(content).style(move |s| {
        s.width_full()
            .height(LIST_HEIGHT)
            .items_center()
            .padding_left(indent)
            .padding_right(8.0)
            .hover(|s| s.background(theme.element_bg))
    })
}

pub fn list_label(text: String, theme: UiTheme, strong: bool) -> impl IntoView {
    Label::new(text).style(move |s| {
        let mut s = s
            .font_size(LIST_FONT_SIZE)
            .color(if strong { theme.text } else { theme.text_muted });
        if strong {
            s = s.font_bold();
        }
        s
    })
}

pub fn splitter(theme: UiTheme) -> impl IntoView {
    Container::new(Empty::new()).style(move |s| {
        s.width(1.0)
            .height_full()
            .background(theme.border_subtle)
    })
}

pub fn tab_button(
    label: String,
    is_active: impl Fn() -> bool + 'static,
    theme: UiTheme,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    Button::new(label)
        .action(on_click)
        .style(move |s| {
            let mut s = s
                .padding_horiz(10.0)
                .padding_vert(4.0)
                .font_size(12.0)
                .border_radius(6.0)
                .cursor(CursorStyle::Pointer);
            if is_active() {
                s = s.background(theme.element_bg).color(theme.accent);
            } else {
                s = s
                    .background(theme.surface)
                    .color(theme.text_muted)
                    .hover(|s| s.background(theme.element_bg).color(theme.text));
            }
            s
        })
}

pub fn meta_text(text: String, theme: UiTheme) -> impl IntoView {
    Label::new(text).style(move |s| {
        s.font_size(11.0)
            .color(theme.text_soft)
            .height(18.0)
            .items_center()
    })
}

fn spaced_uppercase(text: &str) -> String {
    text.chars()
        .map(|c| c.to_ascii_uppercase().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}
