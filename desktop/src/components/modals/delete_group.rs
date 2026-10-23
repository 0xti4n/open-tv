use dioxus::prelude::*;
use open_tv_lib::types::{Channel, IdName};

use crate::{
    backend::{blocking, notify, report_error},
    components::modals::dialog::Dialog,
    state::{AppData, ToastKind},
};

#[component]
pub fn DeleteGroupModal(open: Signal<bool>, group: Channel) -> Element {
    let app = use_context::<Signal<AppData>>();
    let mut query = use_signal(String::new);
    let mut groups = use_signal(Vec::<IdName>::new);
    let mut new_id = use_signal(|| None::<i64>);
    let mut loading = use_signal(|| false);
    let source_id = group.source_id.unwrap_or_default();

    rsx! {
        Dialog { open,
            h2 { class: "text-xl font-semibold", "Delete group · {group.name}" }
            p { class: "mt-3 text-sm text-muted-foreground", "Optionally move its channels into another group." }
            input {
                class: "mt-4 w-full rounded border border-border bg-background px-3 py-2",
                placeholder: "Search destination groups",
                value: "{query()}",
                oninput: move |event| {
                    let text = event.value();
                    query.set(text.clone());
                    spawn(async move {
                        match blocking(move || open_tv_lib::group_auto_complete(Some(text), source_id)).await {
                            Ok(results) => groups.set(results.into_iter().filter(|result| Some(result.id) != group.id).collect()),
                            Err(message) => report_error(app, message),
                        }
                    });
                }
            }
            for candidate in groups() {
                button {
                    key: "{candidate.id}",
                    class: "mt-1 block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                    onclick: move |_| { new_id.set(Some(candidate.id)); query.set(candidate.name.clone()); groups.set(Vec::new()); },
                    "{candidate.name}"
                }
            }
            footer { class: "mt-5 flex justify-end gap-2",
                button { class: "rounded border border-border px-4 py-2", onclick: move |_| open.set(false), "Cancel" }
                button {
                    class: "rounded bg-destructive px-4 py-2 text-white",
                    disabled: loading(),
                    onclick: move |_| {
                        let id = group.id.unwrap_or_default();
                        let target = new_id();
                        loading.set(true);
                        spawn(async move {
                            match blocking(move || open_tv_lib::delete_custom_group(id, target, true)).await {
                                Ok(()) => {
                                    notify(app, ToastKind::Success, "Successfully deleted category");
                                    open.set(false);
                                }
                                Err(message) => report_error(app, message),
                            }
                            loading.set(false);
                        });
                    },
                    if loading() { "Deleting…" } else { "Delete group" }
                }
            }
        }
    }
}
