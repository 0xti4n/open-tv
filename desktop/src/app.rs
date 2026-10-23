use std::sync::Arc;

use dioxus::prelude::*;
use open_tv_lib::types::AppState;
use tokio::sync::Mutex;

use crate::{
    backend::{blocking, report_error},
    components::{
        modals::error_modal::ErrorModal,
        modals::whats_new_modal::WhatsNewModal,
        download_manager::DownloadManager,
        tray::TrayIntegration,
        toasts::Toasts,
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

pub fn app() -> Element {
    let mut app_data = use_context_provider(|| Signal::new(AppData::default()));
    let app_state = use_context_provider(|| Arc::new(Mutex::new(AppState::default())));
    let mut whats_new_open = use_signal(|| false);

    use_effect(move || {
        let app_state = app_state.clone();
        spawn(async move {
            if let Err(message) = blocking(open_tv_lib::initialize_database).await {
                report_error(app_data, message.clone());
                let mut data = app_data.write();
                data.error = Some(message);
                data.loaded = true;
                return;
            }

            let settings = match blocking(open_tv_lib::get_settings).await {
                Ok(settings) => Some(settings),
                Err(message) => {
                    report_error(app_data, message);
                    None
                }
            };
            let sources = match blocking(open_tv_lib::get_sources).await {
                Ok(sources) => sources,
                Err(message) => {
                    report_error(app_data, message);
                    Vec::new()
                }
            };

            if settings
                .as_ref()
                .is_some_and(|settings| settings.refresh_on_start == Some(true))
                && !sources.is_empty()
                && let Err(message) = open_tv_lib::refresh_all().await
            {
                report_error(app_data, message);
            }
            if !sources.is_empty()
                && let Err(message) = open_tv_lib::on_start_check_epg(&app_state).await
            {
                report_error(app_data, message);
            }
            if !sources.is_empty() {
                let last_seen = dioxus::document::eval(
                    "return localStorage.getItem('lastSeenVersion')",
                )
                .join::<Option<String>>()
                .await
                .unwrap_or(None);
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
