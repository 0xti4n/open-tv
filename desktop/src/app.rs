use std::sync::Arc;

use dioxus::prelude::*;
use nyx_tv_lib::types::AppState;
use tokio::sync::Mutex;

use crate::{
    backend::{app_data_dir, blocking, report_error},
    components::{
        download_manager::DownloadManager, modals::error_modal::ErrorModal,
        modals::whats_new_modal::WhatsNewModal, toasts::Toasts, tray::TrayIntegration,
    },
    state::AppData,
    views::{home::Home, settings::Settings, setup::Setup},
};

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/setup")]
    Setup {},
    #[route("/settings")]
    Settings {},
}

const GEIST: Asset = asset!(
    "/assets/fonts/Geist-VariableFont_wght.ttf",
    AssetOptions::builder().with_hash_suffix(false)
);
const GEIST_ITALIC: Asset = asset!(
    "/assets/fonts/Geist-Italic-VariableFont_wght.ttf",
    AssetOptions::builder().with_hash_suffix(false)
);
const OXANIUM: Asset = asset!(
    "/assets/fonts/Oxanium-VariableFont_wght.ttf",
    AssetOptions::builder().with_hash_suffix(false)
);

pub fn app() -> Element {
    let mut app_data = use_context_provider(|| Signal::new(AppData::default()));
    let app_state = use_context_provider(|| Arc::new(Mutex::new(AppState::default())));
    let mut zoom = use_context_provider(|| Signal::new(1.0_f64));
    let mut whats_new_open = use_signal(|| false);
    let window = dioxus::desktop::use_window();

    use_effect(move || {
        let app_state = app_state.clone();
        let window = window.clone();
        spawn(async move {
            if let Err(message) = blocking(nyx_tv_lib::initialize_database).await {
                report_error(app_data, message.clone());
                let mut data = app_data.write();
                data.error = Some(message);
                data.loaded = true;
                return;
            }

            let settings = match blocking(nyx_tv_lib::get_settings).await {
                Ok(settings) => Some(settings),
                Err(message) => {
                    report_error(app_data, message);
                    None
                }
            };
            let sources = match blocking(nyx_tv_lib::get_sources).await {
                Ok(sources) => sources,
                Err(message) => {
                    report_error(app_data, message);
                    Vec::new()
                }
            };

            let zoom_level = settings
                .as_ref()
                .and_then(|settings| settings.zoom)
                .unwrap_or(100) as f64
                / 100.0;
            zoom.set(zoom_level);
            window.set_zoom_level(zoom_level);

            if settings
                .as_ref()
                .is_some_and(|settings| settings.refresh_on_start == Some(true))
                && !sources.is_empty()
                && let Err(message) = nyx_tv_lib::refresh_all().await
            {
                report_error(app_data, message);
            }
            if !sources.is_empty()
                && let Err(message) = nyx_tv_lib::on_start_check_epg(&app_state).await
            {
                report_error(app_data, message);
            }
            if !sources.is_empty() {
                let path = app_data_dir().map(|directory| directory.join("last_seen_version"));
                let last_seen =
                    blocking(move || path.and_then(|path| std::fs::read_to_string(path).ok()))
                        .await;
                if last_seen.as_deref() != Some(env!("CARGO_PKG_VERSION")) {
                    whats_new_open.set(true);
                }
            }

            let mut data = app_data.write();
            data.settings = settings;
            data.sources = sources;
            data.loaded = true;
        });
    });

    rsx! {
        for font in [GEIST, GEIST_ITALIC, OXANIUM] {
            document::Link { rel: "preload", href: font, r#as: "font", r#type: "font/ttf", crossorigin: "anonymous" }
        }
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div { class: "dark min-h-screen bg-background text-foreground",
            Router::<Route> {}
            Toasts {}
            DownloadManager {}
            TrayIntegration {}
            ErrorModal {}
            WhatsNewModal { open: whats_new_open, version: env!("CARGO_PKG_VERSION").to_string() }
        }
    }
}
