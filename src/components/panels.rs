use crate::components::atoms::{
    collapsible_panel_header, collapsible_panel_header_with_actions, icon, list_item, list_label,
    panel_header,
};
use crate::components::icons::{CHEVRON_DOWN, CHEVRON_RIGHT, FILE, FOLDER, GIT};
use crate::logging;
use crate::model::TreeEntry;
use crate::services::list_dir_entries;
use crate::theme::{TREE_INDENT, UiTheme};
use floem::event::{EventListener, EventPropagation};
use floem::ext_event::{register_ext_trigger, ExtSendTrigger};
use floem::prelude::*;
use floem::style::Display;
use floem::reactive::create_effect;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn panel_view<V: IntoView + 'static>(
    title: &str,
    icon_svg: &'static str,
    body: V,
    theme: UiTheme,
) -> impl IntoView {
    let header_title = title.to_string();
    v_stack((
        panel_header(header_title, icon_svg, theme),
        container(body).style(move |s| {
            s.size_full()
                .flex_grow(1.0)
                .items_stretch()
                .background(theme.panel_bg)
        }),
    ))
    .style(move |s| s.size_full().background(theme.panel_bg))
}

/// Collapsible panel with internal scrolling (VSCode-style)
/// - Header is always visible with fixed height
/// - Expanded panels share available space via flex-grow
/// - Body scrolls internally, no outer scrollbars
/// - Multiple expanded panels divide space equally
pub fn collapsible_panel_view<V: IntoView + 'static>(
    title: &str,
    icon_svg: &'static str,
    body: V,
    expanded: RwSignal<bool>,
    theme: UiTheme,
) -> impl IntoView {
    let header_title = title.to_string();

    // Body container: internal scroll, no horizontal overflow
    let body_container = container(body)
        .style(|s| s.width_full())
        .scroll()
        .scroll_style(|s| s.overflow_clip(true)) // Hide horizontal scrollbar
        .style(move |s| {
            use floem::style::OverflowX;
            let mut style = s.width_full().background(theme.panel_bg);
            if expanded.get() {
                // When expanded: take available space, allow internal scroll
                // flex_basis(0) ensures content height is IGNORED in flex calculations
                style = style
                    .flex_grow(1.0)
                    .flex_shrink(1.0)
                    .flex_basis(0)  // Ignore content height
                    .min_height(0)  // Allow shrinking to 0
                    .set(OverflowX, floem::taffy::Overflow::Hidden); // No horizontal scroll
            } else {
                // When collapsed: completely hidden
                style = style.display(Display::None);
            }
            style
        });

    v_stack((
        collapsible_panel_header(header_title, icon_svg, expanded, theme),
        body_container,
    ))
    .style(move |s| {
        use crate::theme::HEADER_HEIGHT;
        use floem::style::OverflowY;
        if expanded.get() {
            // Expanded: use flex_basis(0) pattern to IGNORE content height
            // This is CSS "flex: 1 1 0" - distributes space purely by flex-grow
            // Without flex_basis(0), content height affects flex calculations
            // which causes sibling panels (terminal) to resize unexpectedly
            s.width_full()
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .flex_basis(0)  // Critical: ignore content height in flex calculations
                .min_height(HEADER_HEIGHT + 20.0) // At least header + some content visible
                .set(OverflowY, floem::taffy::Overflow::Hidden) // Prevent overflow
                .background(theme.panel_bg)
        } else {
            // Collapsed: only header height, don't grow
            s.width_full()
                .flex_grow(0.0)
                .flex_shrink(0.0)
                .height(HEADER_HEIGHT) // Exact header height
                .background(theme.panel_bg)
        }
    })
}

pub fn collapsible_panel_view_with_actions<V: IntoView + 'static, A: IntoView + 'static>(
    title: &str,
    icon_svg: &'static str,
    body: V,
    expanded: RwSignal<bool>,
    actions: A,
    theme: UiTheme,
) -> impl IntoView {
    let header_title = title.to_string();

    let body_container = container(body)
        .style(|s| s.width_full())
        .scroll()
        .scroll_style(|s| s.overflow_clip(true))
        .style(move |s| {
            use floem::style::OverflowX;
            let mut style = s.width_full().background(theme.panel_bg);
            if expanded.get() {
                style = style
                    .flex_grow(1.0)
                    .flex_shrink(1.0)
                    .flex_basis(0)
                    .min_height(0)
                    .set(OverflowX, floem::taffy::Overflow::Hidden);
            } else {
                style = style.display(Display::None);
            }
            style
        });

    v_stack((
        collapsible_panel_header_with_actions(header_title, icon_svg, expanded, actions, theme),
        body_container,
    ))
    .style(move |s| {
        use crate::theme::HEADER_HEIGHT;
        use floem::style::OverflowY;
        if expanded.get() {
            s.width_full()
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .flex_basis(0)
                .min_height(HEADER_HEIGHT + 20.0)
                .set(OverflowY, floem::taffy::Overflow::Hidden)
                .background(theme.panel_bg)
        } else {
            s.width_full()
                .flex_grow(0.0)
                .flex_shrink(0.0)
                .height(HEADER_HEIGHT)
                .background(theme.panel_bg)
        }
    })
}

pub fn file_tree_view<F, S, C>(
    entries: Vec<TreeEntry>,
    theme: UiTheme,
    on_file_click: F,
    on_send_to_terminal: S,
    on_copy_path: C,
) -> impl IntoView
where
    F: Fn(std::path::PathBuf, bool) + Clone + 'static,
    S: Fn(std::path::PathBuf) + Clone + 'static,
    C: Fn(std::path::PathBuf) + Clone + 'static,
{
    let entries = entries.into_iter().map(Arc::new).collect::<Vec<_>>();
    let entries = RwSignal::new(entries);
    let toggle_trigger = ExtSendTrigger::new();
    let pending_toggle = Arc::new(Mutex::new(None::<Instant>));

    {
        let pending_toggle = Arc::clone(&pending_toggle);
        let entries = entries;
        create_effect(move |_| {
            toggle_trigger.track();
            if let Ok(mut pending) = pending_toggle.lock() {
                if let Some(start) = pending.take() {
                    let elapsed = start.elapsed().as_millis();
                    logging::log_line(
                        "INFO",
                        &format!(
                            "file_tree toggle post-update: entries={} ms={}",
                            entries.get_untracked().len(),
                            elapsed
                        ),
                    );
                }
            }
        });
    }

    dyn_stack(
        move || entries.get(),
        |entry| (entry.id.clone(), entry.expanded),
        move |entry| {
            let indent = TREE_INDENT * entry.depth as f32;
            let is_dir = entry.is_dir;
            let entry_id = entry.id.clone();
            let entry_path = entry.path.clone();
            let entry_depth = entry.depth;
            let entries_signal = entries;
            let pending_toggle = Arc::clone(&pending_toggle);
            let on_file_click = on_file_click.clone();
            let on_send_to_terminal = on_send_to_terminal.clone();
            let on_copy_path = on_copy_path.clone();
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
                    empty().style(|s| s.width(12.0).height(12.0)).into_any()
                } else {
                    icon(chevron, theme).into_any()
                },
                icon(icon_svg, theme).into_any(),
                list_label(entry.name.clone(), theme, is_dir).into_any(),
            ))
            .style(|s| s.items_center().col_gap(6.0));

            let path = entry_path.clone();
            let path_for_menu = path.clone();
            let on_send = on_send_to_terminal.clone();
            let on_copy = on_copy_path.clone();
            
            let row = list_item(row, indent, theme)
                .context_menu(move || {
                    let p1 = path_for_menu.clone();
                    let p2 = path_for_menu.clone();
                    let on_send = on_send.clone();
                    let on_copy = on_copy.clone();
                    floem::menu::Menu::new("")
                        .entry(floem::menu::MenuItem::new("Send Path to Terminal").action(move || on_send(p1.clone())))
                        .entry(floem::menu::MenuItem::new("Copy Absolute Path").action(move || on_copy(p2.clone())))
                });

            if is_dir {
                row.on_click_stop(move |_| {
                    logging::measure_ui_event("file_tree toggle", || {
                        logging::breadcrumb(format!("file_tree toggle click: {entry_id}"));
                        if let Ok(mut pending) = pending_toggle.lock() {
                            *pending = Some(Instant::now());
                            register_ext_trigger(toggle_trigger);
                        }
                        toggle_dir(
                            entries_signal,
                            entry_id.clone(),
                            entry_path.clone(),
                            entry_depth,
                        );
                    });
                })
                .into_any()
            } else {
                let path = entry_path.clone();
                let path_dbl = entry_path.clone();
                let on_click = on_file_click.clone();
                let on_dbl_click = on_file_click.clone();
                
                row.on_click_stop(move |_| {
                    on_click(path.clone(), false);
                })
                .on_event(EventListener::DoubleClick, move |_| {
                    on_dbl_click(path_dbl.clone(), true);
                    EventPropagation::Stop
                })
                .into_any()
            }
        },
    )
    .style(|s| s.flex_col().width_full())
}

pub fn git_status_view<S, C>(
    entries: Vec<String>,
    theme: UiTheme,
    on_send_to_terminal: S,
    on_copy_path: C,
    workspace_root: std::path::PathBuf,
) -> impl IntoView
where
    S: Fn(std::path::PathBuf) + Clone + 'static,
    C: Fn(std::path::PathBuf) + Clone + 'static,
{
    dyn_stack(
        move || entries.clone(),
        |entry| entry.clone(),
        move |entry| {
            let on_send = on_send_to_terminal.clone();
            let on_copy = on_copy_path.clone();
            let root = workspace_root.clone();
            
            // Extract relative path from porcelain status line (e.g., " M src/main.rs")
            // Porcelain status: XY path
            let relative_path = if entry.len() > 3 {
                entry[3..].trim().to_string()
            } else {
                entry.clone()
            };
            
            let full_path = root.join(&relative_path);
            let path_for_menu = full_path.clone();

            let row = h_stack((
                icon(GIT, theme).into_any(),
                list_label(entry.clone(), theme, false).into_any(),
            ))
            .style(|s| s.items_center().col_gap(6.0));

            list_item(row, TREE_INDENT, theme)
                .context_menu(move || {
                    let p1 = path_for_menu.clone();
                    let p2 = path_for_menu.clone();
                    let on_send = on_send.clone();
                    let on_copy = on_copy.clone();
                    floem::menu::Menu::new("")
                        .entry(floem::menu::MenuItem::new("Send Path to Terminal").action(move || on_send(p1.clone())))
                        .entry(floem::menu::MenuItem::new("Copy Absolute Path").action(move || on_copy(p2.clone())))
                })
        },
    )
    .style(|s| s.flex_col().width_full())
}

fn toggle_dir(
    entries: RwSignal<Vec<Arc<TreeEntry>>>,
    entry_id: String,
    entry_path: std::path::PathBuf,
    entry_depth: usize,
) {
    let start = Instant::now();
    let path_display = entry_path.display().to_string();
    entries.update(|entries| {
        let Some(index) = entries.iter().position(|entry| entry.id == entry_id) else {
            return;
        };
        let before_len = entries.len();
        if entries[index].expanded {
            let mut updated = (*entries[index]).clone();
            updated.expanded = false;
            entries[index] = Arc::new(updated);
            let mut remove_count = 0;
            for entry in entries.iter().skip(index + 1) {
                if entry.depth > entry_depth {
                    remove_count += 1;
                } else {
                    break;
                }
            }
            if remove_count > 0 {
                entries.drain(index + 1..index + 1 + remove_count);
            }
            let after_len = entries.len();
            logging::log_line(
                "INFO",
                &format!(
                    "file_tree toggle: collapse path={} depth={} before={} removed={} after={} ms={}",
                    path_display,
                    entry_depth,
                    before_len,
                    remove_count,
                    after_len,
                    start.elapsed().as_millis()
                ),
            );
        } else {
            let mut updated = (*entries[index]).clone();
            updated.expanded = true;
            entries[index] = Arc::new(updated);
            let children = list_dir_entries(&entry_path, entry_depth + 1)
                .into_iter()
                .map(Arc::new)
                .collect::<Vec<_>>();
            let child_count = children.len();
            entries.splice(index + 1..index + 1, children);
            let after_len = entries.len();
            logging::log_line(
                "INFO",
                &format!(
                    "file_tree toggle: expand path={} depth={} before={} added={} after={} ms={}",
                    path_display,
                    entry_depth,
                    before_len,
                    child_count,
                    after_len,
                    start.elapsed().as_millis()
                ),
            );
        }
    });
}
