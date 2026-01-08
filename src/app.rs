use crate::components::{
    app_shell, collapsible_panel_view, collapsible_panel_view_with_actions, file_tree_view,
    git_status_view, icon, main_layout, tab_bar, tab_button, tab_button_with_menu, terminal_view,
    FILE, FOLDER, GIT, REFRESH,
};
use crate::logging;
use crate::model::{TerminalPane, WorkspaceTab};
use crate::services::{
    build_tree_entries, git_status_entries, load_launchers, AppState, Launcher,
    save_state,
};
use crate::theme::UiTheme;
use floem::event::{Event, EventListener, EventPropagation};
use floem::ext_event::{register_ext_trigger, ExtSendTrigger};
use floem::keyboard::{Key, NamedKey};
use floem::prelude::*;
use floem::reactive::{Scope, create_effect, with_scope};
use floem::style::CursorStyle;
use floem::text::FamilyOwned;
use floem::views::editor::WrapProp;
use floem::views::editor::keypress::default_key_handler;
use floem::views::editor::text::{SimpleStyling, WrapMethod};
use floem::views::editor::text_document::TextDocument;
use floem::views::editor::view::editor_container_view;
use floem::views::editor::Editor;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

#[cfg(target_os = "macos")]
use std::process::Command;

static UI_WATCHDOG: OnceLock<()> = OnceLock::new();

pub fn app_view(initial_state: AppState) -> impl IntoView {
    let theme = UiTheme::new();
    install_ui_watchdog();
    
    // Load tabs from state
    let mut initial_tabs = Vec::new();
    for (i, path) in initial_state.workspaces.iter().enumerate() {
        initial_tabs.push(build_tab(i, path.clone()));
    }
    
    // Fallback if empty
    if initial_tabs.is_empty() {
        let root = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        initial_tabs.push(build_tab(0, root));
    }
    
    let next_id_val = initial_tabs.last().map(|t| t.id + 1).unwrap_or(1);
    
    let active_id = if initial_state.active_workspace_index < initial_tabs.len() {
        initial_tabs[initial_state.active_workspace_index].id
    } else {
        initial_tabs.first().map(|t| t.id).unwrap_or(0)
    };

    let launchers = RwSignal::new(load_launchers());
    let tabs = RwSignal::new(initial_tabs);
    let active_tab = RwSignal::new(active_id);
    let next_tab_id = RwSignal::new(next_id_val);

    // Effect to auto-save state
    create_effect(move |_| {
        let current_tabs = tabs.get();
        let active_id = active_tab.get();
        
        let paths: Vec<PathBuf> = current_tabs.iter().map(|t| t.root.get()).collect();
        let active_idx = current_tabs.iter().position(|t| t.id == active_id).unwrap_or(0);
        
        // logging::log_line("DEBUG", &format!("Auto-saving state: {} tabs", paths.len()));
        save_state(&paths, active_idx);
    });

    let tab_list = dyn_stack(
        move || tabs.get(),
        |tab| tab.id,
        move |tab| {
            let tab_id = tab.id;
            let tab_name_signal = tab.name;
            let tab_root_signal = tab.root;
            let tab_file_tree_signal = tab.file_tree;
            let tab_git_status_signal = tab.git_status;

            tab_button_with_menu(
                tab_name_signal,  // Pass the signal for reactive updates
                move || active_tab.get() == tab_id,
                theme,
                // on_click: select this tab
                move || {
                    active_tab.set(tab_id);
                },
                // on_open_folder: open file picker and change workspace
                move || {
                    let current_root = tab_root_signal.get();
                    logging::log_line("INFO", &format!("open folder: tab_id={tab_id}"));
                    #[cfg(target_os = "macos")]
                    {
                        // Use rfd to pick a folder
                        let dialog = rfd::FileDialog::new()
                            .set_directory(&current_root)
                            .pick_folder();
                        if let Some(new_path) = dialog {
                            logging::log_line(
                                "INFO",
                                &format!("selected folder: {}", new_path.display()),
                            );
                            // Update the tab signals - UI will react automatically
                            let name = new_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("workspace")
                                .to_string();
                            tab_name_signal.set(name);
                            tab_root_signal.set(new_path.clone());
                            tab_file_tree_signal.set(build_tree_entries(&new_path, 3));
                            tab_git_status_signal.set(git_status_entries(&new_path));
                        }
                    }
                },
                // on_reveal_in_finder: open folder in Finder
                move || {
                    let root = tab_root_signal.get();
                    logging::log_line(
                        "INFO",
                        &format!("reveal in finder: {}", root.display()),
                    );
                    #[cfg(target_os = "macos")]
                    {
                        let _ = Command::new("open").arg(&root).spawn();
                    }
                },
                // on_close: close this tab
                move || {
                    logging::log_line("INFO", &format!("close tab: id={tab_id}"));
                    tabs.update(|tabs_vec| {
                        tabs_vec.retain(|t| t.id != tab_id);
                    });
                    // If closing the active tab, switch to another tab
                    if active_tab.get() == tab_id {
                        let remaining = tabs.get();
                        if let Some(first) = remaining.first() {
                            active_tab.set(first.id);
                        }
                    }
                },
            )
        },
    );

    let new_tab_button = tab_button(
        "+".to_string(),
        || false,
        theme,
        move || {
            let id = next_tab_id.get();
            next_tab_id.set(id + 1);
            let active_id = active_tab.get();
            let root = tabs
                .get()
                .into_iter()
                .find(|tab| tab.id == active_id)
                .map(|tab| tab.root.get())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            logging::breadcrumb(format!("new tab click: id={id}"));
            logging::log_line(
                "INFO",
                &format!("new tab: id={id} root={}", root.display()),
            );
            tabs.update(|tabs| tabs.push(build_tab(id, root)));
            active_tab.set(id);
        },
    );

    // Combine tabs and + button together so + is right next to tabs
    let tabs_with_add = h_stack((tab_list, new_tab_button))
        .style(|s| s.flex_row().col_gap(6.0).items_center());

    let tabs_bar = tab_bar(tabs_with_add, empty(), theme);

    let content = dyn_container(move || active_tab.get(), move |tab_id| {
        let tabs_vec = tabs.get();
        let tab = tabs_vec.into_iter().find(|tab| tab.id == tab_id);
        match tab {
            Some(tab) => workspace_view(tab, launchers, theme).into_any(),
            None => label(|| "No workspace").into_any(),
        }
    })
    .style(|s| s.size_full().flex_grow(1.0).items_stretch());

    if let Some(path) = logging::log_path() {
        logging::log_line("INFO", &format!("log file: {}", path.display()));
    }
    app_shell(v_stack((tabs_bar, content)).style(|s| s.size_full()), theme)
        .on_event(EventListener::KeyDown, move |event| {
            if let Event::KeyDown(key_event) = event {
                if key_event.modifiers.meta() {
                    let mut current_tabs = tabs.get_untracked();
                    if current_tabs.is_empty() { return EventPropagation::Continue; }
                    
                    let active_id = active_tab.get_untracked();
                    let current_idx = current_tabs.iter().position(|t| t.id == active_id).unwrap_or(0);
                    
                    match key_event.key.logical_key {
                        Key::Named(NamedKey::ArrowLeft) => {
                            let next_idx = if current_idx == 0 {
                                current_tabs.len() - 1
                            } else {
                                current_idx - 1
                            };
                            active_tab.set(current_tabs[next_idx].id);
                            return EventPropagation::Stop;
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            let next_idx = (current_idx + 1) % current_tabs.len();
                            active_tab.set(current_tabs[next_idx].id);
                            return EventPropagation::Stop;
                        }
                        _ => {}
                    }
                }
            }
            EventPropagation::Continue
        })
}

fn install_ui_watchdog() {
    if UI_WATCHDOG.set(()).is_err() {
        return;
    }

    const PING_INTERVAL: Duration = Duration::from_millis(500);
    const STALE_AFTER: Duration = Duration::from_secs(2);

    let heartbeat_trigger = ExtSendTrigger::new();
    create_effect(move |_| {
        heartbeat_trigger.track();
        logging::touch_heartbeat();
    });

    logging::touch_heartbeat();
    logging::log_line("INFO", "UI watchdog started");

    std::thread::spawn(move || loop {
        std::thread::sleep(PING_INTERVAL);
        register_ext_trigger(heartbeat_trigger);
        logging::check_heartbeat(STALE_AFTER);
    });
}

fn workspace_view(
    tab: WorkspaceTab,
    launchers: RwSignal<Vec<Launcher>>,
    theme: UiTheme,
) -> impl IntoView {
    let workspace_name = tab.name;
    let workspace_root = tab.root;
    let file_tree_entries = tab.file_tree;
    let git_status_entries_signal = tab.git_status;
    let git_status_buffer = tab.git_status_buffer.clone();
    let editor_tabs = tab.editor_tabs;
    let active_editor_tab_id = tab.active_editor_tab;
    let terminal_panes = tab.terminal_panes;
    let focused_pane_id = tab.focused_pane_id;

    // Start polling for git status updates
    let git_status_trigger = ExtSendTrigger::new();
    let workspace_root_val = workspace_root.get_untracked();
    let git_status_trigger_clone = git_status_trigger;
    let git_status_buffer_clone = git_status_buffer.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(5));
        let entries = git_status_entries(&workspace_root_val);
        if let Ok(mut guard) = git_status_buffer_clone.lock() {
            *guard = Some(entries);
        }
        register_ext_trigger(git_status_trigger_clone);
    });

    create_effect(move |_| {
        git_status_trigger.track();
        if let Ok(mut guard) = git_status_buffer.lock() {
            if let Some(entries) = guard.take() {
                git_status_entries_signal.set(entries);
            }
        }
    });


    // Collapse state signals - all expanded by default
    let files_expanded = RwSignal::new(true);
    let changes_expanded = RwSignal::new(true);
    let history_expanded = RwSignal::new(true);

    let refresh_changes = move || {
        let root = workspace_root.get_untracked();
        let entries = git_status_entries(&root);
        git_status_entries_signal.set(entries);
        logging::breadcrumb("manual git status refresh");
    };

    let on_file_click = move |path: PathBuf, is_double_click: bool| {
        logging::breadcrumb(format!("file clicked: {} dbl={}", path.display(), is_double_click));
        
        // 1. Check if already open
        let mut tabs = editor_tabs.get_untracked();
        if let Some(existing_idx) = tabs.iter().position(|t| t.path == path) {
            let id = tabs[existing_idx].id;
            if is_double_click {
                tabs[existing_idx].is_pinned.set(true);
            }
            active_editor_tab_id.set(Some(id));
            return;
        }

        // 2. Read file content
        match crate::services::read_file_preview(&path) {
            Ok(content) => {
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                
                let new_id = tab.next_editor_tab_id.get_untracked();
                tab.next_editor_tab_id.set(new_id + 1);

                let new_tab = crate::model::EditorTab {
                    id: new_id,
                    path,
                    name,
                    is_pinned: RwSignal::new(is_double_click),
                    content,
                };

                // 3. Find temporary tab to replace (only if single click and not forcing pin)
                if !is_double_click {
                    if let Some(temp_idx) = tabs.iter().position(|t| !t.is_pinned.get_untracked()) {
                        // Replace temp tab
                        tabs[temp_idx] = new_tab;
                        editor_tabs.set(tabs);
                        active_editor_tab_id.set(Some(new_id));
                        return;
                    }
                }
                
                // Otherwise append (pinned or no temp tab found)
                tabs.push(new_tab);
                editor_tabs.set(tabs);
                active_editor_tab_id.set(Some(new_id));
            }
            Err(e) => {
                logging::log_line("ERROR", &format!("Failed to read file {}: {}", path.display(), e));
            }
        }
    };

    let on_send_to_terminal = move |path: PathBuf| {
        let path_str = path.to_string_lossy();
        let quoted = if path_str.contains(' ') {
            format!("'{}'", path_str)
        } else {
            path_str.to_string()
        };
        
        let pane_id_opt = focused_pane_id.get_untracked();
        let panes = terminal_panes.get_untracked();
        let target_pane = if let Some(id) = pane_id_opt {
            panes.iter().find(|p| p.id == id).cloned()
        } else {
            panes.first().cloned()
        };
        
        if let Some(pane) = target_pane {
            pane.should_focus.set(true);
            if let Some(session) = pane.session.get_untracked() {
                let _ = session.write(quoted.as_bytes());
            }
        }
    };

    let on_copy_path = move |path: PathBuf| {
        crate::services::set_clipboard_string(&path.to_string_lossy());
    };

    let project_header = project_header_view(workspace_name, workspace_root, theme)
        .style(|s| s.flex_shrink(0.0)); // Fixed height, don't shrink
    let files_panel = collapsible_panel_view(
        "File explorer",
        FOLDER,
        file_tree_view_reactive(
            file_tree_entries,
            on_file_click,
            on_send_to_terminal.clone(),
            on_copy_path.clone(),
            theme,
        )
        .style(|s| s.width_full()),
        files_expanded,
        theme,
    );
    let changes_panel = collapsible_panel_view_with_actions(
        "Changes",
        GIT,
        git_status_view_reactive(
            git_status_entries_signal,
            on_send_to_terminal.clone(),
            on_copy_path.clone(),
            workspace_root.get_untracked(),
            theme,
        )
        .style(|s| s.width_full()),
        changes_expanded,
        container(icon(REFRESH, theme))
            .on_click_stop(move |_| refresh_changes())
            .style(move |s| {
                s.padding(4.0)
                    .border_radius(4.0)
                    .hover(|s| s.background(theme.surface))
                    .cursor(floem::style::CursorStyle::Pointer)
            }),
        theme,
    );
    let history_panel = collapsible_panel_view(
        "History",
        FILE,
        history_view(theme)
            .style(|s| s.width_full()),
        history_expanded,
        theme,
    );

    // VSCode-style sidebar: no outer scroll, panels share space
    // Headers always visible, expanded panels divide remaining height
    let left_column = v_stack((
        project_header,
        files_panel,
        changes_panel,
        history_panel,
    ))
    .style(move |s| {
        use floem::style::{OverflowX, OverflowY};
        s.width_full()
            .height_full()
            .row_gap(4.0)
            .background(theme.panel_bg)
            .padding(6.0)
            // Critical: prevent outer overflow, force internal scrolling
            .set(OverflowX, floem::taffy::Overflow::Hidden)
            .set(OverflowY, floem::taffy::Overflow::Hidden)
    });

    let center_column = terminal_view(theme, tab, launchers);
    let right_column = editor_workspace_view(editor_tabs, active_editor_tab_id, theme);

    main_layout(left_column, center_column, right_column, theme)
}

fn project_header_view(
    name: RwSignal<String>,
    root: RwSignal<PathBuf>,
    theme: UiTheme,
) -> impl IntoView {
    v_stack((
        label(move || name.get()).style(move |s| s.font_size(13.0).font_bold().color(theme.text)),
        label(move || root.get().to_string_lossy().to_string()).style(move |s| {
            s.font_size(11.0)
                .color(theme.text_soft)
                .text_ellipsis()
        }),
    ))
    .style(move |s| {
        s.width_full()
            .padding(10.0)
            .row_gap(4.0)
            .border_bottom(1.0)
            .border_color(theme.border_subtle)
            .background(theme.panel_bg)
    })
}

fn file_tree_view_reactive<F, S, C>(
    entries: RwSignal<Vec<crate::model::TreeEntry>>,
    on_file_click: F,
    on_send_to_terminal: S,
    on_copy_path: C,
    theme: UiTheme,
) -> impl IntoView
where
    F: Fn(PathBuf, bool) + Clone + 'static,
    S: Fn(PathBuf) + Clone + 'static,
    C: Fn(PathBuf) + Clone + 'static,
{
    dyn_container(
        move || entries.get(),
        move |entries_vec| {
            file_tree_view(
                entries_vec,
                theme,
                on_file_click.clone(),
                on_send_to_terminal.clone(),
                on_copy_path.clone(),
            )
            .into_any()
        },
    )
}

/// Reactive git status view - rebuilds when signal changes
fn git_status_view_reactive<S, C>(
    entries: RwSignal<Vec<String>>,
    on_send_to_terminal: S,
    on_copy_path: C,
    workspace_root: PathBuf,
    theme: UiTheme,
) -> impl IntoView
where
    S: Fn(PathBuf) + Clone + 'static,
    C: Fn(PathBuf) + Clone + 'static,
{
    dyn_container(
        move || entries.get(),
        move |entries_vec| {
            git_status_view(
                entries_vec,
                theme,
                on_send_to_terminal.clone(),
                on_copy_path.clone(),
                workspace_root.clone(),
            )
            .into_any()
        },
    )
}

fn history_view(theme: UiTheme) -> impl IntoView {
    let entries = [
        "Refactoring the whole repo to ts",
        "Update readme",
        "Improve checkpoint requirements",
    ];
    v_stack_from_iter(entries.into_iter().map(move |entry| history_item(entry, theme)))
        .style(|s| s.flex_col().width_full())
}

fn history_item(text: &str, theme: UiTheme) -> impl IntoView {
    let text = text.to_string();
    container(label(move || text.clone()).style(move |s| {
        s.font_size(12.0)
            .color(theme.text_muted)
            .text_ellipsis()
    }))
    .style(move |s| {
        s.width_full()
            .height(22.0)
            .padding_horiz(10.0)
            .items_center()
            .hover(|s| s.background(theme.element_bg))
    })
}

fn chat_workspace_view(theme: UiTheme) -> impl IntoView {
    let chat_thread = chat_thread_view(theme).style(|s| s.flex_grow(1.0));
    let input = chat_input_view(theme);
    let content = v_stack((chat_thread, input)).style(|s| s.size_full());
    let menu = context_menu_view(theme).style(move |s| {
        s.absolute()
            .inset_left(160.0)
            .inset_top(120.0)
            .z_index(1)
    });

    stack((content, menu))
        .style(move |s| s.size_full().background(theme.surface))
}

fn chat_thread_view(theme: UiTheme) -> impl IntoView {
    let card_primary = chat_card_view("You", "Refactoring the whole repo to ts", true, theme);
    let card_secondary = chat_card_view("Assistant", "Update index", false, theme);
    let card_review = chat_card_view("You", "Review changes", true, theme);
    v_stack((card_primary, card_secondary, card_review))
        .style(|s| s.flex_col().row_gap(12.0).padding(16.0).width_full())
        .scroll()
        .style(|s| s.flex_grow(1.0))
}

fn chat_card_view(title: &str, body: &str, is_primary: bool, theme: UiTheme) -> impl IntoView {
    let title = title.to_string();
    let body = body.to_string();
    let header = label(move || title.clone()).style(move |s| {
        s.font_size(11.0)
            .color(theme.text_soft)
            .text_ellipsis()
    });
    let message = label(move || body.clone()).style(move |s| {
        s.font_size(12.0)
            .color(theme.text)
            .text_ellipsis()
    });

    let background = if is_primary {
        theme.element_bg
    } else {
        theme.panel_bg
    };

    container(v_stack((header, message)).style(|s| s.row_gap(6.0)))
        .style(move |s| {
            s.width_full()
                .padding(12.0)
                .border(1.0)
                .border_color(theme.border_subtle)
                .border_radius(8.0)
                .background(background)
        })
}

fn chat_input_view(theme: UiTheme) -> impl IntoView {
    container(label(|| "Ask Tide").style(move |s| {
        s.font_size(12.0).color(theme.text_soft)
    }))
    .style(move |s| {
        s.width_full()
            .height(36.0)
            .padding_horiz(12.0)
            .items_center()
            .border(1.0)
            .border_color(theme.border_subtle)
            .border_radius(8.0)
            .background(theme.panel_bg)
    })
    .style(|s| s.margin(12.0))
}

fn context_menu_view(theme: UiTheme) -> impl IntoView {
    let entries = [
        "New Chat",
        "Branch Chat",
        "Close",
        "Split Right",
        "Split Left",
        "Split Up",
        "Split Down",
    ];
    v_stack_from_iter(entries.into_iter().map(move |entry| menu_item_view(entry, theme)))
        .style(move |s| {
        s.width(160.0)
            .padding_vert(6.0)
            .border(1.0)
            .border_color(theme.border_subtle)
            .border_radius(8.0)
            .background(theme.panel_bg)
    })
}

fn menu_item_view(label_text: &str, theme: UiTheme) -> impl IntoView {
    let label_text = label_text.to_string();
    container(label(move || label_text.clone()).style(move |s| {
        s.font_size(12.0).color(theme.text)
    }))
    .style(move |s| {
        s.width_full()
            .height(26.0)
            .padding_horiz(10.0)
            .items_center()
            .hover(|s| s.background(theme.element_bg))
    })
}

fn read_only_code_viewer(content: String, theme: UiTheme) -> impl IntoView {
    // `floem::views::text_editor::text_editor` hard-codes `is_active = |_| true`, which means it
    // continuously calls `set_ime_cursor_area` even when it isn't focused. That breaks terminal
    // IME: the OS preedit UI appears anchored to the editor instead of the focused terminal.
    //
    // Build a minimal editor container with an `is_active` gate tied to focus.
    let scope = Scope::current().create_child();

    let mut style_builder = SimpleStyling::builder();
    style_builder
        .font_size(12)
        .line_height(1.25)
        .font_family(
            FamilyOwned::parse_list("Menlo, Monaco, 'Courier New', monospace").collect(),
        );
    let styling = Rc::new(style_builder.build());

    let doc = Rc::new(TextDocument::new(scope, content));
    let editor = Editor::new(scope, doc, styling, false);
    editor.read_only.set(true);
    let editor_sig = scope.create_rw_signal(editor);

    with_scope(scope, move || {
        // Keep this view non-active: it must not drive global IME state or cursor area, since it's
        // a read-only viewer and the terminal should own IME when focused.
        editor_container_view(editor_sig, |_| false, default_key_handler(editor_sig))
    })
    .style(move |s| {
        s.flex_grow(1.0)
            .size_full()
            .background(theme.surface)
            .color(theme.text)
            .set(WrapProp, WrapMethod::None)
    })
}

fn editor_workspace_view(
    editor_tabs: RwSignal<Vec<crate::model::EditorTab>>,
    active_tab_id: RwSignal<Option<usize>>,
    theme: UiTheme,
) -> impl IntoView {
    let tabs = dyn_stack(
        move || editor_tabs.get(),
        |tab| tab.id,
        move |tab| {
            let id = tab.id;
            // Create a reactive closure for active state
            let is_active = move || active_tab_id.get() == Some(id);
            let editor_tabs = editor_tabs;
            let active_tab_id = active_tab_id;
            let is_pinned_signal = tab.is_pinned;
            
            editor_tab_view(
                tab.clone(),
                is_active,
                move || {
                    active_tab_id.set(Some(id));
                },
                move || {
                    // Close tab
                    let mut tabs = editor_tabs.get_untracked();
                    if let Some(idx) = tabs.iter().position(|t| t.id == id) {
                        tabs.remove(idx);
                        let next_active = if tabs.is_empty() {
                            None
                        } else {
                            Some(tabs[idx.min(tabs.len() - 1)].id)
                        };
                        editor_tabs.set(tabs);
                        active_tab_id.set(next_active);
                    }
                },
                move || {
                    // Pin tab on double click
                    is_pinned_signal.set(true);
                },
                theme,
            )
        },
    )
    .style(move |s| {
        s.height(30.0)
            .items_center()
            .padding_horiz(8.0)
            .col_gap(2.0)
            .border_bottom(1.0)
            .border_color(theme.border_subtle)
            .background(theme.panel_bg)
    });

    let editor_body = dyn_container(
        move || (active_tab_id.get(), editor_tabs.get()),
        move |(id_opt, tabs)| {
            if let Some(id) = id_opt {
                if let Some(tab) = tabs.iter().find(|t| t.id == id) {
                    let content = tab.content.clone();
                    return read_only_code_viewer(content, theme).into_any();
                }
            }
            
            // Empty state
            container(label(|| "No file open"))
                .style(move |s| {
                    s.flex_grow(1.0)
                        .size_full()
                        .items_center()
                        .justify_center()
                        .background(theme.surface)
                        .color(theme.text_soft)
                })
                .into_any()
        },
    )
    .style(|s| s.flex_grow(1.0).size_full());

    v_stack((tabs, editor_body)).style(|s| s.size_full())
}

fn editor_tab_view<F>(
    tab: crate::model::EditorTab,
    is_active: F,
    on_click: impl Fn() + 'static,
    on_close: impl Fn() + 'static,
    on_double_click: impl Fn() + 'static,
    theme: UiTheme,
) -> impl IntoView 
where F: Fn() -> bool + 'static + Clone
{
    let is_active_bg = is_active.clone();
    let background = move || if is_active_bg() {
        theme.element_bg
    } else {
        theme.panel_bg
    };
    
    let is_active_border = is_active.clone();
    let border_color = move || if is_active_border() {
        theme.accent
    } else {
        floem::peniko::Color::TRANSPARENT
    };
    
    let is_pinned = tab.is_pinned;
    let font_style = move || if is_pinned.get() {
        floem::text::Style::Normal
    } else {
        floem::text::Style::Italic
    };
    
    let is_active_font = is_active.clone();
    let font_weight = move || if is_active_font() {
        floem::text::Weight::BOLD
    } else {
        floem::text::Weight::NORMAL
    };

    let is_active_text = is_active.clone();
    h_stack((
        label(move || tab.name.clone()).style(move |s| {
            s.font_size(12.0)
                .font_style(font_style())
                .font_weight(font_weight())
                .color(if is_active_text() { theme.text } else { theme.text_muted })
                .text_ellipsis()
        }),
        // Close button
        label(|| "×")
            .on_click_stop(move |_| on_close())
            .style(move |s| {
                s.margin_left(6.0)
                    .font_size(14.0)
                    .color(theme.text_soft)
                    .hover(|s| s.color(theme.text))
                    .cursor(CursorStyle::Pointer)
            }),
    ))
    .on_click_stop(move |_| on_click())
    .on_event(EventListener::DoubleClick, move |_| {
        on_double_click();
        EventPropagation::Stop
    })
    .style(move |s| {
        s.height(26.0)
            .padding_horiz(8.0)
            .items_center()
            .border_top(2.0)
            .border_color(border_color())
            .background(background())
            .cursor(CursorStyle::Pointer)
            .hover(|s| s.background(theme.element_bg))
    })
}

fn build_tab(id: usize, root: PathBuf) -> WorkspaceTab {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace")
        .to_string();
    let file_tree = build_tree_entries(&root, 3);
    let git_status = git_status_entries(&root);

    // Create initial terminal pane
    let initial_pane = TerminalPane {
        id: 0,
        session: RwSignal::new(None),
        trigger: ExtSendTrigger::new(),
        flex_ratio: RwSignal::new(1.0),
        title: RwSignal::new("Terminal".to_string()),
        should_focus: RwSignal::new(false),
        title_buffer: Arc::new(Mutex::new(None)),
    };

    WorkspaceTab {
        id,
        name: RwSignal::new(name),
        root: RwSignal::new(root.clone()),
        file_tree: RwSignal::new(build_tree_entries(&root, 0)),
        git_status: RwSignal::new(git_status_entries(&root)),
        git_status_buffer: Arc::new(Mutex::new(None)),
        editor_tabs: RwSignal::new(Vec::new()),
        active_editor_tab: RwSignal::new(None),
        focused_pane_id: RwSignal::new(None),
        terminal_panes: RwSignal::new(vec![initial_pane]),
        next_pane_id: RwSignal::new(1),
        next_editor_tab_id: RwSignal::new(0),
    }
}
