mod app;
mod backend;
mod components;
mod state;
mod views;

pub use components::ui;

fn main() {
    let instance = single_instance::SingleInstance::new("dev.nyxtv.nyx-tv")
        .expect("failed to acquire single-instance lock");
    if !instance.is_single() {
        return;
    }

    if let Err(error) = nyx_tv_lib::paths::migrate_legacy_data_dir() {
        eprintln!("Failed to migrate legacy Nyx TV data: {error}");
    }

    let icon = dioxus::desktop::icon_from_memory::<dioxus::desktop::tao::window::Icon>(
        include_bytes!("../icons/128x128.png"),
    )
    .expect("failed to load application icon");
    let window = dioxus::desktop::WindowBuilder::new()
        .with_title(format!("Nyx TV (v{})", env!("CARGO_PKG_VERSION")))
        .with_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0))
        .with_resizable(true)
        .with_window_icon(Some(icon));

    let config = dioxus::desktop::Config::new()
        .with_window(window)
        .with_disable_context_menu(true)
        .with_tray_icon_show_window_on_click(true);
    #[cfg(not(target_os = "macos"))]
    let config = config.with_menu(None);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app::app);
}
