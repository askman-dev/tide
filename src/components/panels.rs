use crate::model::TreeEntry;
use crate::theme::{vertical_gradient, UiColors};
use floem::prelude::*;

pub fn panel_view<V: IntoView + 'static>(title: &str, body: V, colors: UiColors) -> impl IntoView {
    let header = Container::new(
        Label::new(title)
            .style(move |s| s.font_size(11.0).font_bold().color(colors.text_muted)),
    )
    .style(move |s| {
        s.width_full()
            .padding_horiz(10.0)
            .padding_vert(6.0)
            .background(colors.panel_header)
            .border_bottom(1.0)
            .border_color(colors.border_soft)
    });

    let body = Container::new(body).style(|s| s.width_full().height_full().padding(8.0));

    v_stack((header, body)).style(move |s| {
        s.width_full()
            .row_gap(0.0)
            .background(vertical_gradient(140.0, colors.panel_top, colors.panel_bottom))
            .border(1.0)
            .border_color(colors.border_soft)
            .border_radius(10.0)
    })
}

pub fn file_tree_view(entries: Vec<TreeEntry>, colors: UiColors) -> impl IntoView {
    dyn_stack(
        move || entries.clone(),
        |entry| entry.id.clone(),
        move |entry| {
            let indent = "  ".repeat(entry.depth);
            let prefix = if entry.is_dir { "[dir] " } else { "      " };
            let is_dir = entry.is_dir;
            Label::new(format!("{indent}{prefix}{}", entry.name)).style(move |s| {
                let mut s = s
                    .font_size(11.0)
                    .color(if is_dir { colors.text } else { colors.text_muted })
                    .padding_vert(2.0);
                if is_dir {
                    s = s.font_bold();
                }
                s
            })
        },
    )
    .style(|s| {
        s.flex_col()
            .row_gap(2.0)
            .font_family("SF Mono, Menlo, Monaco".to_string())
    })
}

pub fn git_status_view(entries: Vec<String>, colors: UiColors) -> impl IntoView {
    dyn_stack(
        move || entries.clone(),
        |entry| entry.clone(),
        move |entry| {
            Label::new(entry).style(move |s| {
                s.font_size(11.0)
                    .color(colors.text_muted)
                    .padding_vert(2.0)
            })
        },
    )
    .style(|s| {
        s.flex_col()
            .row_gap(2.0)
            .font_family("SF Mono, Menlo, Monaco".to_string())
    })
}
