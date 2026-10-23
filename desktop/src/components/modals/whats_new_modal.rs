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
                    let path = path.ok_or_else(|| {
                        "Unable to locate the Fred TV data directory".to_string()
                    })?;
                    let parent = path.parent().ok_or_else(|| {
                        "Unable to locate the Fred TV data directory".to_string()
                    })?;
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
            h2 { class: "text-2xl font-semibold", "What's new in v{version}" }
            div { class: "mt-4 grid gap-4 text-sm",
                p {
                    "Fred TV is in crucial need of your help. Consider donating any amount (even 1$, it makes a big difference!) to support future development. "
                    a {
                        class: "text-primary underline",
                        href: "https://github.com/Fredolx/open-tv/discussions/69",
                        onclick: move |event| {
                            event.prevent_default();
                            if let Err(error) = open::that("https://github.com/Fredolx/open-tv/discussions/69") {
                                report_error(app, error.to_string());
                            }
                        },
                        "Click here for more information."
                    }
                }
                p {
                    "If you'd like to follow updates about Fred TV and related projects, I made a bluesky! "
                    a {
                        class: "text-primary underline",
                        href: "https://bsky.app/profile/fredoldev.bsky.social",
                        onclick: move |event| {
                            event.prevent_default();
                            if let Err(error) = open::that("https://bsky.app/profile/fredoldev.bsky.social") {
                                report_error(app, error.to_string());
                            }
                        },
                        "You can follow me here."
                    }
                }
                h3 { class: "font-semibold", "Changes" }
                ul { class: "list-disc space-y-1 pl-5",
                    li { "Fixed catchup download/play erroring out" }
                    li { "Fixed creating new custom channels erroring out" }
                    li { "Fixed favorites not showing up in hidden view" }
                    li { "Improved linux support" }
                    li { "Fixed tag search icon" }
                    li { "Fixed hidden categories not persisting on refresh" }
                    li { "Added bulk operations in search bar" }
                }
                h3 { class: "font-semibold", "Upcoming improvements" }
                ul { class: "list-disc space-y-1 pl-5",
                    li { "Recording scheduling" }
                    li { "Your idea here!" }
                }
            }
            div { class: "mt-6 flex justify-end gap-3",
                button {
                    class: "rounded border border-border px-4 py-2",
                    onclick: move |event| {
                        event.prevent_default();
                        if let Err(error) = open::that("https://github.com/Fredolx/open-tv/discussions/69") {
                            report_error(app, error.to_string());
                        }
                    },
                    "Donate"
                }
                button {
                    class: "rounded bg-primary px-4 py-2 text-primary-foreground",
                    onclick: move |_| open.set(false),
                    "Don't show again"
                }
            }
        }
    }
}
