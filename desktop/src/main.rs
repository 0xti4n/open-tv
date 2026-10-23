mod app;
mod backend;
mod components;
mod state;
mod views;

pub use components::ui;

fn main() {
    let instance = single_instance::SingleInstance::new("dev.fredol.open-tv")
        .expect("failed to acquire single-instance lock");
    if !instance.is_single() {
        return;
    }
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_disable_context_menu(true)
                .with_tray_icon_show_window_on_click(true),
        )
        .launch(app::app);
}
