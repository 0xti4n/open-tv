use std::sync::Arc;

use dioxus::prelude::*;
use nyx_tv_lib::types::{AppState, Channel, EPG, EPGNotify};
use tokio::sync::Mutex;

use crate::{
    backend::{blocking, notify, report_error},
    components::modals::dialog::Dialog,
    state::{AppData, ToastKind},
};

#[component]
pub fn EpgModal(open: Signal<bool>, channel: Channel, programs: Vec<EPG>) -> Element {
    let app = use_context::<Signal<AppData>>();
    let state = use_context::<Arc<Mutex<AppState>>>();
    let mut watched = use_signal(Vec::<String>::new);

    use_effect(move || {
        spawn(async move {
            match blocking(nyx_tv_lib::get_epg_ids).await {
                Ok(ids) => watched.set(ids),
                Err(message) => report_error(app, message),
            }
        });
    });

    rsx! {
        Dialog { open,
            h2 { class: "text-xl font-semibold", "EPG · {channel.name}" }
            div { class: "mt-4 max-h-[60vh] space-y-3 overflow-y-auto",
                for program in programs {
                    {
                        let epg_id = program.epg_id.clone();
                        let epg_item = program.clone();
                        let is_watched = watched().contains(&epg_id);
                        let state_for_item = state.clone();
                        let channel_name = channel.name.clone();
                        rsx! {
                            article { key: "{epg_id}", class: "rounded-lg border border-border p-3",
                                div { class: "flex items-start justify-between gap-3",
                                    div {
                                        h3 { class: "font-medium", "{program.title}" }
                                        p { class: "mt-1 text-xs text-muted-foreground", "{program.start_time} – {program.end_time}" }
                                        p { class: "mt-2 text-sm text-muted-foreground", "{program.description}" }
                                    }
                                    button {
                                        class: "shrink-0 rounded border border-border px-3 py-1 text-xs",
                                        onclick: move |_| {
                                            let id = epg_id.clone();
                                            let item = epg_item.clone();
                                            let state = state_for_item.clone();
                                            let channel_name = channel_name.clone();
                                            spawn(async move {
                                                let result = if is_watched {
                                                    nyx_tv_lib::remove_epg(&state, id.clone()).await
                                                } else {
                                                    nyx_tv_lib::add_epg(&state, EPGNotify {
                                                        epg_id: id.clone(),
                                                        title: item.title,
                                                        start_timestamp: item.start_timestamp,
                                                        channel_name,
                                                    }).await
                                                };
                                                match result {
                                                    Ok(()) => {
                                                        if is_watched {
                                                            watched.with_mut(|ids| ids.retain(|value| value != &id));
                                                            notify(app, ToastKind::Success, "EPG reminder removed");
                                                        } else {
                                                            watched.with_mut(|ids| ids.push(id));
                                                            notify(app, ToastKind::Success, "EPG reminder added");
                                                        }
                                                    }
                                                    Err(message) => report_error(app, message),
                                                }
                                            });
                                        },
                                        if is_watched { "Remove reminder" } else { "Remind me" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "mt-5 flex justify-end",
                button { class: "rounded border border-border px-4 py-2", onclick: move |_| open.set(false), "Close" }
            }
        }
    }
}
