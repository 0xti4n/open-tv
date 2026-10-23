use dioxus::prelude::*;

use crate::{
    backend::{blocking, notify, report_error},
    state::{AppData, ToastKind},
};

#[component]
pub fn ConfirmDeleteModal(open: Signal<bool>) -> Element {
    let app = use_context::<Signal<AppData>>();
    let mut loading = use_signal(|| false);

    rsx! {
        h2 { class: "text-lg font-semibold text-destructive", "Delete everything?" }
        p { class: "mt-3 text-sm text-muted-foreground",
            "This schedules the local database for deletion. Use this only if an upgrade did not fix a problem."
        }
        div { class: "mt-6 flex justify-end gap-2",
            button {
                class: "rounded-md border border-border px-4 py-2 text-sm",
                onclick: move |_| open.set(false),
                "Cancel"
            }
            button {
                class: "rounded-md bg-destructive px-4 py-2 text-sm text-white",
                disabled: loading(),
                onclick: move |_| { spawn(async move {
                    loading.set(true);
                    match blocking(open_tv_lib::delete_database).await {
                        Ok(()) => {
                            notify(app, ToastKind::Success, "Database deletion scheduled");
                            open.set(false);
                        }
                        Err(message) => report_error(app, message),
                    }
                    loading.set(false);
                }); },
                if loading() { "Deleting…" } else { "Delete everything" }
            }
        }
    }
}
