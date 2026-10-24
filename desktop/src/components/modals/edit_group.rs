use dioxus::prelude::*;
use nyx_tv_lib::types::Group;

use crate::{
    backend::{blocking, notify, report_error},
    components::modals::dialog::Dialog,
    state::{AppData, ToastKind},
};

#[component]
pub fn EditGroupModal(open: Signal<bool>, source_id: i64, group: Option<Group>) -> Element {
    let app = use_context::<Signal<AppData>>();
    let mut name = use_signal(|| {
        group
            .as_ref()
            .map(|group| group.name.clone())
            .unwrap_or_default()
    });
    let mut image = use_signal(|| {
        group
            .as_ref()
            .and_then(|group| group.image.clone())
            .unwrap_or_default()
    });
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);

    rsx! {
        Dialog { open,
            h2 { class: "text-xl font-semibold", if group.is_some() { "Edit group" } else { "Add group" } }
            label { class: "mt-4 grid gap-1 text-sm", "Name"
                input { class: "rounded border border-border bg-background px-3 py-2", value: "{name()}", oninput: move |event| name.set(event.value()) }
            }
            label { class: "mt-3 grid gap-1 text-sm", "Image URL"
                input { class: "rounded border border-border bg-background px-3 py-2", value: "{image()}", oninput: move |event| image.set(event.value()) }
            }
            if let Some(message) = error() {
                p { class: "mt-2 text-sm text-destructive", "{message}" }
            }
            footer { class: "mt-5 flex justify-end gap-2",
                button { class: "rounded border border-border px-4 py-2", onclick: move |_| open.set(false), "Cancel" }
                button {
                    class: "rounded bg-primary px-4 py-2 text-primary-foreground",
                    disabled: loading(),
                    onclick: move |_| {
                        let group = group.clone();
                        let name_value = name().trim().to_string();
                        let image_value = image().trim().to_string();
                        if name_value.is_empty() {
                            error.set(Some("Enter a group name.".to_string()));
                            return;
                        }
                        loading.set(true);
                        spawn(async move {
                            let original = group.as_ref().map(|group| group.name.as_str()).unwrap_or_default();
                            let exists = if original == name_value {
                                Ok(false)
                            } else {
                                let query = name_value.clone();
                                blocking(move || nyx_tv_lib::group_exists(query, source_id)).await
                            };
                            match exists {
                                Ok(true) => error.set(Some("A group with this name already exists.".to_string())),
                                Err(message) => report_error(app, message),
                                Ok(false) => {
                                    let new_group = Group {
                                        id: group.as_ref().and_then(|group| group.id),
                                        name: name_value,
                                        image: (!image_value.is_empty()).then_some(image_value),
                                        source_id: Some(source_id),
                                        hidden: group.as_ref().and_then(|group| group.hidden),
                                    };
                                    let result = if group.is_some() {
                                        blocking(move || nyx_tv_lib::edit_custom_group(new_group)).await
                                    } else {
                                        blocking(move || nyx_tv_lib::add_custom_group(new_group)).await
                                    };
                                    match result {
                                        Ok(()) => {
                                            notify(app, ToastKind::Success, if group.is_some() { "Successfully updated category" } else { "Successfully added category" });
                                            open.set(false);
                                        }
                                        Err(message) => report_error(app, message),
                                    }
                                }
                            }
                            loading.set(false);
                        });
                    },
                    if loading() { "Saving…" } else { "Save" }
                }
            }
        }
    }
}
