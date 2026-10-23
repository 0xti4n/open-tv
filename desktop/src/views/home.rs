use std::time::Duration;

use dioxus::prelude::*;
use nyx_tv_lib::{
    bulk_action_type, media_type,
    types::{Channel, Filters},
    view_type,
};

use crate::{
    app::Route,
    backend::{blocking, notify, report_error},
    components::{
        channel_tile::ChannelTile,
        sort_button::SortButton,
        ui::{
            button::{Button, ButtonSize, ButtonVariant},
            dropdown_menu::DropdownMenu,
            input::{Input, InputType},
            spinner::Spinner,
        },
    },
    state::{AppData, ToastKind},
};

const PAGE_SIZE: usize = 36;

fn focus_element(id: String, scroll_to_top: bool) {
    spawn(async move {
        if scroll_to_top {
            let _ = document::eval("window.scrollTo({ top: 0, behavior: 'smooth' })").await;
        }
        let _ = document::eval(&format!("document.getElementById('{id}')?.focus()")).await;
    });
}

fn clear_search_input() {
    spawn(async {
        let _ = document::eval("document.getElementById('search').value=''").await;
    });
}

#[component]
pub fn Home() -> Element {
    let app = use_context::<Signal<AppData>>();
    let mut zoom = use_context::<Signal<f64>>();
    let window = dioxus::desktop::use_window();
    let navigator = use_navigator();
    let sources = app.read().sources.clone();
    let enabled_ids = sources
        .iter()
        .filter(|source| source.enabled)
        .filter_map(|source| source.id)
        .collect::<Vec<_>>();

    let mut query = use_signal(String::new);
    let mut search_focused = use_signal(|| false);
    let mut keyword_search = use_signal(|| false);
    let mut view = use_signal(|| view_type::ALL);
    let mut media_types =
        use_signal(|| vec![media_type::LIVESTREAM, media_type::MOVIE, media_type::SERIE]);
    let mut source_ids = use_signal(|| enabled_ids.clone());
    let source_ids_initialized = use_signal(|| app.read().loaded);
    let mut sort = use_signal(|| 2_u8);
    let mut group_id = use_signal(|| None::<i64>);
    let mut series_id = use_signal(|| None::<i64>);
    let mut season = use_signal(|| None::<i64>);
    let mut page = use_signal(|| 1_u8);
    let channels = use_signal(Vec::<Channel>::new);
    let loading = use_signal(|| false);
    let has_more = use_signal(|| true);
    let error = use_signal(|| None::<String>);
    let mut selected_index = use_signal(|| 0_usize);
    let mut bulk_menu = use_signal(|| false);
    let mut sort_menu = use_signal(|| false);
    let mut request_number = use_signal(|| 0_u64);
    let mut refresh_number = use_signal(|| 0_u64);

    use_effect(move || {
        let data = app.read();
        if data.loaded && data.sources.is_empty() {
            navigator.replace(Route::Setup {});
        }
    });

    use_effect(move || {
        let data = app.read();
        if !data.loaded || source_ids_initialized() {
            return;
        }
        let mut source_ids = source_ids;
        let mut source_ids_initialized = source_ids_initialized;
        source_ids.set(
            data.sources
                .iter()
                .filter(|source| source.enabled)
                .filter_map(|source| source.id)
                .collect(),
        );
        source_ids_initialized.set(true);
    });

    use_effect(move || {
        let _ = refresh_number();
        let data = app.read();
        if !data.loaded {
            return;
        }
        let current_query = query();
        let current_page = page();
        let mut request_filters = Filters {
            query: Some(current_query.clone()),
            source_ids: source_ids(),
            media_types: Some(media_types()),
            view_type: view(),
            page: current_page,
            series_id: series_id(),
            group_id: group_id(),
            use_keywords: keyword_search(),
            sort: sort(),
            season: season(),
        };
        if request_filters.page == 0 {
            request_filters.page = 1;
        }
        let page_for_request = request_filters.page;
        let current_request = request_number.with_mut(|number| {
            *number += 1;
            *number
        });
        let mut loading = loading;
        let mut error = error;
        let mut channels = channels;
        let mut has_more = has_more;
        let request_number = request_number;
        let delay = !current_query.is_empty();
        loading.set(true);
        error.set(None);
        spawn(async move {
            if delay {
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
            if request_number() != current_request {
                return;
            }
            if request_filters.source_ids.is_empty() {
                channels.set(Vec::new());
                has_more.set(false);
                loading.set(false);
                return;
            }
            match blocking(move || nyx_tv_lib::search(request_filters)).await {
                Ok(results) => {
                    if request_number() != current_request {
                        return;
                    }
                    has_more.set(results.len() == PAGE_SIZE);
                    channels.with_mut(|current| {
                        if page_for_request == 1 {
                            *current = results;
                        } else {
                            current.extend(results);
                        }
                    });
                }
                Err(message) => {
                    report_error(app, message.clone());
                    error.set(Some(message));
                    if page_for_request == 1 {
                        channels.set(Vec::new());
                    }
                    has_more.set(false);
                }
            }
            loading.set(false);
        });
    });

    if !app.read().loaded {
        return rsx! { div { class: "grid min-h-screen place-items-center", Spinner {} } };
    }

    let active_view = view();
    let title = match active_view {
        view_type::FAVORITES => "Favorites",
        view_type::CATEGORIES => "Categories",
        view_type::HISTORY => "History",
        view_type::HIDDEN => "Hidden",
        _ => "All channels",
    };
    let can_go_back = group_id().is_some() || series_id().is_some() || season().is_some();

    rsx! {
        main {
            id: "top",
            class: "mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-5 px-5 py-5",
            tabindex: 0,
            onkeydown: move |event| {
                let key = event.key();
                let character = match &key {
                    Key::Character(value) => value.to_ascii_lowercase(),
                    _ => String::new(),
                };
                let modifiers = event.data().modifiers();
                let command_modifier = modifiers.contains(Modifiers::CONTROL)
                    || modifiers.contains(Modifiers::META);

                if command_modifier
                    && (character == "f" || event.data().code() == Code::Space)
                {
                    event.prevent_default();
                    if search_focused() {
                        selected_index.set(0);
                        focus_element("channel-tile-0".to_string(), true);
                    } else {
                        focus_element("search".to_string(), false);
                    }
                    return;
                }

                if command_modifier && !search_focused() {
                    let target_view = match character.as_str() {
                        "a" => Some(view_type::ALL),
                        "s" => Some(view_type::CATEGORIES),
                        "d" => Some(view_type::HISTORY),
                        "r" => Some(view_type::FAVORITES),
                        _ => None,
                    };
                    if let Some(target_view) = target_view {
                        view.set(target_view);
                        page.set(1);
                        group_id.set(None);
                        series_id.set(None);
                        season.set(None);
                        selected_index.set(0);
                        event.prevent_default();
                        return;
                    }
                }

                if modifiers.contains(Modifiers::CONTROL) {
                    let next_zoom = match character.as_str() {
                        "=" | "+" => Some((zoom() + 0.1).min(10.0)),
                        "-" => Some((zoom() - 0.1).max(0.1)),
                        "0" => Some(1.0),
                        _ => None,
                    };
                    if let Some(next_zoom) = next_zoom {
                        zoom.set(next_zoom);
                        window.set_zoom_level(next_zoom);
                        event.prevent_default();
                        return;
                    }

                    let media_type = match character.as_str() {
                        "q" => Some(media_type::LIVESTREAM),
                        "w" => Some(media_type::MOVIE),
                        "e" => Some(media_type::SERIE),
                        _ => None,
                    };
                    if let Some(media_type) = media_type {
                        media_types.with_mut(|items| {
                            if items.contains(&media_type) {
                                if items.len() > 1 {
                                    items.retain(|item| *item != media_type);
                                }
                            } else {
                                items.push(media_type);
                            }
                        });
                        page.set(1);
                        selected_index.set(0);
                        event.prevent_default();
                        return;
                    }
                }

                if matches!(key, Key::Escape | Key::BrowserBack)
                    || (key == Key::Backspace && !search_focused())
                {
                    event.prevent_default();
                    if bulk_menu() {
                        bulk_menu.set(false);
                    } else if sort_menu() {
                        sort_menu.set(false);
                    } else if search_focused() {
                        selected_index.set(0);
                        focus_element("channel-tile-0".to_string(), true);
                    } else if !query().is_empty() {
                        query.set(String::new());
                        clear_search_input();
                        page.set(1);
                        selected_index.set(0);
                        focus_element("channel-tile-0".to_string(), true);
                    } else if can_go_back {
                        if season().is_some() {
                            season.set(None);
                        } else if series_id().is_some() {
                            series_id.set(None);
                        } else {
                            group_id.set(None);
                        }
                        page.set(1);
                        selected_index.set(0);
                        focus_element("channel-tile-0".to_string(), true);
                    } else {
                        selected_index.set(0);
                        focus_element("channel-tile-0".to_string(), true);
                    }
                    return;
                }

                if !search_focused() {
                    let channel_count = channels().len();
                    if channel_count > 0 {
                        let current_index = selected_index();
                        let next_index = match key {
                            Key::ArrowRight => Some((current_index + 1).min(channel_count - 1)),
                            Key::ArrowLeft => Some(current_index.saturating_sub(1)),
                            Key::ArrowDown => Some((current_index + 3).min(channel_count - 1)),
                            Key::ArrowUp => Some(current_index.saturating_sub(3)),
                            _ => None,
                        };
                        if let Some(next_index) = next_index {
                            event.prevent_default();
                            selected_index.set(next_index);
                            focus_element(format!("channel-tile-{next_index}"), false);
                        }
                    }
                }
            },
            header { class: "flex flex-wrap items-center justify-between gap-3",
                div { class: "flex items-center gap-3",
                    h1 { class: "text-2xl font-semibold tracking-tight", "Nyx TV" }
                    span { class: "text-sm text-muted-foreground", "{title}" }
                }
                nav { class: "flex flex-wrap items-center gap-2",
                    button {
                        class: "rounded-md border border-border bg-card px-3 py-2 text-sm hover:bg-accent",
                        onclick: move |_| {
                            navigator.push(Route::Settings {});
                        },
                        "Settings"
                    }
                    button {
                        class: "rounded-md border border-border bg-card px-3 py-2 text-sm hover:bg-accent",
                        onclick: move |_| {
                            navigator.push(Route::Setup {});
                        },
                        "Add source"
                    }
                    button {
                        class: "rounded-md border border-border bg-card px-3 py-2 text-sm hover:bg-accent",
                        onclick: move |_| {
                            let mut loading = loading;
                            let mut app = app;
                            spawn(async move {
                                loading.set(true);
                                if let Err(message) = nyx_tv_lib::refresh_all().await {
                                    report_error(app, message);
                                }
                                match blocking(nyx_tv_lib::get_sources).await {
                                    Ok(sources) => app.write().sources = sources,
                                    Err(message) => report_error(app, message),
                                }
                                refresh_number.with_mut(|number| *number += 1);
                                loading.set(false);
                            });
                        },
                        "Reload"
                    }
                }
            }

            section { class: "flex flex-col gap-3 rounded-xl border border-border bg-card p-4",
                div { class: "flex flex-wrap items-center gap-2",
                    div { class: "min-w-56 flex-1",
                        Input {
                            r#type: InputType::Search,
                            id: Some("search".to_string()),
                            placeholder: Some("Search channels".to_string()),
                            onfocus: move |_| search_focused.set(true),
                            onblur: move |_| search_focused.set(false),
                            oninput: move |event: FormEvent| {
                                query.set(event.value());
                                page.set(1);
                                selected_index.set(0);
                            }
                        }
                    }
                    Button {
                        size: ButtonSize::Sm,
                        variant: if keyword_search() { ButtonVariant::Default } else { ButtonVariant::Outline },
                        onclick: move |_| keyword_search.set(!keyword_search()),
                        "Keywords"
                    }
                    SortButton {
                        sort: sort(),
                        open: sort_menu,
                        on_change: move |value| {
                            sort.set(value);
                            page.set(1);
                        },
                    }
                    DropdownMenu {
                        open: bulk_menu,
                        label: "Bulk actions".to_string(),
                        for (action, label) in [
                            (bulk_action_type::HIDE, "Hide"),
                            (bulk_action_type::UNHIDE, "Unhide"),
                            (bulk_action_type::FAVORITE, "Favorite"),
                            (bulk_action_type::UNFAVORITE, "Unfavorite"),
                        ] {
                            button {
                                class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                                onclick: move |_| {
                                    let filters = Filters {
                                        query: Some(query()),
                                        source_ids: source_ids(),
                                        media_types: Some(media_types()),
                                        view_type: view(),
                                        page: 1,
                                        series_id: series_id(),
                                        group_id: group_id(),
                                        use_keywords: keyword_search(),
                                        sort: sort(),
                                        season: season(),
                                    };
                                    bulk_menu.set(false);
                                    spawn(async move {
                                        match blocking(move || nyx_tv_lib::bulk_update(filters, action)).await {
                                            Ok(()) => {
                                                page.set(1);
                                                refresh_number.with_mut(|number| *number += 1);
                                                notify(app, ToastKind::Success, "Bulk update completed");
                                            }
                                            Err(message) => report_error(app, message),
                                        }
                                    });
                                },
                                "{label}"
                            }
                        }
                    }
                }

                div { class: "flex flex-wrap items-center gap-2",
                    for (value, label) in [
                        (view_type::ALL, "All"),
                        (view_type::CATEGORIES, "Categories"),
                        (view_type::HISTORY, "History"),
                        (view_type::FAVORITES, "Favorites"),
                        (view_type::HIDDEN, "Hidden"),
                    ] {
                        button {
                            class: if view() == value {
                                "rounded-md bg-primary px-3 py-1.5 text-sm text-primary-foreground"
                            } else {
                                "rounded-md bg-secondary px-3 py-1.5 text-sm text-secondary-foreground hover:bg-accent"
                            },
                            onclick: move |_| {
                                view.set(value);
                                page.set(1);
                                group_id.set(None);
                                series_id.set(None);
                                season.set(None);
                                selected_index.set(0);
                            },
                            "{label}"
                        }
                    }
                    span { class: "mx-1 h-6 w-px bg-border" }
                    for (value, label) in [
                        (media_type::LIVESTREAM, "Live"),
                        (media_type::MOVIE, "Movies"),
                        (media_type::SERIE, "Series"),
                    ] {
                        button {
                            class: if media_types().contains(&value) {
                                "rounded-md border border-primary/50 bg-primary/10 px-3 py-1.5 text-sm"
                            } else {
                                "rounded-md border border-border px-3 py-1.5 text-sm text-muted-foreground"
                            },
                            onclick: move |_| {
                                media_types.with_mut(|items| {
                                    if items.contains(&value) {
                                        if items.len() > 1 {
                                            items.retain(|item| *item != value);
                                        }
                                    } else {
                                        items.push(value);
                                    }
                                });
                                page.set(1);
                            },
                            "{label}"
                        }
                    }
                }

                if !sources.is_empty() {
                    div { class: "flex flex-wrap items-center gap-2 text-xs text-muted-foreground",
                        span { "Sources:" }
                        for source in sources.iter().filter(|source| source.enabled) {
                            if let Some(id) = source.id {
                                button {
                                    class: if source_ids().contains(&id) {
                                        "rounded-full border border-primary/50 px-2.5 py-1 text-foreground"
                                    } else {
                                        "rounded-full border border-border px-2.5 py-1 opacity-60"
                                    },
                                    onclick: move |_| {
                                        source_ids.with_mut(|ids| {
                                            if ids.contains(&id) {
                                                if ids.len() > 1 {
                                                    ids.retain(|selected| *selected != id);
                                                }
                                            } else {
                                                ids.push(id);
                                            }
                                        });
                                        page.set(1);
                                    },
                                    "{source.name}"
                                }
                            }
                        }
                    }
                }
            }

            if can_go_back {
                button {
                    class: "w-fit rounded-md border border-border px-3 py-2 text-sm hover:bg-accent",
                    onclick: move |_| {
                        if season().is_some() {
                            season.set(None);
                        } else if series_id().is_some() {
                            series_id.set(None);
                        } else {
                            group_id.set(None);
                        }
                        page.set(1);
                    },
                    "← Back"
                }
            }

            if let Some(message) = error() {
                p { class: "rounded-md border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm text-destructive", "{message}" }
            }

            if channels().is_empty() && !loading() {
                section { class: "grid min-h-64 place-items-center rounded-xl border border-dashed border-border bg-card/50 text-center",
                    div {
                        h2 { class: "font-medium", "No channels found" }
                        p { class: "mt-1 text-sm text-muted-foreground", "Try another search or filter." }
                    }
                }
            } else {
                section {
                    id: "channel-grid",
                    class: "grid auto-rows-max max-h-[calc(100vh-19rem)] min-h-48 flex-1 grid-cols-[repeat(auto-fill,minmax(155px,1fr))] content-start gap-3 overflow-y-auto",
                    onscroll: move |event| {
                        let scroll = event.data();
                        let threshold = (scroll.scroll_top() + f64::from(scroll.client_height()))
                            >= f64::from(scroll.scroll_height()) * 0.75;
                        if threshold && has_more() && !loading() {
                            page.with_mut(|current| *current = current.saturating_add(1));
                        }
                    },
                    for (index, channel) in channels().into_iter().enumerate() {
                        ChannelTile {
                            key: "{channel.source_id:?}-{channel.id:?}-{index}",
                            index,
                            source_name: channel
                                .source_id
                                .and_then(|id| sources.iter().find(|source| source.id == Some(id)))
                                .map(|source| source.name.clone()),
                            channel,
                            selected: index == selected_index(),
                            is_history: view() == view_type::HISTORY,
                            on_select: move |selected| selected_index.set(selected),
                            on_drill: move |channel: Channel| {
                                match channel.media_type {
                                    media_type::GROUP => group_id.set(channel.id.or(channel.group_id)),
                                    media_type::SERIE => {
                                        series_id.set(channel.series_id.map(|id| id as i64));
                                        let episode_channel = channel.clone();
                                        spawn(async move {
                                            if let Err(message) = nyx_tv_lib::get_episodes(episode_channel).await {
                                                report_error(app, message);
                                            }
                                        });
                                    }
                                    media_type::SEASON => season.set(channel.season_id),
                                    _ => {}
                                }
                                page.set(1);
                            }
                        }
                    }
                }
            }

            if loading() {
                div { class: "flex justify-center py-2", Spinner {} }
            }
            if has_more() && !channels().is_empty() {
                button {
                    class: "mx-auto w-fit rounded-md border border-border px-4 py-2 text-sm hover:bg-accent",
                    disabled: loading(),
                    onclick: move |_| page.with_mut(|current| *current = current.saturating_add(1)),
                    "Load more"
                }
            }
            a {
                class: "fixed bottom-5 right-5 grid size-10 place-items-center rounded-full border border-border bg-card shadow-lg hover:bg-accent",
                href: "#top",
                title: "Scroll to top",
                "↑"
            }
        }
    }
}
