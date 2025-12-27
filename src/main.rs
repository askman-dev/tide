mod app;
mod components;
mod logging;
mod model;
mod services;
mod theme;

fn main() {
    logging::init();
    let window_config = if cfg!(target_os = "macos") {
        floem::window::WindowConfig::default()
            .show_titlebar(false)
            .with_mac_os_config(|mac| {
                mac.transparent_title_bar(true)
                    .full_size_content_view(true)
                    .hide_title(true)
                    .movable_by_window_background(true)
                    .traffic_lights_offset((10.0, 9.5))
            })
    } else {
        floem::window::WindowConfig::default()
    };

    floem::Application::new()
        .window(|_| app::app_view(), Some(window_config))
        .run();
}
