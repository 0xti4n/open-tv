use dioxus::prelude::*;

use crate::{
    backend::{app_data_dir, blocking, report_error},
    components::modals::dialog::Dialog,
    state::AppData,
};

#[component]
pub fn WhatsNewModal(open: Signal<bool>, version: String) -> Element {
    let app = use_context::<Signal<AppData>>();
    let mut was_open = use_signal(|| false);
    let storage_version = version.clone();

    use_effect(move || {
        if open() {
            was_open.set(true);
        } else if was_open() {
            let version = storage_version.clone();
            spawn(async move {
                let path = app_data_dir().map(|directory| directory.join("last_seen_version"));
                let result = blocking(move || {
                    let path = path
                        .ok_or_else(|| "Unable to locate the Nyx TV data directory".to_string())?;
                    let parent = path
                        .parent()
                        .ok_or_else(|| "Unable to locate the Nyx TV data directory".to_string())?;
                    std::fs::create_dir_all(parent)
                        .map_err(|error| format!("Unable to create data directory: {error}"))?;
                    std::fs::write(path, version)
                        .map_err(|error| format!("Unable to save What's New version: {error}"))
                })
                .await;
                if let Err(message) = result {
                    report_error(app, message);
                }
            });
            was_open.set(false);
        }
    });

    rsx! {
        Dialog { open,
            h2 { class: "text-2xl font-semibold", "Nyx TV v{version}" }
            p { class: "mt-4 text-sm", "Rewritten in Rust with Dioxus. Video plays in mpv." }
            div { class: "mt-6 flex justify-end",
                button {
                    class: "rounded bg-primary px-4 py-2 text-primary-foreground",
                    onclick: move |_| open.set(false),
                    "Don't show again"
                }
            }
        }
    }
}
