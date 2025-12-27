use crate::components::{file_tree_view, git_status_view, panel_view};
use crate::model::WorkspaceTab;
use crate::services::{build_tree_entries, git_status_entries};
use crate::theme::{rgba, vertical_gradient, UiColors};
use floem::prelude::*;
use floem::style::{Background, CursorStyle, TextColor, Transition};
use floem::views::{slider, Empty};
use std::path::PathBuf;
use std::time::Duration;

pub fn app_view() -> impl IntoView {
    let colors = UiColors::new();
    let ui_font = "Avenir Next, SF Pro Text, Helvetica Neue".to_string();
    let initial_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let tabs = RwSignal::new(vec![build_tab(0, initial_root)]);
    let active_tab = RwSignal::new(0usize);
    let next_tab_id = RwSignal::new(1usize);

    let tab_list = dyn_stack(
        move || tabs.get(),
        |tab| tab.id,
        move |tab| {
            let tab_id = tab.id;
            let tab_name = tab.name.clone();
            Button::new(tab_name)
                .action(move || active_tab.set(tab_id))
                .style(move |s| {
                    let mut s = s
                        .padding_horiz(12.0)
                        .padding_vert(6.0)
                        .font_size(12.0)
                        .border_radius(10.0)
                        .border(1.0)
                        .border_color(colors.border_soft)
                        .cursor(CursorStyle::Pointer)
                        .transition(
                            Background,
                            Transition::ease_in_out(Duration::from_millis(140)),
                        )
                        .transition(
                            TextColor,
                            Transition::ease_in_out(Duration::from_millis(140)),
                        );
                    if active_tab.get() == tab_id {
                        s = s
                            .background(vertical_gradient(
                                80.0,
                                colors.surface_top,
                                colors.surface_bottom,
                            ))
                            .color(colors.text)
                            .border_color(colors.accent);
                    } else {
                        s = s
                            .background(colors.panel_bottom)
                            .color(colors.text_muted)
                            .hover(|s| {
                                s.background(colors.panel_top)
                                    .color(colors.text)
                                    .border_color(colors.border)
                            });
                    }
                    s
                })
        },
    )
    .style(|s| s.flex_row().col_gap(8.0));

    let new_tab_button = Button::new("+")
        .action(move || {
            let id = next_tab_id.get();
            next_tab_id.set(id + 1);
            let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            tabs.update(|tabs| tabs.push(build_tab(id, root)));
            active_tab.set(id);
        })
        .style(move |s| {
            s.padding_horiz(10.0)
                .padding_vert(6.0)
                .font_size(13.0)
                .border_radius(8.0)
                .border(1.0)
                .border_color(colors.border_soft)
                .background(colors.panel_bottom)
                .color(colors.text)
                .cursor(CursorStyle::Pointer)
                .hover(|s| {
                    s.background(colors.panel_top)
                        .border_color(colors.accent)
                        .color(colors.text)
                })
        });

    let tab_bar = h_stack((
        tab_list,
        Empty::new().style(|s| s.flex_grow(1.0)),
        new_tab_button,
    ))
    .style(move |s| {
        s.width_full()
            .height(44.0)
            .items_center()
            .padding_horiz(12.0)
            .background(vertical_gradient(
                80.0,
                colors.chrome_top,
                colors.chrome_bottom,
            ))
            .border_bottom(1.0)
            .border_color(colors.border)
    });

    let content = dyn_container(move || active_tab.get(), move |tab_id| {
        let tabs_vec = tabs.get();
        let tab = tabs_vec.into_iter().find(|tab| tab.id == tab_id);
        match tab {
            Some(tab) => workspace_view(tab, colors).into_any(),
            None => Label::new("No workspace").into_any(),
        }
    })
    .style(|s| s.size_full().padding(12.0));

    let shell = Container::new(v_stack((tab_bar, content)).style(|s| s.size_full())).style(
        move |s| {
            s.size_full()
                .background(vertical_gradient(
                    260.0,
                    colors.surface_top,
                    colors.surface_bottom,
                ))
                .border(1.0)
                .border_color(colors.border)
                .border_radius(14.0)
                .box_shadow_color(rgba(0, 0, 0, 120))
                .box_shadow_blur(20.0)
                .box_shadow_v_offset(12.0)
        },
    );

    let backdrop = Container::new(Empty::new()).style(move |s| {
        s.size_full()
            .background(vertical_gradient(900.0, colors.bg_top, colors.bg_bottom))
    });

    stack((backdrop, shell)).style(move |s| {
        s.size_full()
            .padding(14.0)
            .font_family(ui_font.clone())
            .font_size(13.0)
            .color(colors.text)
    })
}

fn workspace_view(tab: WorkspaceTab, colors: UiColors) -> impl IntoView {
    let left_width = tab.left_width;
    let file_panel = panel_view(
        "Files",
        file_tree_view(tab.file_tree, colors).scroll(),
        colors,
    )
    .style(|s| s.flex_grow(1.0));
    let git_panel = panel_view("Git", git_status_view(tab.git_status, colors).scroll(), colors)
        .style(|s| s.height(160.0));

    let left_panel = v_stack((file_panel, git_panel)).style(move |s| {
        s.width(left_width.get())
            .height_full()
            .padding(10.0)
            .row_gap(10.0)
            .background(vertical_gradient(
                320.0,
                colors.sidebar_top,
                colors.sidebar_bottom,
            ))
            .border(1.0)
            .border_color(colors.border_soft)
            .border_radius(12.0)
    });

    let resize_handle = slider::Slider::new_rw(left_width)
        .slider_style(move |s| {
            s.handle_radius(0)
                .handle_color(Some(rgba(0, 0, 0, 0).into()))
                .bar_height(100.pct())
                .bar_radius(0)
                .accent_bar_radius(0)
                .bar_color(colors.border_soft)
                .accent_bar_color(colors.border_soft)
        })
        .style(move |s| s.width(6.0).height_full().cursor(CursorStyle::ColResize));

    let terminal_body = Container::new(Label::new(
        "Terminal placeholder (Alacritty or Lapce terminal next).",
    ))
    .style(move |s| {
        s.width_full()
            .height_full()
            .padding(14.0)
            .background(vertical_gradient(
                260.0,
                colors.terminal_top,
                colors.terminal_bottom,
            ))
            .border(1.0)
            .border_color(colors.border_soft)
            .border_radius(10.0)
            .font_family("SF Mono, Menlo, Monaco".to_string())
            .font_size(12.0)
            .color(colors.text_soft)
    });

    let terminal_view = Container::new(
        v_stack((
            Label::new("Terminal")
                .style(move |s| s.font_size(12.0).font_bold().color(colors.text)),
            terminal_body,
        ))
        .style(|s| s.width_full().height_full().row_gap(8.0)),
    )
    .style(move |s| {
        s.width_full()
            .height_full()
            .padding(12.0)
            .background(vertical_gradient(
                220.0,
                colors.surface_top,
                colors.surface_bottom,
            ))
            .border(1.0)
            .border_color(colors.border_soft)
            .border_radius(12.0)
    });

    h_stack((left_panel, resize_handle, terminal_view))
        .style(|s| s.size_full().col_gap(8.0))
}

fn build_tab(id: usize, root: PathBuf) -> WorkspaceTab {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace")
        .to_string();
    let file_tree = build_tree_entries(&root, 3);
    let git_status = git_status_entries(&root);

    WorkspaceTab {
        id,
        name,
        root,
        left_width: RwSignal::new(28.pct()),
        file_tree,
        git_status,
    }
}
