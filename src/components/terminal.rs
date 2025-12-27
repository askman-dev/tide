use crate::components::atoms::meta_text;
use crate::theme::UiTheme;
use floem::prelude::*;

pub fn terminal_placeholder(theme: UiTheme, workspace: String) -> impl IntoView {
    v_stack((
        Label::new("Terminal").style(move |s| {
            s.font_size(12.0)
                .font_bold()
                .color(theme.text_muted)
        }),
        meta_text(format!("Workspace: {workspace}"), theme),
        Container::new(Label::new(
            "Terminal placeholder (Alacritty or Lapce terminal next).",
        ))
        .style(move |s| {
            s.width_full()
                .height_full()
                .padding(12.0)
                .background(theme.panel_bg)
                .border(1.0)
                .border_color(theme.border_subtle)
                .color(theme.text_soft)
        }),
    ))
    .style(|s| s.width_full().height_full().row_gap(8.0))
}
