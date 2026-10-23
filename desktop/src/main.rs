mod app;
mod components;

pub use components::ui;

fn main() {
    dioxus::launch(app::app);
}
