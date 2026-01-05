mod app;
mod components;
mod logging;
mod model;
mod services;
mod theme;

fn main() {
    logging::init();
    let window_config = if cfg!(target_os = "macos") {
        // Debug toggles (no library patch needed):
        // - Default: standard titlebar (no blur during zoom animation)
        // - `TIDE_WINDOW_STYLE=fullsize` uses transparent titlebar + full content view (tabs in titlebar, but has 2s blur on zoom)
        // - `TIDE_WINDOW_STYLE=hidden` uses floem's show_titlebar(false) - WARNING: enables fullsize_content_view internally, causes blur
        // - `TIDE_DISALLOW_HIDPI=1` disables HiDPI backing scale (may reduce resize/GPU pressure).
        // Note: floem's show_titlebar(false) on macOS internally enables fullsize_content_view, causing zoom animation blur.
        let style = std::env::var("TIDE_WINDOW_STYLE").unwrap_or_default();
        let use_fullsize = style.eq_ignore_ascii_case("fullsize");
        let use_hidden = style.eq_ignore_ascii_case("hidden");
        let disallow_hidpi = matches!(
            std::env::var("TIDE_DISALLOW_HIDPI").as_deref(),
            Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
        );

        let mut cfg = floem::window::WindowConfig::default();
        if use_fullsize {
            // Full size content view - can put tabs in titlebar area, but has 2s blur on zoom animation
            // This is due to macOS capturing a snapshot during zoom animation
            logging::log_line("INFO", "mac window style: fullsize (has zoom blur)");
            cfg = cfg
                .show_titlebar(false)
                .with_mac_os_config(|mac| mac.full_size_content_view(true).transparent_title_bar(true));
        } else if use_hidden {
            // Hidden titlebar - WARNING: floem internally enables fullsize_content_view, causes zoom blur
            logging::log_line("INFO", "mac window style: hidden (has zoom blur)");
            cfg = cfg.show_titlebar(false);
        } else {
            // Default: Hidden titlebar (like Lapce) - tabs at top, may have brief blur during zoom
            logging::log_line("INFO", "mac window style: hidden titlebar (default)");
            cfg = cfg.show_titlebar(false);
        }

        if disallow_hidpi {
            logging::log_line("INFO", "mac window: disallow high dpi");
            cfg = cfg.with_mac_os_config(|mac| mac.disallow_high_dpi(true));
        }

        cfg
    } else {
        floem::window::WindowConfig::default()
    };

    floem::Application::new_with_config(floem::AppConfig::default().exit_on_close(true))
        .window(|_| app::app_view(), Some(window_config))
        .run();
}
