use dioxus::prelude::*;

use crate::{
    components::modals::dialog::Dialog,
    state::AppData,
};

#[component]
pub fn ErrorModal() -> Element {
    let mut app = use_context::<Signal<AppData>>();
    let mut open = use_signal(|| false);
    let detail = app.read().error_modal.clone();

    use_effect(move || open.set(app.read().error_modal.is_some()));

    rsx! {
        Dialog {
            open,
            if let Some(message) = detail {
                h2 { class: "text-lg font-semibold text-destructive", "Something went wrong" }
                p { class: "mt-3 whitespace-pre-wrap text-sm text-muted-foreground", "{message}" }
                div { class: "mt-6 flex justify-end",
                    button {
                        class: "rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:bg-primary/90",
                        onclick: move |_| app.write().error_modal = None,
                        "Close"
                    }
                }
            }
        }
    }
}
