use crate::components::atoms::{icon, list_item, list_label, panel_header};
use crate::components::icons::{CHEVRON_DOWN, CHEVRON_RIGHT, FILE, FOLDER, GIT};
use crate::model::TreeEntry;
use crate::theme::{TREE_INDENT, UiTheme};
use floem::prelude::*;

pub fn panel_view<V: IntoView + 'static>(
    title: &str,
    icon_svg: &'static str,
    body: V,
    theme: UiTheme,
) -> impl IntoView {
    let header_title = title.to_string();
    v_stack((
        panel_header(header_title, icon_svg, theme),
        Container::new(body).style(move |s| {
            s.width_full()
                .height_full()
                .background(theme.panel_bg)
        }),
    ))
    .style(move |s| s.width_full().background(theme.panel_bg))
}

pub fn file_tree_view(entries: Vec<TreeEntry>, theme: UiTheme) -> impl IntoView {
    dyn_stack(
        move || entries.clone(),
        |entry| entry.id.clone(),
        move |entry| {
            let indent = TREE_INDENT * entry.depth as f32;
            let is_dir = entry.is_dir;
            let chevron = if is_dir {
                if entry.expanded {
                    CHEVRON_DOWN
                } else {
                    CHEVRON_RIGHT
                }
            } else {
                ""
            };
            let icon_svg = if is_dir { FOLDER } else { FILE };
            let row = h_stack((
                if chevron.is_empty() {
                    Empty::new().style(|s| s.width(12.0).height(12.0)).into_any()
                } else {
                    icon(chevron, theme).into_any()
                },
                icon(icon_svg, theme).into_any(),
                list_label(entry.name, theme, is_dir).into_any(),
            ))
            .style(|s| s.items_center().col_gap(6.0));

            list_item(row, indent, theme)
        },
    )
    .style(|s| s.flex_col())
}

pub fn git_status_view(entries: Vec<String>, theme: UiTheme) -> impl IntoView {
    dyn_stack(
        move || entries.clone(),
        |entry| entry.clone(),
        move |entry| {
            let row = h_stack((
                icon(GIT, theme).into_any(),
                list_label(entry, theme, false).into_any(),
            ))
            .style(|s| s.items_center().col_gap(6.0));

            list_item(row, TREE_INDENT, theme)
        },
    )
    .style(|s| s.flex_col())
}
