use crate::theme::{HEADER_FONT_SIZE, HEADER_HEIGHT, LIST_FONT_SIZE, LIST_HEIGHT, UiTheme};
use floem::event::{EventListener, EventPropagation};
use floem::menu::{Menu, MenuItem};
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
        label(move || header_text.clone()).style(move |s| {
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

/// Clickable collapse chevron indicator (rotates based on expanded state)
const COLLAPSE_CHEVRON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>"#;
const EXPAND_CHEVRON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>"#;

/// Clickable panel header with collapse/expand toggle
pub fn collapsible_panel_header(
    title: String,
    icon_svg: &'static str,
    expanded: RwSignal<bool>,
    theme: UiTheme,
) -> impl IntoView {
    let header_text = spaced_uppercase(&title);
    h_stack((
        // Collapse/expand chevron
        dyn_container(
            move || expanded.get(),
            move |is_expanded| {
                let chevron_svg = if is_expanded { COLLAPSE_CHEVRON } else { EXPAND_CHEVRON };
                svg(chevron_svg)
                    .style(move |s| {
                        s.width(10.0)
                            .height(10.0)
                            .color(theme.text_soft)
                    })
                    .into_any()
            },
        ),
        icon_soft(icon_svg, theme),
        label(move || header_text.clone()).style(move |s| {
            s.font_size(HEADER_FONT_SIZE)
                .font_bold()
                .color(theme.text_muted)
        }),
    ))
    .on_click_stop(move |_| {
        expanded.update(|v| *v = !*v);
    })
    .style(move |s| {
        s.width_full()
            .height(HEADER_HEIGHT)
            .flex_shrink(0.0) // Prevent header from shrinking when scrollbar appears
            .items_center()
            .padding_left(10.0)
            .col_gap(6.0)
            .background(theme.panel_bg)
            .cursor(CursorStyle::Pointer)
            .hover(|s| s.background(theme.element_bg))
    })
}

pub fn collapsible_panel_header_with_actions<A: IntoView + 'static>(
    title: String,
    icon_svg: &'static str,
    expanded: RwSignal<bool>,
    actions: A,
    theme: UiTheme,
) -> impl IntoView {
    let header_text = spaced_uppercase(&title);
    h_stack((
        // Left side: chevron + icon + title
        h_stack((
            // Collapse/expand chevron
            dyn_container(
                move || expanded.get(),
                move |is_expanded| {
                    let chevron_svg = if is_expanded { COLLAPSE_CHEVRON } else { EXPAND_CHEVRON };
                    svg(chevron_svg)
                        .style(move |s| {
                            s.width(10.0)
                                .height(10.0)
                                .color(theme.text_soft)
                        })
                        .into_any()
                },
            ),
            icon_soft(icon_svg, theme),
            label(move || header_text.clone()).style(move |s| {
                s.font_size(HEADER_FONT_SIZE)
                    .font_bold()
                    .color(theme.text_muted)
            }),
        ))
        .on_click_stop(move |_| {
            expanded.update(|v| *v = !*v);
        })
        .style(move |s| {
            s.flex_grow(1.0)
                .height_full()
                .items_center()
                .col_gap(6.0)
                .cursor(CursorStyle::Pointer)
        }),
        // Right side: actions
        container(actions).style(move |s| {
            s.padding_right(10.0)
                .items_center()
        }),
    ))
    .style(move |s| {
        s.width_full()
            .height(HEADER_HEIGHT)
            .flex_shrink(0.0)
            .items_center()
            .background(theme.panel_bg)
            .padding_left(10.0)
            .hover(|s| s.background(theme.element_bg))
    })
}

pub fn list_item<V: IntoView + 'static>(content: V, indent: f32, theme: UiTheme) -> impl IntoView {
    container(content)
        .on_event(EventListener::PointerDown, |_| EventPropagation::Stop)
        .style(move |s| {
            s.width_full()
                .height(LIST_HEIGHT)
                .flex_row()
                .items_center()
                .padding_left(indent)
                .padding_right(8.0)
                .hover(|s| s.background(theme.element_bg))
        })
}

pub fn list_label(text: String, theme: UiTheme, strong: bool) -> impl IntoView {
    label(move || text.clone()).style(move |s| {
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
    container(empty()).style(move |s| {
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
    label(move || text.clone()).style(move |s| {
        s.font_size(11.0)
            .color(theme.text_soft)
            .height(18.0)
            .items_center()
    })
}

/// Down arrow icon for dropdown trigger
const CHEVRON_DOWN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>"#;

/// Tab button with dropdown menu (click arrow for Open Folder, Reveal in Finder, Close)
/// tab_label is RwSignal<String> for reactive updates when workspace changes
pub fn tab_button_with_menu(
    tab_label: RwSignal<String>,
    is_active: impl Fn() -> bool + 'static + Copy,
    theme: UiTheme,
    on_click: impl Fn() + 'static,
    on_open_folder: impl Fn() + 'static + Clone,
    on_reveal_in_finder: impl Fn() + 'static + Clone,
    on_close: impl Fn() + 'static + Clone,
) -> impl IntoView {
    let on_open_folder_clone = on_open_folder.clone();
    let on_reveal_clone = on_reveal_in_finder.clone();
    let on_close_clone = on_close.clone();

    // Tab label part - clicking selects the tab
    // Uses signal.get() in closure for reactive updates
    let tab_text = label(move || tab_label.get())
        .on_click_stop(move |_| {
            on_click();
        })
        .style(move |s| {
            s.font_size(12.0)
                .cursor(CursorStyle::Pointer)
        });

    // Dropdown arrow - clicking shows the menu
    let dropdown_arrow = svg(CHEVRON_DOWN)
        .style(move |s| {
            s.width(10.0)
                .height(10.0)
                .margin_left(4.0)
                .cursor(CursorStyle::Pointer)
                .color(if is_active() { theme.accent } else { theme.text_muted })
        })
        .popout_menu(move || {
            let on_open = on_open_folder_clone.clone();
            let on_reveal = on_reveal_clone.clone();
            let on_close = on_close_clone.clone();
            Menu::new("")
                .entry(MenuItem::new("Open Folder").action(move || on_open()))
                .entry(MenuItem::new("Reveal in Finder").action(move || on_reveal()))
                .separator()
                .entry(MenuItem::new("Close").action(move || on_close()))
        });

    h_stack((tab_text, dropdown_arrow))
        .style(move |s| {
            let mut s = s
                .padding_left(10.0)
                .padding_right(6.0)
                .padding_vert(4.0)
                .border_radius(6.0)
                .items_center();
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

fn spaced_uppercase(text: &str) -> String {
    text.chars()
        .map(|c| c.to_ascii_uppercase().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}
