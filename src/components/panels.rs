use crate::components::atoms::{icon, list_item, list_label, panel_header};
use crate::components::icons::{CHEVRON_DOWN, CHEVRON_RIGHT, FILE, FOLDER, GIT};
use crate::logging;
use crate::model::TreeEntry;
use crate::services::list_dir_entries;
use crate::theme::{TREE_INDENT, UiTheme};
use floem::ext_event::{register_ext_trigger, ExtSendTrigger};
use floem::prelude::*;
use floem::reactive::Effect;
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
        Container::new(body).style(move |s| {
            s.size_full()
                .flex_grow(1.0)
                .items_stretch()
                .background(theme.panel_bg)
        }),
    ))
    .style(move |s| s.size_full().background(theme.panel_bg))
}

pub fn file_tree_view(entries: Vec<TreeEntry>, theme: UiTheme) -> impl IntoView {
    let entries = entries.into_iter().map(Arc::new).collect::<Vec<_>>();
    let entries = RwSignal::new(entries);
    let toggle_trigger = ExtSendTrigger::new();
    let pending_toggle = Arc::new(Mutex::new(None::<Instant>));

    {
        let pending_toggle = Arc::clone(&pending_toggle);
        let entries = entries;
        Effect::new(move |_| {
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
        |entry| entry.id.clone(),
        move |entry| {
            let indent = TREE_INDENT * entry.depth as f32;
            let is_dir = entry.is_dir;
            let entry_id = entry.id.clone();
            let entry_path = entry.path.clone();
            let entry_depth = entry.depth;
            let entries_signal = entries;
            let pending_toggle = Arc::clone(&pending_toggle);
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
                list_label(entry.name.clone(), theme, is_dir).into_any(),
            ))
            .style(|s| s.items_center().col_gap(6.0));

            let row = list_item(row, indent, theme);
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
                row.into_any()
            }
        },
    )
    .style(|s| s.flex_col().width_full())
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
