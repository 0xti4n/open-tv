use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use dioxus::prelude::*;

use crate::state::{AppData, Toast, ToastKind};

static NEXT_TOAST_ID: AtomicU64 = AtomicU64::new(1);

pub fn app_data_dir() -> Option<PathBuf> {
    nyx_tv_lib::paths::project_dirs().map(|directories| directories.data_dir().to_path_buf())
}

pub async fn blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    tokio::task::spawn_blocking(f)
        .await
        .expect("blocking backend task panicked")
}

pub fn notify(mut app: Signal<AppData>, kind: ToastKind, message: impl Into<String>) {
    let id = NEXT_TOAST_ID.fetch_add(1, Ordering::Relaxed);
    app.write().toasts.push(Toast {
        id,
        kind,
        message: message.into(),
        error: None,
    });
    spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        app.write().toasts.retain(|toast| toast.id != id);
    });
}

pub fn report_error(mut app: Signal<AppData>, message: impl Into<String>) {
    let message = message.into();
    let id = NEXT_TOAST_ID.fetch_add(1, Ordering::Relaxed);
    app.write().toasts.push(Toast {
        id,
        kind: ToastKind::Error,
        message: message.clone(),
        error: Some(message),
    });
    spawn(async move {
        tokio::time::sleep(Duration::from_secs(8)).await;
        app.write().toasts.retain(|toast| toast.id != id);
    });
}
