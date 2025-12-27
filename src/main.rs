mod app;
mod components;
mod model;
mod services;
mod theme;

fn main() {
    floem::launch(app::app_view);
}
