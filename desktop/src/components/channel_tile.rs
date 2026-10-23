use std::sync::Arc;

use dioxus::prelude::*;
use open_tv_lib::{media_type, types::{AppState, Channel}};
use tokio::sync::Mutex;

use crate::{
    backend::{blocking, notify, report_error},
    components::modals::{
        delete_group::DeleteGroupModal,
        dialog::Dialog,
        edit_channel::EditChannelModal,
        epg_modal::EpgModal,
        restream_modal::RestreamModal,
    },
    state::{ActiveDownload, AppData, ToastKind},
};

#[component]
pub fn ChannelTile(
    channel: Channel,
    index: usize,
    source_name: Option<String>,
    selected: bool,
    is_history: bool,
    on_select: EventHandler<usize>,
    on_drill: EventHandler<Channel>,
) -> Element {
    let app_state = use_context::<Arc<Mutex<AppState>>>();
    let app = use_context::<Signal<AppData>>();
    let starting = use_signal(|| false);
    let mut logo_failed = use_signal(|| false);
    let mut menu_position = use_signal(|| None::<(f64, f64)>);
    let mut favorite = use_signal(|| channel.favorite);
    let mut hidden = use_signal(|| channel.hidden.unwrap_or(false));
    let mut epg_open = use_signal(|| false);
    let mut epg_programs = use_signal(Vec::new);
    let mut restream_open = use_signal(|| false);
    let mut edit_open = use_signal(|| false);
    let mut delete_group_open = use_signal(|| false);
    let image = channel.image.clone();
    let name = channel.name.clone();
    let click_channel = channel.clone();
    let click_app_state = app_state.clone();
    let channel_context = channel.clone();
    let channel_history = channel.clone();
    let channel_record = channel.clone();
    let channel_download = channel.clone();
    let channel_epg = channel.clone();
    let channel_share = channel.clone();
    let channel_delete = channel.clone();
    let channel_copy = channel.clone();
    let state_record = app_state.clone();
    let state_download = app_state.clone();
    let state_cancel_download = app_state.clone();
    let click_on_select = on_select;
    let click_on_drill = on_drill;
    let custom_source = app
        .read()
        .sources
        .iter()
        .any(|source| source.id == channel.source_id && source.source_type == 3);
    let can_epg = channel.media_type == media_type::LIVESTREAM
        && app.read().sources.iter().any(|source| {
            source.id == channel.source_id && source.source_type == open_tv_lib::source_type::XTREAM
        });
    let download_id = format!("{}-{}", channel.source_id.unwrap_or_default(), channel.id.unwrap_or_default());
    let download_id_for_download = download_id.clone();
    let download_id_for_cancel = download_id.clone();
    let downloading = app.read().downloads.iter().any(|download| download.id == download_id);

    rsx! {
        article {
            id: "channel-tile-{index}",
            class: if selected {
                "group min-h-48 overflow-hidden rounded-xl border border-primary bg-card shadow-lg ring-2 ring-primary/40"
            } else {
                "group min-h-48 overflow-hidden rounded-xl border border-border bg-card transition hover:border-primary/60 hover:shadow-lg"
            },
            tabindex: 0,
            oncontextmenu: move |event| {
                event.prevent_default();
                if channel_context.media_type != media_type::SEASON {
                    let point = event.client_coordinates();
                    menu_position.set(Some((point.x, point.y)));
                }
            },
            onfocus: move |_| on_select.call(index),
            onclick: move |_| {
                click_on_select.call(index);
                activate_channel(
                    click_channel.clone(),
                    click_app_state.clone(),
                    starting,
                    click_on_drill,
                    app,
                );
            },
            onkeydown: move |event| {
                if event.key() == Key::Enter
                    || matches!(event.key(), Key::Character(character) if character == " ")
                {
                    event.prevent_default();
                    activate_channel(channel.clone(), app_state.clone(), starting, on_drill, app);
                }
            },
            div { class: "relative grid h-32 place-items-center overflow-hidden bg-muted/40 p-3",
                if let Some(src) = image.as_ref().filter(|_| !logo_failed()) {
                    img {
                        class: "max-h-full max-w-full object-contain",
                        src: "{src}",
                        alt: "{name}",
                        onerror: move |_| logo_failed.set(true),
                    }
                } else {
                    div { class: "grid size-16 place-items-center rounded-full bg-secondary text-2xl font-semibold text-muted-foreground",
                        "{name.chars().next().unwrap_or('?').to_ascii_uppercase()}"
                    }
                }
                if channel.tv_archive == Some(true) {
                    span { class: "absolute left-2 top-2 rounded bg-primary px-1.5 py-0.5 text-[10px] font-semibold text-primary-foreground", "ARCHIVE" }
                }
                if channel.favorite {
                    span { class: "absolute right-2 top-2 text-amber-400", "★" }
                }
                if starting() {
                    span { class: "absolute inset-x-0 bottom-0 bg-primary/90 py-1 text-center text-xs font-medium text-primary-foreground", "Starting · click to cancel" }
                }
            }
            div { class: "p-3",
                h2 { class: "truncate text-sm font-medium", title: "{name}", "{name}" }
                p { class: "mt-1 truncate text-xs text-muted-foreground", "{source_name.as_deref().or(channel.group.as_deref()).unwrap_or(\"Live TV\")}" }
            }
            if let Some((x, y)) = menu_position() {
                div {
                    class: "fixed z-[100] min-w-48 rounded-md border border-border bg-card p-1 shadow-xl",
                    style: "left: {x}px; top: {y}px;",
                    onclick: move |event| event.stop_propagation(),
                    if is_history {
                        button {
                            class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                            onclick: move |_| {
                                menu_position.set(None);
                                let Some(id) = channel_history.id else { return; };
                                let channel_name = channel_history.name.clone();
                                spawn(async move {
                                    match blocking(move || open_tv_lib::remove_from_history(id)).await {
                                        Ok(()) => notify(app, ToastKind::Success, format!("Removed \"{channel_name}\" from history")),
                                        Err(message) => report_error(app, message),
                                    }
                                });
                            },
                            "Remove from history"
                        }
                    }
                    if !is_history && !matches!(channel.media_type, media_type::GROUP | media_type::SEASON) {
                        button {
                            class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                            onclick: move |_| {
                                menu_position.set(None);
                                let Some(id) = channel.id else { return; };
                                let was_favorite = favorite();
                                spawn(async move {
                                    let result = if was_favorite {
                                        blocking(move || open_tv_lib::unfavorite_channel(id)).await
                                    } else {
                                        blocking(move || open_tv_lib::favorite_channel(id)).await
                                    };
                                    match result {
                                        Ok(()) => {
                                            favorite.set(!was_favorite);
                                            notify(app, ToastKind::Success, if was_favorite { "Removed from favorites (updates on reload)" } else { "Added to favorites" });
                                        }
                                        Err(message) => report_error(app, message),
                                    }
                                });
                            },
                            if favorite() { "Unfavorite" } else { "Favorite" }
                        }
                    }
                    if !custom_source && !is_history {
                        button {
                            class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                            onclick: move |_| {
                                menu_position.set(None);
                                let Some(id) = channel.id else { return; };
                                let is_group = channel.media_type == media_type::GROUP;
                                let hide = !hidden();
                                spawn(async move {
                                    let result = if is_group {
                                        blocking(move || open_tv_lib::hide_group(id, hide)).await
                                    } else {
                                        blocking(move || open_tv_lib::hide_channel(id, hide)).await
                                    };
                                    match result {
                                        Ok(()) => {
                                            hidden.set(hide);
                                            notify(app, ToastKind::Success, format!("{} (updates on reload)", if hide { "Hidden" } else { "Unhidden" }));
                                        }
                                        Err(message) => report_error(app, message),
                                    }
                                });
                            },
                            if hidden() { "Unhide" } else { "Hide" }
                        }
                    }
                    if channel.media_type == media_type::LIVESTREAM {
                        button {
                            class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                            onclick: move |_| {
                                menu_position.set(None);
                                let channel = channel_record.clone();
                                let app_state = state_record.clone();
                                spawn(async move {
                                    let path = if let Some(file) = rfd::AsyncFileDialog::new()
                                        .add_filter("MP4 recording", &["mp4"])
                                        .set_file_name(format!("{}.mp4", channel.name))
                                        .save_file().await {
                                        Some(file.path().to_string_lossy().to_string())
                                    } else {
                                        return;
                                    };
                                    if let Err(message) = open_tv_lib::play(channel, true, path, &app_state).await {
                                        report_error(app, message);
                                    }
                                });
                            },
                            "Record"
                        }
                    }
                    if channel.media_type == media_type::MOVIE && !downloading {
                        button {
                            class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                            onclick: move |_| {
                                menu_position.set(None);
                                let channel = channel_download.clone();
                                let state = state_download.clone();
                                let id = download_id_for_download.clone();
                                let mut app = app;
                                spawn(async move {
                                    if let Some(file) = rfd::AsyncFileDialog::new()
                                        .add_filter("MP4 video", &["mp4"])
                                        .set_file_name(format!("{}.mp4", channel.name))
                                        .save_file().await
                                    {
                                        let path = file.path().to_string_lossy().to_string();
                                        let source_id = channel.source_id.unwrap_or_default();
                                        app.write().downloads.push(ActiveDownload {
                                            id: id.clone(),
                                            source_id,
                                            name: channel.name.clone(),
                                            progress: 0.0,
                                        });
                                        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
                                        let task_state = state.clone();
                                        let task_id = id.clone();
                                        let task = tokio::spawn(async move {
                                            open_tv_lib::download(&task_state, sender, channel, task_id, Some(path)).await
                                        });
                                        while let Some(progress) = receiver.recv().await {
                                            if let Some(download) = app.write().downloads.iter_mut()
                                                .find(|download| download.id == id)
                                            {
                                                download.progress = progress;
                                            }
                                        }
                                        match task.await {
                                            Ok(Ok(())) => notify(app, ToastKind::Success, "Download completed"),
                                            Ok(Err(message)) => report_error(app, message),
                                            Err(error) => report_error(app, format!("Download task failed: {error}")),
                                        }
                                        app.write().downloads.retain(|download| download.id != id);
                                    }
                                });
                            },
                            "Download"
                        }
                    }
                    if downloading {
                        button {
                            class: "block w-full rounded px-3 py-2 text-left text-sm text-destructive hover:bg-accent",
                            onclick: move |_| {
                                menu_position.set(None);
                                let state = state_cancel_download.clone();
                                let id = download_id_for_cancel.clone();
                                let source_id = channel_context.source_id.unwrap_or_default();
                                spawn(async move {
                                    if let Err(message) = open_tv_lib::abort_download(&state, source_id, id).await {
                                        report_error(app, message);
                                    }
                                });
                            },
                            "Cancel download"
                        }
                    }
                    if can_epg {
                        button {
                            class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                            onclick: move |_| {
                                menu_position.set(None);
                                let channel = channel_epg.clone();
                                spawn(async move {
                                    match open_tv_lib::get_epg(channel).await {
                                        Ok(programs) if programs.is_empty() => notify(app, ToastKind::Info, "No EPG data for this channel"),
                                        Ok(programs) => { epg_programs.set(programs); epg_open.set(true); }
                                        Err(message) => report_error(app, message),
                                    }
                                });
                            },
                            "EPG"
                        }
                    }
                    if custom_source {
                        button { class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent", onclick: move |_| { menu_position.set(None); edit_open.set(true); }, "Edit" }
                        button {
                            class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                            onclick: move |_| {
                                menu_position.set(None);
                                let channel = channel_share.clone();
                                spawn(async move {
                                    if let Some(file) = rfd::AsyncFileDialog::new()
                                        .add_filter("Open TV channel", &["otv"])
                                        .set_file_name(format!("{}.otv", channel.name))
                                        .save_file().await
                                    {
                                        let path = file.path().to_string_lossy().to_string();
                                        match blocking(move || open_tv_lib::share_custom_channel(channel, path)).await {
                                            Ok(()) => notify(app, ToastKind::Success, "Channel exported successfully"),
                                            Err(message) => report_error(app, message),
                                        }
                                    }
                                });
                            },
                            "Share"
                        }
                        if channel.media_type == media_type::LIVESTREAM {
                            button { class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent", onclick: move |_| { menu_position.set(None); restream_open.set(true); }, "Re-stream" }
                        }
                        button {
                            class: "block w-full rounded px-3 py-2 text-left text-sm text-destructive hover:bg-accent",
                            onclick: move |_| {
                                menu_position.set(None);
                                if channel_delete.media_type == media_type::GROUP {
                                    delete_group_open.set(true);
                                } else if let Some(id) = channel_delete.id {
                                    spawn(async move {
                                        match blocking(move || open_tv_lib::delete_custom_channel(id)).await {
                                            Ok(()) => notify(app, ToastKind::Success, "Channel deleted"),
                                            Err(message) => report_error(app, message),
                                        }
                                    });
                                }
                            },
                            "Delete"
                        }
                    }
                    if !matches!(channel.media_type, media_type::GROUP | media_type::SEASON) {
                        button {
                            class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                            onclick: move |_| {
                                menu_position.set(None);
                                if let Some(url) = channel_copy.url.clone() {
                                    spawn(async move {
                                        match blocking(move || {
                                            arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(url))
                                        }).await {
                                            Ok(()) => notify(app, ToastKind::Success, "Stream URL copied"),
                                            Err(error) => report_error(app, format!("Clipboard error: {error}")),
                                        }
                                    });
                                }
                            },
                            "Copy URL"
                        }
                    }
                }
            }
            Dialog { open: epg_open,
                EpgModal { open: epg_open, channel: channel.clone(), programs: epg_programs() }
            }
            Dialog { open: restream_open,
                RestreamModal { open: restream_open, channel: channel.clone() }
            }
            Dialog { open: edit_open,
                EditChannelModal { open: edit_open, source_id: channel.source_id.unwrap_or_default(), channel: Some(channel.clone()) }
            }
            Dialog { open: delete_group_open,
                DeleteGroupModal { open: delete_group_open, group: channel.clone() }
            }
        }
    }
}

fn activate_channel(
    channel: Channel,
    app_state: Arc<Mutex<AppState>>,
    mut starting: Signal<bool>,
    on_drill: EventHandler<Channel>,
    app: Signal<AppData>,
) {
    if matches!(
        channel.media_type,
        media_type::GROUP | media_type::SERIE | media_type::SEASON
    ) {
        on_drill.call(channel);
    } else if starting() {
        if let (Some(source_id), Some(id)) = (channel.source_id, channel.id) {
            spawn(async move {
                if let Err(message) =
                    open_tv_lib::cancel_play(source_id, id, &app_state).await
                {
                    report_error(app, message);
                }
            });
        }
        starting.set(false);
    } else {
        starting.set(true);
        if let Some(id) = channel.id {
            spawn(async move {
                if let Err(message) = blocking(move || open_tv_lib::add_last_watched(id)).await {
                    report_error(app, message);
                }
            });
        }
        spawn(async move {
            if let Err(message) = open_tv_lib::play(channel, false, None, &app_state).await {
                report_error(app, message);
            }
            starting.set(false);
        });
    }
}
