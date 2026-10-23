use dioxus::prelude::*;
use nyx_tv_lib::{source_type, types::Settings as AppSettings};

use crate::{
    app::Route,
    backend::{blocking, notify, report_error},
    components::{
        modals::{confirm_delete::ConfirmDeleteModal, dialog::Dialog},
        source_tile::SourceTile,
        ui::button::{Button, ButtonSize, ButtonVariant},
    },
    state::{AppData, ToastKind},
};

fn defaults() -> AppSettings {
    AppSettings {
        recording_path: None,
        mpv_params: None,
        use_stream_caching: Some(true),
        default_view: Some(0),
        volume: Some(100),
        refresh_on_start: Some(false),
        restream_port: Some(3000),
        enable_tray_icon: Some(true),
        zoom: Some(100),
        default_sort: Some(2),
        enable_hwdec: Some(true),
        always_ask_save: Some(false),
        enable_gpu: Some(false),
    }
}

async fn save_settings(
    app: Signal<AppData>,
    settings: AppSettings,
    window: Signal<dioxus::desktop::DesktopContext>,
    mut zoom: Signal<f64>,
) {
    let zoom_level = settings.zoom.unwrap_or(100) as f64 / 100.0;
    match blocking(move || nyx_tv_lib::update_settings(settings)).await {
        Ok(()) => {
            zoom.set(zoom_level);
            window.read().set_zoom_level(zoom_level);
        }
        Err(message) => report_error(app, message),
    }
}

#[component]
pub fn Settings() -> Element {
    let mut app = use_context::<Signal<AppData>>();
    let zoom = use_context::<Signal<f64>>();
    let desktop_window = dioxus::desktop::use_window();
    let window = use_signal(|| desktop_window.clone());
    let navigator = use_navigator();
    let mut settings = use_signal(|| app.read().settings.clone().unwrap_or_else(defaults));
    let mut loading = use_signal(|| false);
    let mut confirm_delete = use_signal(|| false);
    let mut expiries_loaded = use_signal(|| false);

    use_effect(move || {
        if let Some(updated) = app.read().settings.clone() {
            settings.set(updated);
        }
    });

    use_effect(move || {
        let has_xtream = app
            .read()
            .sources
            .iter()
            .any(|source| source.source_type == source_type::XTREAM);
        if has_xtream && !expiries_loaded() {
            expiries_loaded.set(true);
            spawn(async move {
                match nyx_tv_lib::get_all_expiries().await {
                    Ok(expiries) => app.write().expiries = expiries,
                    Err(message) => report_error(app, message),
                }
            });
        }
    });

    let update = move |settings: AppSettings| {
        let settings = settings.clone();
        spawn(async move { save_settings(app, settings, window, zoom).await });
    };

    rsx! {
        main { class: "mx-auto min-h-screen max-w-6xl px-5 py-6",
            header { class: "mb-6 flex items-center justify-between",
                h1 { class: "text-2xl font-semibold", "Settings" }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| { navigator.push(Route::Home {}); },
                    "← Back to channels"
                }
            }
            section { class: "rounded-xl border border-border bg-card p-5",
                h2 { class: "mb-4 text-lg font-semibold", "Playback and interface" }
                div { class: "grid gap-4 md:grid-cols-2",
                    label { class: "grid gap-1 text-sm",
                        "Recording path"
                        div { class: "flex gap-2",
                            input { class: "min-w-0 flex-1 rounded border border-border bg-background px-3 py-2", readonly: true, value: "{settings().recording_path.as_deref().unwrap_or(\"~/Home/Videos\")}" }
                            button {
                                class: "rounded border border-border px-3 py-2 hover:bg-accent",
                                onclick: move |_| {
                                    spawn(async move {
                                        if let Some(folder) = rfd::AsyncFileDialog::new().pick_folder().await {
                                            let mut current = app.read().settings.clone().unwrap_or_else(defaults);
                                            current.recording_path = Some(folder.path().to_string_lossy().to_string());
                                            app.write().settings = Some(current.clone());
                                            save_settings(app, current, window, zoom).await;
                                        }
                                    });
                                },
                                "Change"
                            }
                        }
                    }
                    label { class: "grid gap-1 text-sm",
                        "Global MPV parameters"
                        textarea {
                            class: "min-h-20 rounded border border-border bg-background px-3 py-2",
                            placeholder: "MPV parameters such as --hwdec auto",
                            value: "{settings().mpv_params.clone().unwrap_or_default()}",
                            oninput: move |event| {
                                let mut next = settings();
                                next.mpv_params = Some(event.value());
                                settings.set(next.clone());
                                app.write().settings = Some(next.clone());
                                update(next);
                            }
                        }
                    }
                    label { class: "grid gap-1 text-sm", "Default view"
                        select {
                            class: "rounded border border-border bg-background px-3 py-2",
                            value: "{settings().default_view.unwrap_or(0)}",
                            onchange: move |event| {
                                let mut next = settings();
                                next.default_view = event.value().parse().ok();
                                settings.set(next.clone());
                                app.write().settings = Some(next.clone());
                                update(next);
                            },
                            option { value: "0", "All" }
                            option { value: "1", "Categories" }
                            option { value: "3", "Favorites" }
                        }
                    }
                    label { class: "grid gap-1 text-sm", "Default sort"
                        select {
                            class: "rounded border border-border bg-background px-3 py-2",
                            value: "{settings().default_sort.unwrap_or(2)}",
                            onchange: move |event| {
                                let mut next = settings();
                                next.default_sort = event.value().parse().ok();
                                settings.set(next.clone());
                                app.write().settings = Some(next.clone());
                                update(next);
                            },
                            option { value: "0", "Alphabetically asc" }
                            option { value: "1", "Alphabetically desc" }
                            option { value: "2", "Provider" }
                        }
                    }
                    label { class: "grid gap-1 text-sm", "Volume on start: {settings().volume.unwrap_or(100)}"
                        input {
                            r#type: "range",
                            min: "0", max: "100", step: "1",
                            value: "{settings().volume.unwrap_or(100)}",
                            oninput: move |event| {
                                let mut next = settings();
                                next.volume = event.value().parse().ok();
                                settings.set(next.clone());
                                app.write().settings = Some(next.clone());
                                update(next);
                            }
                        }
                    }
                    label { class: "grid gap-1 text-sm", "Re-stream port"
                        input {
                            r#type: "number", min: "1", max: "65535",
                            class: "rounded border border-border bg-background px-3 py-2",
                            value: "{settings().restream_port.unwrap_or(3000)}",
                            oninput: move |event| {
                                let mut next = settings();
                                next.restream_port = event.value().parse().ok();
                                settings.set(next.clone());
                                app.write().settings = Some(next.clone());
                                update(next);
                            }
                        }
                    }
                    label { class: "grid gap-1 text-sm", "UI zoom (%)"
                        input {
                            r#type: "number", min: "10", max: "1000", step: "10",
                            class: "rounded border border-border bg-background px-3 py-2",
                            value: "{settings().zoom.unwrap_or(100)}",
                            oninput: move |event| {
                                let mut next = settings();
                                next.zoom = event.value().parse().ok();
                                settings.set(next.clone());
                                app.write().settings = Some(next.clone());
                                update(next);
                            }
                        }
                    }
                    for (label, value, key) in [
                        ("Use stream caching", settings().use_stream_caching.unwrap_or(true), 0_u8),
                        ("Enable hardware acceleration", settings().enable_hwdec.unwrap_or(true), 1),
                        ("Use modern hardware optimizations", settings().enable_gpu.unwrap_or(false), 2),
                        ("Always ask where to save downloads", settings().always_ask_save.unwrap_or(false), 3),
                        ("Refresh all sources on start", settings().refresh_on_start.unwrap_or(false), 4),
                        ("Enable tray icon", settings().enable_tray_icon.unwrap_or(true), 5),
                    ] {
                        label { class: "flex items-center justify-between gap-3 rounded border border-border px-3 py-2 text-sm",
                            "{label}"
                            input {
                                r#type: "checkbox",
                                checked: value,
                                onchange: move |event| {
                                    let mut next = settings();
                                    let checked = event.value() == "true";
                                    match key {
                                        0 => next.use_stream_caching = Some(checked),
                                        1 => next.enable_hwdec = Some(checked),
                                        2 => next.enable_gpu = Some(checked),
                                        3 => next.always_ask_save = Some(checked),
                                        4 => next.refresh_on_start = Some(checked),
                                        _ => next.enable_tray_icon = Some(checked),
                                    }
                                    settings.set(next.clone());
                                    app.write().settings = Some(next.clone());
                                    update(next);
                                }
                            }
                        }
                    }
                }
            }

            section { class: "mt-6",
                h2 { class: "mb-4 text-lg font-semibold", "Sources" }
                for source in app.read().sources.iter() {
                    {
                        let source = source.clone();
                        let expiry = source.id.and_then(|id| app.read().expiries.get(&id).copied());
                        rsx! { SourceTile { key: "{source.id:?}", source, expiry } }
                    }
                }
                div { class: "mt-5 flex flex-wrap gap-3",
                    button {
                        class: "rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground",
                        onclick: move |_| { navigator.push(Route::Setup {}); },
                        "Add source"
                    }
                    button {
                        class: "rounded-md border border-border px-4 py-2 text-sm hover:bg-accent",
                        disabled: loading(),
                        onclick: move |_| {
                            spawn(async move {
                                loading.set(true);
                                match nyx_tv_lib::refresh_all().await {
                                    Ok(()) => notify(app, ToastKind::Success, "Successfully updated all sources"),
                                    Err(message) => report_error(app, message),
                                }
                                match blocking(nyx_tv_lib::get_sources).await {
                                    Ok(sources) => app.write().sources = sources,
                                    Err(message) => report_error(app, message),
                                }
                                loading.set(false);
                            });
                        },
                        if loading() { "Refreshing…" } else { "Refresh all" }
                    }
                    button {
                        class: "rounded-md border border-border px-4 py-2 text-sm hover:bg-accent",
                        onclick: move |_| { spawn(async move {
                            match blocking(nyx_tv_lib::clear_history).await {
                                Ok(()) => notify(app, ToastKind::Success, "History cleared successfully"),
                                Err(message) => report_error(app, message),
                            }
                        }); },
                        "Clear history"
                    }
                    button {
                        class: "rounded-md bg-destructive px-4 py-2 text-sm text-white",
                        onclick: move |_| confirm_delete.set(true),
                        "Delete everything"
                    }
                }
            }
            Dialog { open: confirm_delete,
                ConfirmDeleteModal { open: confirm_delete }
            }
        }
    }
}
