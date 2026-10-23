use std::collections::HashMap;

use open_tv_lib::types::{Settings as AppSettings, Source};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Error,
    Info,
    Success,
}

#[derive(Clone)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub message: String,
    pub error: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct ActiveDownload {
    pub id: String,
    pub source_id: i64,
    pub name: String,
    pub progress: f64,
}

#[derive(Clone, Default)]
pub struct AppData {
    pub loaded: bool,
    pub sources: Vec<Source>,
    pub settings: Option<AppSettings>,
    pub expiries: HashMap<i64, i64>,
    pub error: Option<String>,
    pub error_modal: Option<String>,
    pub toasts: Vec<Toast>,
    pub downloads: Vec<ActiveDownload>,
}
