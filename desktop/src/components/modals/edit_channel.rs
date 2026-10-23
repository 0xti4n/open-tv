use dioxus::prelude::*;
use open_tv_lib::{
    media_type,
    types::{Channel, CustomChannel, IdName},
};

use crate::{
    backend::{blocking, notify, report_error},
    components::modals::dialog::Dialog,
    state::{AppData, ToastKind},
};

#[component]
pub fn EditChannelModal(
    open: Signal<bool>,
    source_id: i64,
    channel: Option<Channel>,
) -> Element {
    let app = use_context::<Signal<AppData>>();
    let mut name = use_signal(|| channel.as_ref().map(|channel| channel.name.clone()).unwrap_or_default());
    let mut url = use_signal(|| channel.as_ref().and_then(|channel| channel.url.clone()).unwrap_or_default());
    let mut selected_media_type = use_signal(|| {
        channel
            .as_ref()
            .map(|channel| channel.media_type)
            .unwrap_or(media_type::LIVESTREAM)
    });
    let mut image = use_signal(|| channel.as_ref().and_then(|channel| channel.image.clone()).unwrap_or_default());
    let mut query = use_signal(String::new);
    let mut groups = use_signal(Vec::<IdName>::new);
    let mut selected_group = use_signal(|| channel.as_ref().and_then(|channel| channel.group_id));
    let mut loading = use_signal(|| false);

    rsx! {
        Dialog { open,
            h2 { class: "text-xl font-semibold", if channel.is_some() { "Edit custom channel" } else { "Add custom channel" } }
            div { class: "mt-4 grid gap-3",
                label { class: "grid gap-1 text-sm", "Name"
                    input { class: "rounded border border-border bg-background px-3 py-2", value: "{name()}", oninput: move |event| name.set(event.value()) }
                }
                label { class: "grid gap-1 text-sm", "Stream URL"
                    input { class: "rounded border border-border bg-background px-3 py-2", value: "{url()}", oninput: move |event| url.set(event.value()) }
                }
                label { class: "grid gap-1 text-sm", "Media type"
                    select {
                        class: "rounded border border-border bg-background px-3 py-2",
                        value: "{selected_media_type()}",
                        onchange: move |event| {
                            if let Ok(value) = event.value().parse::<u8>() {
                                selected_media_type.set(value);
                            }
                        },
                        option { value: "{media_type::LIVESTREAM}", "Livestream" }
                        option { value: "{media_type::MOVIE}", "Movie" }
                    }
                }
                label { class: "grid gap-1 text-sm", "Logo URL"
                    input { class: "rounded border border-border bg-background px-3 py-2", value: "{image()}", oninput: move |event| image.set(event.value()) }
                }
                label { class: "grid gap-1 text-sm", "Group"
                    input {
                        class: "rounded border border-border bg-background px-3 py-2",
                        value: "{query()}",
                        placeholder: "Search groups",
                        oninput: move |event| {
                            let value = event.value();
                            query.set(value.clone());
                            let source_id = source_id;
                            spawn(async move {
                                match blocking(move || open_tv_lib::group_auto_complete(Some(value), source_id)).await {
                                    Ok(results) => groups.set(results),
                                    Err(message) => report_error(app, message),
                                }
                            });
                        }
                    }
                    div { class: "max-h-32 overflow-y-auto",
                        for group in groups() {
                            button {
                                key: "{group.id}",
                                class: "block w-full rounded px-3 py-1 text-left text-sm hover:bg-accent",
                                onclick: move |_| { selected_group.set(Some(group.id)); query.set(group.name.clone()); groups.set(Vec::new()); },
                                "{group.name}"
                            }
                        }
                    }
                }
            }
            footer { class: "mt-5 flex justify-end gap-2",
                button { class: "rounded border border-border px-4 py-2", onclick: move |_| open.set(false), "Cancel" }
                button {
                    class: "rounded bg-primary px-4 py-2 text-primary-foreground",
                    disabled: loading(),
                    onclick: move |_| {
                        loading.set(true);
                        let mut data = channel.clone().unwrap_or(Channel {
                            id: None,
                            name: String::new(),
                            url: None,
                            group: None,
                            image: None,
                            media_type: media_type::LIVESTREAM,
                            source_id: Some(source_id),
                            series_id: None,
                            group_id: None,
                            favorite: false,
                            stream_id: None,
                            tv_archive: None,
                            season_id: None,
                            episode_num: None,
                            hidden: Some(false),
                        });
                        data.name = name().trim().to_string();
                        data.url = Some(url().trim().to_string());
                        data.image = Some(image().trim().to_string());
                        data.media_type = selected_media_type();
                        data.group_id = selected_group();
                        let existing = channel.is_some();
                        spawn(async move {
                            let custom = CustomChannel { data, headers: None };
                            let result = if existing {
                                blocking(move || open_tv_lib::edit_custom_channel(custom)).await
                            } else {
                                blocking(move || open_tv_lib::add_custom_channel(custom)).await
                            };
                            match result {
                                Ok(()) => {
                                    notify(app, ToastKind::Success, if existing { "Successfully updated channel" } else { "Successfully added channel" });
                                    open.set(false);
                                }
                                Err(message) => report_error(app, message),
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
