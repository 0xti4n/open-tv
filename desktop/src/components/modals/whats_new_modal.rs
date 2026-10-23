use dioxus::prelude::*;

use crate::components::modals::dialog::Dialog;

#[component]
pub fn WhatsNewModal(open: Signal<bool>, version: String) -> Element {
    let mut was_open = use_signal(|| false);
    let storage_version = version.clone();

    use_effect(move || {
        if open() {
            was_open.set(true);
        } else if was_open() {
            let _ = dioxus::document::eval(&format!(
                "localStorage.setItem('lastSeenVersion', {storage_version:?})"
            ));
            was_open.set(false);
        }
    });

    rsx! {
        Dialog { open,
            h2 { class: "text-2xl font-semibold", "What’s new in Fred TV" }
            p { class: "mt-2 text-sm text-muted-foreground", "Version {version}" }
            p { class: "mt-4", "This release includes improvements to playback, source management, and the desktop experience." }
            div { class: "mt-6 flex justify-between gap-3",
                button {
                    class: "rounded border border-border px-4 py-2",
                    onclick: move |_| {
                        let _ = open::that("https://github.com/Fredolx/open-tv/discussions/69");
                    },
                    "Support development"
                }
                button { class: "rounded bg-primary px-4 py-2 text-primary-foreground", onclick: move |_| open.set(false), "Close" }
            }
        }
    }
}
