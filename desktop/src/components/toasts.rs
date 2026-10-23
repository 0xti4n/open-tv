use dioxus::prelude::*;

use crate::state::{AppData, ToastKind};

#[component]
pub fn Toasts() -> Element {
    let mut app = use_context::<Signal<AppData>>();

    rsx! {
        div { class: "fixed bottom-4 right-4 z-[100] flex w-full max-w-sm flex-col gap-2",
            for toast in app.read().toasts.iter() {
                {
                    let toast = toast.clone();
                    let detail = toast.error.clone();
                    let class = match toast.kind {
                        ToastKind::Error => "border-destructive/50 bg-destructive text-white",
                        ToastKind::Info => "border-primary/50 bg-card text-foreground",
                        ToastKind::Success => "border-emerald-500/50 bg-card text-foreground",
                    };
                    rsx! {
                        button {
                            key: "{toast.id}",
                            class: "rounded-lg border px-4 py-3 text-left text-sm shadow-lg {class}",
                            onclick: move |_| {
                                if let Some(detail) = detail.clone() {
                                    app.write().error_modal = Some(detail);
                                }
                            },
                            "{toast.message}"
                        }
                    }
                }
            }
        }
    }
}
