use dioxus::prelude::*;

use crate::{
    backend::{notify, report_error},
    state::{ActiveDownload, AppData, ToastKind},
};

#[component]
pub fn DownloadManager() -> Element {
    let app = use_context::<Signal<AppData>>();
    let mut minimized = use_signal(|| false);
    let downloads = app.read().downloads.clone();
    if downloads.is_empty() {
        return rsx! {};
    }

    rsx! {
        section { class: "fixed bottom-4 left-4 z-[80] w-80 rounded-lg border border-border bg-card p-3 shadow-xl",
            header { class: "flex items-center justify-between",
                h2 { class: "font-semibold", "Downloads ({downloads.len()})" }
                button { class: "text-sm text-muted-foreground", onclick: move |_| minimized.set(!minimized()), if minimized() { "Expand" } else { "Minimize" } }
            }
            if !minimized() {
                for download in downloads {
                    DownloadRow { key: "{download.id}", download }
                }
            }
        }
    }
}

#[component]
fn DownloadRow(download: ActiveDownload) -> Element {
    let mut app = use_context::<Signal<AppData>>();
    let state = use_context::<std::sync::Arc<tokio::sync::Mutex<open_tv_lib::types::AppState>>>();
    let id = download.id.clone();
    let source_id = download.source_id;
    let name = download.name.clone();

    rsx! {
        div { class: "mt-3",
            div { class: "flex items-center justify-between gap-2 text-sm",
                span { class: "truncate", "{name}" }
                button {
                    class: "text-destructive",
                    title: "Cancel download",
                    onclick: move |_| {
                        let state = state.clone();
                        let id = id.clone();
                        spawn(async move {
                            match open_tv_lib::abort_download(&state, source_id, id.clone()).await {
                                Ok(()) => {
                                    app.write().downloads.retain(|item| item.id != id);
                                    notify(app, ToastKind::Info, "Download cancelled");
                                }
                                Err(message) => report_error(app, message),
                            }
                        });
                    },
                    "Cancel"
                }
            }
            progress { class: "mt-2 h-2 w-full accent-primary", max: "100", value: "{download.progress}" }
            p { class: "mt-1 text-right text-xs text-muted-foreground", "{download.progress:.0}%" }
        }
    }
}
