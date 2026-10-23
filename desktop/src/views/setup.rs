use dioxus::prelude::*;
use nyx_tv_lib::{source_type, types::Source};

use crate::{
    app::Route,
    backend::{blocking, notify, report_error},
    components::ui::{
        button::{Button, ButtonVariant},
        input::{Input, InputType},
    },
    state::{AppData, ToastKind},
};

#[component]
pub fn Setup() -> Element {
    let app = use_context::<Signal<AppData>>();
    let navigator = use_navigator();
    let mut source_kind = use_signal(|| 1_u8);
    let mut name = use_signal(String::new);
    let mut url = use_signal(String::new);
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut file_path = use_signal(|| None::<String>);
    let loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let submit = move |_event: MouseEvent| {
        let source_kind = source_kind();
        let name_value = name().trim().to_string();
        let url_value = url().trim().to_string();
        let username_value = username().trim().to_string();
        let password_value = password().trim().to_string();
        let file_path = file_path();
        let mut app = app;
        let mut loading = loading;
        let mut error = error;
        let navigator = navigator;
        spawn(async move {
            if name_value.is_empty() {
                error.set(Some("Enter a source name.".to_string()));
                return;
            }
            let name_for_check = name_value.clone();
            match blocking(move || nyx_tv_lib::source_name_exists(name_for_check)).await {
                Ok(true) => {
                    error.set(Some("A source with this name already exists.".to_string()));
                    return;
                }
                Err(message) => {
                    report_error(app, message.clone());
                    error.set(Some(message));
                    return;
                }
                Ok(false) => {}
            }
            if source_kind == 0 && file_path.is_none() {
                error.set(Some("Choose an M3U file first.".to_string()));
                return;
            }
            if source_kind != 0 && source_kind != source_type::CUSTOM && url_value.is_empty() {
                error.set(Some("Enter a playlist or server URL.".to_string()));
                return;
            }
            if source_kind == 2 && (username_value.is_empty() || password_value.is_empty()) {
                error.set(Some("Enter your Xtream username and password.".to_string()));
                return;
            }
            if source_kind == source_type::CUSTOM {
                loading.set(true);
                error.set(None);
                let custom_name = name_value.clone();
                match blocking(move || nyx_tv_lib::add_custom_source(custom_name)).await {
                    Ok(()) => match blocking(nyx_tv_lib::get_sources).await {
                        Ok(sources) => {
                            app.write().sources = sources;
                            app.write().loaded = true;
                            notify(app, ToastKind::Success, "Custom source added successfully");
                            navigator.replace(Route::Home {});
                        }
                        Err(message) => {
                            report_error(app, message.clone());
                            error.set(Some(message));
                        }
                    },
                    Err(message) => {
                        report_error(app, message.clone());
                        error.set(Some(message));
                    }
                }
                loading.set(false);
                return;
            }

            let mut source = Source {
                id: None,
                name: name_value,
                url: if source_kind == 0 {
                    file_path
                } else {
                    Some(url_value)
                },
                url_origin: None,
                username: (source_kind == 2).then_some(username_value),
                password: (source_kind == 2).then_some(password_value),
                source_type: source_kind,
                use_tvg_id: (source_kind != 2).then_some(false),
                enabled: true,
                user_agent: None,
                max_streams: None,
                stream_user_agent: None,
                last_updated: None,
            };
            if source_kind == source_type::XTREAM
                && let Some(address) = source.url.as_mut()
            {
                match nyx_tv_lib::xtream::normalize_xtream_url(address) {
                    Ok(normalized) => *address = normalized,
                    Err(message) => {
                        error.set(Some(message.to_string()));
                        return;
                    }
                }
            }
            loading.set(true);
            error.set(None);
            let result = match source_kind {
                0 => blocking(move || nyx_tv_lib::get_m3u8(source)).await,
                1 => nyx_tv_lib::get_m3u8_from_link(source).await,
                _ => nyx_tv_lib::get_xtream(source).await,
            };
            match result {
                Ok(()) => match blocking(nyx_tv_lib::get_sources).await {
                    Ok(sources) => {
                        app.write().sources = sources;
                        app.write().loaded = true;
                        notify(app, ToastKind::Success, "Source added successfully");
                        navigator.replace(Route::Home {});
                    }
                    Err(message) => {
                        report_error(app, message.clone());
                        error.set(Some(message));
                    }
                },
                Err(message) => {
                    report_error(app, message.clone());
                    error.set(Some(message));
                }
            }
            loading.set(false);
        });
    };

    rsx! {
        main { class: "grid min-h-screen place-items-center px-5 py-10",
            section { class: "w-full max-w-lg rounded-xl border border-border bg-card p-6 shadow-xl",
                header { class: "mb-6",
                    h1 { class: "text-2xl font-semibold", "Add a source" }
                    p { class: "mt-1 text-sm text-muted-foreground", "Connect a playlist or Xtream service to get started." }
                    if let Some(message) = app.read().error.clone() {
                        p { class: "mt-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive", "{message}" }
                    }
                }
                div { class: "mb-5 grid grid-cols-2 gap-2 sm:grid-cols-4",
                    for (kind, label) in [
                        (0_u8, "M3U file"),
                        (1, "M3U URL"),
                        (2, "Xtream"),
                        (source_type::CUSTOM, "Custom"),
                    ] {
                        button {
                            class: if source_kind() == kind {
                                "rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground"
                            } else {
                                "rounded-md bg-secondary px-3 py-2 text-sm hover:bg-accent"
                            },
                            onclick: move |_| {
                                source_kind.set(kind);
                                error.set(None);
                            },
                            "{label}"
                        }
                    }
                }
                div { class: "flex flex-col gap-4",
                    label { class: "flex flex-col gap-2 text-sm font-medium",
                        "Source name"
                        Input {
                            placeholder: Some("e.g. My IPTV".to_string()),
                            value: Some(name()),
                            oninput: move |event: FormEvent| name.set(event.value()),
                        }
                    }
                    if source_kind() == 0 {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                spawn(async move {
                                    if let Some(file) = rfd::AsyncFileDialog::new()
                                        .add_filter("M3U playlist", &["m3u", "m3u8"])
                                        .pick_file()
                                        .await
                                    {
                                        file_path.set(Some(file.path().to_string_lossy().to_string()));
                                    }
                                });
                            },
                            if let Some(path) = file_path() {
                                "{path}"
                            } else {
                                "Choose M3U file"
                            }
                        }
                    } else if source_kind() != source_type::CUSTOM {
                        label { class: "flex flex-col gap-2 text-sm font-medium",
                            if source_kind() == 2 { "Server URL" } else { "Playlist URL" }
                            Input {
                                r#type: InputType::Url,
                                placeholder: Some(if source_kind() == 2 { "http://example.com:8080" } else { "https://example.com/list.m3u" }.to_string()),
                                value: Some(url()),
                                oninput: move |event: FormEvent| url.set(event.value()),
                            }
                        }
                    }
                    if source_kind() == 2 {
                        label { class: "flex flex-col gap-2 text-sm font-medium",
                            "Username"
                            Input {
                                value: Some(username()),
                                oninput: move |event: FormEvent| username.set(event.value()),
                            }
                        }
                        label { class: "flex flex-col gap-2 text-sm font-medium",
                            "Password"
                            Input {
                                r#type: InputType::Password,
                                value: Some(password()),
                                oninput: move |event: FormEvent| password.set(event.value()),
                            }
                        }
                    }
                    if let Some(message) = error() {
                        p { class: "rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive", "{message}" }
                    }
                    Button {
                        disabled: loading(),
                        class: Some("w-full".to_string()),
                        onclick: submit,
                        if loading() { "Connecting…" } else { "Add source" }
                    }
                }
                if !app.read().sources.is_empty() {
                    button {
                        class: "mt-4 w-full text-sm text-muted-foreground hover:text-foreground",
                        onclick: move |_| {
                            navigator.replace(Route::Home {});
                        },
                        "Back to channels"
                    }
                }
            }
        }
    }
}
