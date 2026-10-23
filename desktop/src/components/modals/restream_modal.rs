use std::sync::Arc;

use dioxus::prelude::*;
use nyx_tv_lib::types::{AppState, Channel, NetworkInfo};
use tokio::sync::Mutex;

use crate::{
    backend::{blocking, notify, report_error},
    components::modals::dialog::Dialog,
    state::{AppData, ToastKind},
};

#[component]
pub fn RestreamModal(open: Signal<bool>, channel: Channel) -> Element {
    let app = use_context::<Signal<AppData>>();
    let state = use_context::<Arc<Mutex<AppState>>>();
    let mut network = use_signal(|| None::<NetworkInfo>);
    let mut selected_ip = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut started = use_signal(|| false);
    let mut watching = use_signal(|| false);
    let start_state = state.clone();
    let stop_state = state.clone();
    let watch_state = state.clone();

    use_effect(move || {
        if open() && network().is_none() {
            spawn(async move {
                match nyx_tv_lib::get_network_info().await {
                    Ok(info) => {
                        selected_ip.set(info.local_ips.first().cloned().unwrap_or_default());
                        network.set(Some(info));
                    }
                    Err(message) => report_error(app, message),
                }
            });
        }
    });

    rsx! {
        Dialog { open,
            h2 { class: "text-xl font-semibold", "Re-stream · {channel.name}" }
            if let Some(info) = network() {
                p { class: "mt-3 text-sm text-muted-foreground", "Port {info.port} · WAN address {info.wan_ip}" }
                label { class: "mt-4 grid gap-1 text-sm", "Local interface"
                    select {
                        class: "rounded border border-border bg-background px-3 py-2",
                        value: "{selected_ip()}",
                        onchange: move |event| selected_ip.set(event.value()),
                        for ip in info.local_ips.iter() {
                            option { value: "{ip}", "{ip}" }
                        }
                    }
                }
                div { class: "mt-5 flex flex-wrap gap-2",
                    if !started() {
                        button {
                            class: "rounded bg-primary px-4 py-2 text-sm text-primary-foreground",
                            disabled: loading(),
                            onclick: move |_| {
                                let state = start_state.clone();
                                let channel = channel.clone();
                                let port = info.port;
                                let app = app;
                                let mut loading = loading;
                                let mut started = started;
                                spawn(async move {
                                    loading.set(true);
                                    let (sender, receiver) = tokio::sync::oneshot::channel();
                                    let task = tokio::spawn(async move {
                                        nyx_tv_lib::start_restream(port, &state, sender, channel).await
                                    });
                                    match receiver.await {
                                        Ok(()) => {
                                            started.set(true);
                                            loading.set(false);
                                            let mut started = started;
                                            spawn(async move {
                                                match task.await {
                                                    Ok(Ok(())) => {}
                                                    Ok(Err(message)) => {
                                                        report_error(app, message);
                                                        started.set(false);
                                                    }
                                                    Err(error) => {
                                                        report_error(
                                                            app,
                                                            format!("Re-stream task failed: {error}"),
                                                        );
                                                        started.set(false);
                                                    }
                                                }
                                            });
                                        }
                                        Err(receiver_error) => {
                                            match task.await {
                                                Ok(Err(message)) => report_error(app, message),
                                                Ok(Ok(())) => report_error(
                                                    app,
                                                    format!("Re-stream did not start: {receiver_error}"),
                                                ),
                                                Err(error) => report_error(
                                                    app,
                                                    format!("Re-stream did not start: {error}"),
                                                ),
                                            }
                                            loading.set(false);
                                            started.set(false);
                                        }
                                    }
                                });
                            },
                            if loading() { "Starting…" } else { "Start" }
                        }
                    } else {
                        button {
                            class: "rounded bg-destructive px-4 py-2 text-sm text-white",
                            disabled: loading(),
                            onclick: move |_| {
                                let state = stop_state.clone();
                                spawn(async move {
                                    loading.set(true);
                                    match nyx_tv_lib::stop_restream(&state).await {
                                        Ok(()) => started.set(false),
                                        Err(message) => report_error(app, message),
                                    }
                                    loading.set(false);
                                });
                            },
                            "Stop"
                        }
                        button {
                            class: "rounded border border-border px-4 py-2 text-sm",
                            disabled: watching(),
                            onclick: move |_| {
                                let state = watch_state.clone();
                                let port = info.port;
                                spawn(async move {
                                    watching.set(true);
                                    if let Err(message) = nyx_tv_lib::watch_self(port, &state).await {
                                        report_error(app, message);
                                    }
                                    watching.set(false);
                                });
                            },
                            if watching() { "Opening…" } else { "Watch here" }
                        }
                        button {
                            class: "rounded border border-border px-4 py-2 text-sm",
                            onclick: move |_| {
                                let channel = channel.clone();
                                let ip = selected_ip();
                                let app = app;
                                spawn(async move {
                                    let filename = format!("{}_rst.otv", channel.name);
                                    if let Some(file) = rfd::AsyncFileDialog::new()
                                        .add_filter("Nyx TV channel", &["otv"])
                                        .set_file_name(&filename)
                                        .save_file()
                                        .await
                                    {
                                        let path = file.path().to_string_lossy().to_string();
                                        let export_path = path.clone();
                                        match blocking(move || nyx_tv_lib::share_restream(ip, channel, export_path)).await {
                                            Ok(()) => notify(app, ToastKind::Success, format!("Successfully exported re-stream to {path}")),
                                            Err(message) => report_error(app, message),
                                        }
                                    }
                                });
                            },
                            "Share"
                        }
                    }
                }
            } else {
                p { class: "mt-4 text-sm text-muted-foreground", "Loading network information…" }
            }
            div { class: "mt-5 flex justify-end",
                button { class: "rounded border border-border px-4 py-2", onclick: move |_| open.set(false), "Close" }
            }
        }
    }
}
