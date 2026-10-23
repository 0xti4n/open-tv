use dioxus::prelude::*;

use crate::{
    backend::{blocking, notify, report_error},
    components::modals::dialog::Dialog,
    state::{AppData, ToastKind},
};

#[component]
pub fn ImportModal(open: Signal<bool>, source_id: Option<i64>) -> Element {
    let app = use_context::<Signal<AppData>>();
    let mut name_override = use_signal(String::new);
    let mut loading = use_signal(|| false);

    rsx! {
        Dialog { open,
            h2 { class: "text-xl font-semibold", "Import channel or group" }
            label { class: "mt-4 grid gap-1 text-sm", "Optional name override"
                input { class: "rounded border border-border bg-background px-3 py-2", value: "{name_override()}", oninput: move |event| name_override.set(event.value()) }
            }
            div { class: "mt-5 flex justify-end gap-2",
                button { class: "rounded border border-border px-4 py-2", onclick: move |_| open.set(false), "Cancel" }
                button {
                    class: "rounded bg-primary px-4 py-2 text-primary-foreground",
                    disabled: loading(),
                    onclick: move |_| {
                        let override_name = name_override().trim().to_string();
                        let override_name = (!override_name.is_empty()).then_some(override_name);
                        spawn(async move {
                            if let Some(file) = rfd::AsyncFileDialog::new()
                                .add_filter("Nyx TV export", &["otv", "otvg"])
                                .pick_file()
                                .await
                            {
                                loading.set(true);
                                let path = file.path().to_string_lossy().to_string();
                                match blocking(move || nyx_tv_lib::import(path, source_id, override_name)).await {
                                    Ok(()) => {
                                        notify(app, ToastKind::Success, "Successfully imported file");
                                        open.set(false);
                                    }
                                    Err(message) => report_error(app, message),
                                }
                                loading.set(false);
                            }
                        });
                    },
                    if loading() { "Importing…" } else { "Select export file" }
                }
            }
        }
    }
}
