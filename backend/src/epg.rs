use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
    },
    thread::{self, sleep},
    time::Duration,
};

use anyhow::{Context, Result};
use chrono::Local;
use tokio::sync::Mutex;

use crate::{
    log, sql,
    types::{AppState, EPGNotify},
    utils,
};

pub fn poll(mut to_watch: Vec<EPGNotify>, stop: Arc<AtomicBool>) -> Result<()> {
    while !stop.load(Relaxed) && !to_watch.is_empty() {
        to_watch.retain(|epg| {
            let is_timestamp_over = match is_timestamp_over(epg.start_timestamp) {
                Ok(v) => v,
                Err(e) => {
                    log::log(format!("{:?}", e));
                    return false;
                }
            };
            if is_timestamp_over {
                match notify(epg).context("Failed to notify EPG") {
                    Ok(_) => {}
                    Err(e) => log::log(format!("{:?}", e)),
                }
                return false;
            }
            return true;
        });
        sleep(Duration::from_secs(1));
    }
    Ok(())
}

fn notify(epg: &EPGNotify) -> Result<()> {
    notify_rust::Notification::new()
        .summary(&format!("LIVE: {}", epg.title))
        .body(&format!("Watch on {}", epg.channel_name))
        .show()?;
    Ok(())
}

fn is_timestamp_over(timestamp: i64) -> Result<bool> {
    let time = utils::get_local_time(timestamp)?;
    let current_time = Local::now();
    Ok(current_time >= time)
}

pub async fn add_epg(state: &Mutex<AppState>, epg: EPGNotify) -> Result<()> {
    let mut state = state.lock().await;
    if state.thread_handle.is_some() {
        state.notify_stop.store(true, Relaxed);
        let _ = state
            .thread_handle
            .take()
            .context("no thread in option")?
            .join();
    }
    state.notify_stop.store(false, Relaxed);
    let stop = state.notify_stop.clone();
    sql::clean_epgs()?;
    sql::add_epg(epg)?;
    let list = sql::get_epgs()?;
    state
        .thread_handle
        .replace(thread::spawn(|| poll(list, stop)));
    Ok(())
}

pub async fn remove_epg(state: &Mutex<AppState>, epg_id: String) -> Result<()> {
    let mut state = state.lock().await;
    if state.thread_handle.is_some() {
        state.notify_stop.store(true, Relaxed);
        let _ = state
            .thread_handle
            .take()
            .context("no thread in option")?
            .join();
    }
    state.notify_stop.store(false, Relaxed);
    let stop = state.notify_stop.clone();
    sql::clean_epgs()?;
    sql::remove_epg(epg_id)?;
    let list = sql::get_epgs()?;
    if list.len() == 0 {
        return Ok(());
    }
    state
        .thread_handle
        .replace(thread::spawn(|| poll(list, stop)));
    Ok(())
}

pub async fn on_start_check_epg(state: &Mutex<AppState>) -> Result<()> {
    sql::clean_epgs()?;
    let list = sql::get_epgs()?;
    if list.len() == 0 {
        return Ok(());
    }
    let mut state = state.lock().await;
    state.notify_stop.store(false, Relaxed);
    let stop = state.notify_stop.clone();
    state
        .thread_handle
        .replace(thread::spawn(|| poll(list, stop)));
    Ok(())
}
