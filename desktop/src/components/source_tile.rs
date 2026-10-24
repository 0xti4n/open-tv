use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use nyx_tv_lib::{source_type, types::Source};

use crate::{
    backend::{blocking, notify, report_error},
    components::modals::{
        dialog::Dialog, edit_channel::EditChannelModal, edit_group::EditGroupModal,
        import_modal::ImportModal,
    },
    state::{AppData, ToastKind},
};

fn expiry_label(expiry: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default();
    let remaining = expiry.saturating_sub(now);
    if remaining <= 0 {
        "expired".to_string()
    } else {
        format!("{}d {}h", remaining / 86_400, (remaining % 86_400) / 3_600)
    }
}

#[component]
pub fn SourceTile(source: Source, expiry: Option<i64>) -> Element {
    let mut app = use_context::<Signal<AppData>>();
    let mut source = use_signal(|| source);
    let mut editing = use_signal(|| false);
    let mut draft = use_signal(|| source.read().clone());
    let mut loading = use_signal(|| false);
    let mut show_username = use_signal(|| false);
    let mut show_password = use_signal(|| false);
    let mut add_channel_open = use_signal(|| false);
    let mut add_group_open = use_signal(|| false);
    let mut import_open = use_signal(|| false);
    let kind = source().source_type;
    let kind_name = match kind {
        source_type::M3U => "M3U file",
        source_type::M3U_LINK => "M3U URL",
        source_type::XTREAM => "Xtream",
        _ => "Custom",
    };

    let refresh_sources = move || {
        spawn(async move {
            match blocking(nyx_tv_lib::get_sources).await {
                Ok(sources) => app.write().sources = sources,
                Err(message) => report_error(app, message),
            }
        });
    };

    rsx! {
        article { class: "mb-3 rounded-xl border border-border bg-card p-4",
            header { class: "flex flex-wrap items-center justify-between gap-3",
                div {
                    h3 { class: "font-semibold", "{source().name}" }
                    p { class: "text-xs text-muted-foreground", "{kind_name} · refreshed {source().last_updated.unwrap_or(0)}" }
                }
                if !editing() {
                    button {
                        class: if source().enabled { "rounded border border-emerald-500 px-3 py-1 text-sm text-emerald-500" } else { "rounded border border-border px-3 py-1 text-sm text-muted-foreground" },
                        onclick: move |_| {
                            let source_id = source().id.unwrap_or_default();
                            let enabled = !source().enabled;
                            spawn(async move {
                                match blocking(move || nyx_tv_lib::toggle_source(enabled, source_id)).await {
                                    Ok(()) => {
                                        source.with_mut(|source| source.enabled = enabled);
                                        notify(app, ToastKind::Success, "Source enabled state updated");
                                    }
                                    Err(message) => report_error(app, message),
                                }
                            });
                        },
                        if source().enabled { "Enabled" } else { "Disabled" }
                    }
                }
            }
            if !editing() {
                div { class: "mt-3 grid gap-2 text-sm md:grid-cols-2",
                    if kind != source_type::CUSTOM {
                        p { class: "break-all", "URL: {source().url.as_deref().unwrap_or(\"—\")}" }
                    }
                    if kind != source_type::M3U && kind != source_type::CUSTOM {
                        p { "User-agent: {source().user_agent.as_deref().unwrap_or(\"Nyx TV\")}" }
                    }
                    p { "Stream user-agent: {source().stream_user_agent.as_deref().unwrap_or(\"—\")}" }
                    if kind != source_type::CUSTOM {
                        p { "Max streams: {source().max_streams.unwrap_or(1)}" }
                    }
                    if kind == source_type::XTREAM {
                        div { class: "flex flex-wrap items-center gap-2",
                            span { "Username: " }
                            if show_username() { "{source().username.as_deref().unwrap_or(\"\")}" } else { "••••••" }
                            button { class: "text-primary", onclick: move |_| show_username.set(!show_username()), "Show/hide" }
                        }
                        div { class: "flex flex-wrap items-center gap-2",
                            span { "Password: " }
                            if show_password() { "{source().password.as_deref().unwrap_or(\"\")}" } else { "••••••" }
                            button { class: "text-primary", onclick: move |_| show_password.set(!show_password()), "Show/hide" }
                        }
                        if let Some(expiry) = expiry {
                            p { "Expires: {expiry_label(expiry)}" }
                        }
                    }
                }
            } else {
                div { class: "mt-3 grid gap-3 md:grid-cols-2",
                    if kind != source_type::CUSTOM {
                        label { class: "grid gap-1 text-sm", "URL"
                            if kind == source_type::M3U {
                                div { class: "flex gap-2",
                                    input { class: "min-w-0 flex-1 rounded border border-border bg-background px-3 py-2", readonly: true, value: "{draft().url.as_deref().unwrap_or(\"\")}" }
                                    button {
                                        class: "rounded border border-border px-3",
                                        onclick: move |_| { spawn(async move {
                                            if let Some(file) = rfd::AsyncFileDialog::new().add_filter("M3U playlist", &["m3u", "m3u8"]).pick_file().await {
                                                draft.with_mut(|draft| draft.url = Some(file.path().to_string_lossy().to_string()));
                                            }
                                        }); },
                                        "Browse"
                                    }
                                }
                            } else {
                                input {
                                    class: "rounded border border-border bg-background px-3 py-2",
                                    value: "{draft().url.as_deref().unwrap_or(\"\")}",
                                    oninput: move |event| draft.with_mut(|draft| draft.url = Some(event.value())),
                                }
                            }
                        }
                    }
                    if kind != source_type::M3U && kind != source_type::CUSTOM {
                        label { class: "grid gap-1 text-sm", "User-agent"
                            input { class: "rounded border border-border bg-background px-3 py-2", value: "{draft().user_agent.as_deref().unwrap_or(\"\")}", oninput: move |event| draft.with_mut(|draft| draft.user_agent = Some(event.value())) }
                        }
                    }
                    label { class: "grid gap-1 text-sm", "Stream user-agent"
                        input { class: "rounded border border-border bg-background px-3 py-2", value: "{draft().stream_user_agent.as_deref().unwrap_or(\"\")}", oninput: move |event| draft.with_mut(|draft| draft.stream_user_agent = Some(event.value())) }
                    }
                    if kind != source_type::CUSTOM {
                        label { class: "grid gap-1 text-sm", "Max streams"
                            input { r#type: "number", min: "1", max: "255", class: "rounded border border-border bg-background px-3 py-2", value: "{draft().max_streams.unwrap_or(1)}", oninput: move |event| draft.with_mut(|draft| draft.max_streams = event.value().parse().ok()) }
                        }
                    }
                    if kind == source_type::XTREAM {
                        label { class: "grid gap-1 text-sm", "Username"
                            input { class: "rounded border border-border bg-background px-3 py-2", value: "{draft().username.as_deref().unwrap_or(\"\")}", oninput: move |event| draft.with_mut(|draft| draft.username = Some(event.value())) }
                        }
                        label { class: "grid gap-1 text-sm", "Password"
                            input { class: "rounded border border-border bg-background px-3 py-2", value: "{draft().password.as_deref().unwrap_or(\"\")}", oninput: move |event| draft.with_mut(|draft| draft.password = Some(event.value())) }
                        }
                    }
                    if kind == source_type::M3U || kind == source_type::M3U_LINK {
                        label { class: "flex items-center gap-2 text-sm",
                            input { r#type: "checkbox", checked: draft().use_tvg_id.unwrap_or(false), onchange: move |event| draft.with_mut(|draft| draft.use_tvg_id = Some(event.value() == "true")) }
                            "Use TVG-ID when TVG-NAME is unavailable"
                        }
                    }
                }
            }
            footer { class: "mt-4 flex flex-wrap gap-2",
                if editing() {
                    button {
                        class: "rounded bg-primary px-3 py-2 text-sm text-primary-foreground",
                        disabled: loading(),
                        onclick: move |_| {
                            let mut next = draft();
                            if kind == source_type::XTREAM {
                                if let Some(url) = next.url.as_deref() {
                                    match nyx_tv_lib::xtream::normalize_xtream_url(url) {
                                        Ok(url) => next.url = Some(url),
                                        Err(message) => {
                                            report_error(app, message.to_string());
                                            return;
                                        }
                                    }
                                }
                                next.username = next.username.map(|value| value.trim().to_string());
                                next.password = next.password.map(|value| value.trim().to_string());
                            }
                            next.user_agent = next.user_agent.map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
                            next.stream_user_agent = next.stream_user_agent.map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
                            loading.set(true);
                            spawn(async move {
                                let source_to_save = next.clone();
                                match blocking(move || nyx_tv_lib::update_source(source_to_save)).await {
                                    Ok(()) => {
                                        source.set(next.clone());
                                        draft.set(next);
                                        editing.set(false);
                                        notify(app, ToastKind::Success, "Successfully saved changes");
                                        refresh_sources();
                                    }
                                    Err(message) => report_error(app, message),
                                }
                                loading.set(false);
                            });
                        },
                        "Save"
                    }
                    button { class: "rounded border border-border px-3 py-2 text-sm", onclick: move |_| { draft.set(source()); editing.set(false); }, "Cancel" }
                } else {
                    if kind != source_type::CUSTOM {
                        button {
                            class: "rounded border border-border px-3 py-2 text-sm",
                            onclick: move |_| { draft.set(source()); editing.set(true); },
                            "Edit"
                        }
                        button {
                            class: "rounded border border-border px-3 py-2 text-sm",
                            disabled: loading(),
                            onclick: move |_| {
                                let source = source();
                                loading.set(true);
                                spawn(async move {
                                    match nyx_tv_lib::refresh_source(source).await {
                                        Ok(warnings) => {
                                            notify(app, ToastKind::Success, "Successfully updated source");
                                            for warning in warnings {
                                                notify(app, ToastKind::Info, warning);
                                            }
                                        }
                                        Err(message) => report_error(app, message),
                                    }
                                    loading.set(false);
                                });
                            },
                            "Refresh"
                        }
                        button {
                            class: "rounded border border-border px-3 py-2 text-sm",
                            onclick: move |_| { spawn(async move {
                                if let Some(file) = rfd::AsyncFileDialog::new().add_filter("Favorites backup", &["otvf"]).set_file_name(format!("{}_favs.otvf", source().name)).save_file().await {
                                    let id = source().id.unwrap_or_default();
                                    let path = file.path().to_string_lossy().to_string();
                                    match blocking(move || nyx_tv_lib::backup_favs(id, path)).await {
                                        Ok(()) => notify(app, ToastKind::Success, "Successfully saved favorites backup"),
                                        Err(message) => report_error(app, message),
                                    }
                                }
                            }); },
                            "Backup favs"
                        }
                        button {
                            class: "rounded border border-border px-3 py-2 text-sm",
                            onclick: move |_| { spawn(async move {
                                if let Some(file) = rfd::AsyncFileDialog::new().add_filter("Favorites backup", &["otvf"]).pick_file().await {
                                    let id = source().id.unwrap_or_default();
                                    let path = file.path().to_string_lossy().to_string();
                                    match blocking(move || nyx_tv_lib::restore_favs(id, path)).await {
                                        Ok(()) => notify(app, ToastKind::Success, "Successfully restored favorites backup"),
                                        Err(message) => report_error(app, message),
                                    }
                                }
                            }); },
                            "Restore favs"
                        }
                    } else {
                        button { class: "rounded border border-border px-3 py-2 text-sm", onclick: move |_| add_channel_open.set(true), "Add channel" }
                        button { class: "rounded border border-border px-3 py-2 text-sm", onclick: move |_| add_group_open.set(true), "Add group" }
                        button {
                            class: "rounded border border-border px-3 py-2 text-sm",
                            onclick: move |_| { spawn(async move {
                                if let Some(file) = rfd::AsyncFileDialog::new().add_filter("Nyx TV playlist", &["otvp"]).set_file_name(format!("{}.otvp", source().name)).save_file().await {
                                    let source = source();
                                    let path = file.path().to_string_lossy().to_string();
                                    let export_path = path.clone();
                                    match blocking(move || nyx_tv_lib::share_custom_source(source, export_path)).await {
                                        Ok(()) => notify(app, ToastKind::Success, format!("Successfully exported source in {path}")),
                                        Err(message) => report_error(app, message),
                                    }
                                }
                            }); },
                            "Share"
                        }
                        button { class: "rounded border border-border px-3 py-2 text-sm", onclick: move |_| import_open.set(true), "Import channel/group" }
                    }
                    button {
                        class: "rounded bg-destructive px-3 py-2 text-sm text-white",
                        disabled: loading(),
                        onclick: move |_| {
                            let id = source().id.unwrap_or_default();
                            spawn(async move {
                                match blocking(move || nyx_tv_lib::delete_source(id)).await {
                                    Ok(()) => {
                                        notify(app, ToastKind::Success, "Successfully deleted source");
                                        refresh_sources();
                                    }
                                    Err(message) => report_error(app, message),
                                }
                            });
                        },
                        "Delete"
                    }
                }
            }
        }
        Dialog { open: add_channel_open,
            EditChannelModal { open: add_channel_open, source_id: source().id.unwrap_or_default(), channel: None }
        }
        Dialog { open: add_group_open,
            EditGroupModal { open: add_group_open, source_id: source().id.unwrap_or_default(), group: None }
        }
        Dialog { open: import_open,
            ImportModal { open: import_open, source_id: source().id }
        }
    }
}
