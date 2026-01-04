use crate::components::{
    app_shell, file_tree_view, git_status_view, main_layout, panel_view, tab_bar, tab_button,
    terminal_view, FILE, FOLDER, GIT,
};
use crate::model::WorkspaceTab;
use crate::services::{build_tree_entries, git_status_entries};
use crate::theme::UiTheme;
use floem::ext_event::{register_ext_trigger, ExtSendTrigger};
use floem::prelude::*;
use floem::reactive::Effect;
use crate::logging;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

static UI_WATCHDOG: OnceLock<()> = OnceLock::new();

pub fn app_view() -> impl IntoView {
    let theme = UiTheme::new();
    install_ui_watchdog();
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
                move || {
                    logging::breadcrumb(format!("tab select: id={tab_id}"));
                    active_tab.set(tab_id);
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
                .map(|tab| tab.root)
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

    let tabs_bar = tab_bar(tab_list, new_tab_button, theme);

    let content = dyn_container(move || active_tab.get(), move |tab_id| {
        let tabs_vec = tabs.get();
        let tab = tabs_vec.into_iter().find(|tab| tab.id == tab_id);
        match tab {
            Some(tab) => workspace_view(tab, theme).into_any(),
            None => Label::new("No workspace").into_any(),
        }
    })
    .style(|s| s.size_full().flex_grow(1.0).items_stretch());

    if let Some(path) = logging::log_path() {
        logging::log_line("INFO", &format!("log file: {}", path.display()));
    }
    app_shell(v_stack((tabs_bar, content)).style(|s| s.size_full()), theme)
}

fn install_ui_watchdog() {
    if UI_WATCHDOG.set(()).is_err() {
        return;
    }

    const PING_INTERVAL: Duration = Duration::from_millis(500);
    const STALE_AFTER: Duration = Duration::from_secs(2);

    let heartbeat_trigger = ExtSendTrigger::new();
    Effect::new(move |_| {
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

fn workspace_view(tab: WorkspaceTab, theme: UiTheme) -> impl IntoView {
    let workspace_name = tab.name.clone();
    let workspace_path = tab.root.to_string_lossy().to_string();
    let file_tree_entries = tab.file_tree.clone();
    let git_status_entries = tab.git_status.clone();

    let project_header = project_header_view(workspace_name, workspace_path, theme);
    let files_panel = panel_view(
        "File explorer",
        FOLDER,
        file_tree_view(file_tree_entries, theme)
            .scroll()
            .style(|s| s.width_full().height_full()),
        theme,
    );
    let changes_panel = panel_view(
        "Changes",
        GIT,
        git_status_view(git_status_entries, theme)
            .scroll()
            .style(|s| s.width_full().height_full()),
        theme,
    )
    .style(|s| s.height(140.0));
    let history_panel = panel_view(
        "History",
        FILE,
        history_view(theme)
            .scroll()
            .style(|s| s.width_full().height_full()),
        theme,
    )
    .style(|s| s.height(140.0));

    let left_column = v_stack((
        project_header,
        files_panel.style(|s| s.flex_grow(1.0)),
        changes_panel,
        history_panel,
    ))
    .style(move |s| {
        s.width_full()
            .height_full()
            .row_gap(6.0)
            .background(theme.panel_bg)
            .padding(6.0)
    });

    let center_column = terminal_view(theme, tab);
    let right_column = editor_workspace_view(theme);

    main_layout(left_column, center_column, right_column, theme)
}

fn project_header_view(name: String, path: String, theme: UiTheme) -> impl IntoView {
    v_stack((
        Label::new(name).style(move |s| s.font_size(13.0).font_bold().color(theme.text)),
        Label::new(path).style(move |s| {
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
    Container::new(Label::new(text.to_string()).style(move |s| {
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
    let header = Label::new(title.to_string()).style(move |s| {
        s.font_size(11.0)
            .color(theme.text_soft)
            .text_ellipsis()
    });
    let message = Label::new(body.to_string()).style(move |s| {
        s.font_size(12.0)
            .color(theme.text)
            .text_ellipsis()
    });

    let background = if is_primary {
        theme.element_bg
    } else {
        theme.panel_bg
    };

    Container::new(v_stack((header, message)).style(|s| s.row_gap(6.0)))
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
    Container::new(Label::new("Ask Tide".to_string()).style(move |s| {
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

fn menu_item_view(label: &str, theme: UiTheme) -> impl IntoView {
    Container::new(Label::new(label.to_string()).style(move |s| {
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

fn editor_workspace_view(theme: UiTheme) -> impl IntoView {
    let tabs = h_stack((
        editor_tab_view("index.ts", true, theme),
        editor_tab_view("example.com", false, theme),
        editor_tab_view("spec-checkpoint-requirement.md", false, theme),
    ))
    .style(move |s| {
        s.height(30.0)
            .items_center()
            .padding_horiz(8.0)
            .col_gap(6.0)
            .border_bottom(1.0)
            .border_color(theme.border_subtle)
            .background(theme.panel_bg)
    });

    let editor_body = Container::new(Label::new(
        "Editor placeholder (code, web previews, or docs appear here).",
    ))
    .style(move |s| {
        s.flex_grow(1.0)
            .padding(16.0)
            .background(theme.surface)
            .color(theme.text_soft)
            .font_family("SF Mono, Menlo, Monaco".to_string())
            .font_size(12.0)
    });

    v_stack((tabs, editor_body)).style(|s| s.size_full())
}

fn editor_tab_view(label: &str, is_active: bool, theme: UiTheme) -> impl IntoView {
    let background = if is_active {
        theme.element_bg
    } else {
        theme.panel_bg
    };
    Container::new(Label::new(label.to_string()).style(move |s| {
        s.font_size(12.0)
            .color(if is_active { theme.text } else { theme.text_muted })
            .text_ellipsis()
    }))
    .style(move |s| {
        s.height(22.0)
            .padding_horiz(10.0)
            .items_center()
            .border_radius(6.0)
            .background(background)
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

    WorkspaceTab {
        id,
        name,
        root,
        file_tree,
        git_status,
    }
}
