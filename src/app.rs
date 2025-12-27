use crate::components::{
    app_shell, file_tree_view, git_status_view, main_layout, panel_view, sidebar_stack, tab_bar,
    tab_button, terminal_placeholder, FOLDER, GIT,
};
use crate::model::WorkspaceTab;
use crate::services::{build_tree_entries, git_status_entries};
use crate::theme::UiTheme;
use floem::prelude::*;
use floem::views::Empty;
use std::path::PathBuf;
use crate::logging;

pub fn app_view() -> impl IntoView {
    let theme = UiTheme::new();
    let initial_root = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let tabs = RwSignal::new(vec![build_tab(0, initial_root)]);
    let active_tab = RwSignal::new(0usize);
    let next_tab_id = RwSignal::new(1usize);

    let tab_list = dyn_stack(
        move || tabs.get(),
        |tab| tab.id,
        move |tab| {
            let tab_id = tab.id;
            let tab_name = tab.name.clone();
            tab_button(
                tab_name,
                move || active_tab.get() == tab_id,
                theme,
                move || active_tab.set(tab_id),
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
            let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            tabs.update(|tabs| tabs.push(build_tab(id, root)));
            active_tab.set(id);
        },
    );

    let tabs_bar = tab_bar(tab_list, new_tab_button, theme);

    let content = dyn_container(move || active_tab.get(), move |tab_id| {
        let tabs_vec = tabs.get();
        let tab = tabs_vec.into_iter().find(|tab| tab.id == tab_id);
        match tab {
            Some(tab) => workspace_view(tab, theme).into_any(),
            None => Label::new("No workspace").into_any(),
        }
    });

    if let Some(path) = logging::log_path() {
        logging::log_line("INFO", &format!("log file: {}", path.display()));
    }
    app_shell(v_stack((tabs_bar, content)), theme)
}

fn workspace_view(tab: WorkspaceTab, theme: UiTheme) -> impl IntoView {
    let files_panel = panel_view(
        "Files",
        FOLDER,
        file_tree_view(tab.file_tree, theme)
            .scroll()
            .style(|s| s.width_full()),
        theme,
    );
    let git_panel = panel_view(
        "Git",
        GIT,
        git_status_view(tab.git_status, theme)
            .scroll()
            .style(|s| s.width_full()),
        theme,
    );
    let sidebar = sidebar_stack((files_panel, git_panel), theme);
    main_layout(
        terminal_placeholder(theme, tab.root.to_string_lossy().to_string()),
        Empty::new(),
        sidebar,
        theme,
        false,
    )
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
        file_tree,
        git_status,
    }
}
