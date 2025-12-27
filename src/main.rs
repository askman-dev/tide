mod app;
mod components;
mod logging;
mod model;
mod services;
mod theme;

fn main() {
    logging::init();
    floem::launch(app::app_view);
}
