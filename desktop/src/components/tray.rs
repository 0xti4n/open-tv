use dioxus::prelude::*;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::state::AppData;

#[component]
pub fn TrayIntegration() -> Element {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let app = use_context::<Signal<AppData>>();

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        let window = dioxus::desktop::use_window();
        let mut initialized = use_signal(|| false);

        use_effect(move || {
            let enabled = app
                .read()
                .settings
                .as_ref()
                .and_then(|settings| settings.enable_tray_icon)
                .unwrap_or(true);
            if enabled {
                window.set_close_behavior(dioxus::desktop::WindowCloseBehaviour::WindowHides);
                if !initialized() {
                    use dioxus::desktop::muda::{Menu, MenuItem};
                    let menu = Menu::new();
                    let show = MenuItem::with_id("show", "Show", true, None);
                    let quit = MenuItem::with_id("quit", "Quit", true, None);
                    menu.append_items(&[&show, &quit]).expect("failed to create tray menu");
                    dioxus::desktop::trayicon::init_tray_icon(menu, None);
                    initialized.set(true);
                }
            } else {
                window.set_close_behavior(dioxus::desktop::WindowCloseBehaviour::WindowCloses);
            }
        });

        dioxus::desktop::use_tray_menu_event_handler(move |event| {
            match event.id().as_ref() {
                "show" => {
                    window.set_minimized(false);
                    window.set_visible(true);
                    window.set_focus();
                }
                "quit" => {
                    window.set_close_behavior(dioxus::desktop::WindowCloseBehaviour::WindowCloses);
                    window.close();
                }
                _ => {}
            }
        });
    }

    rsx! {}
}
