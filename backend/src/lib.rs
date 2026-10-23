use std::collections::HashMap;

use anyhow::Error;
use tokio::sync::Mutex;
use types::{
    AppState, Channel, CustomChannel, CustomChannelExtraData, EPG, EPGNotify, Filters, Group,
    IdName, NetworkInfo, Settings, Source,
};

pub mod bulk_action_type;
pub mod epg;
pub mod log;
pub mod m3u;
pub mod media_type;
pub mod mpv;
pub mod paths;
pub mod restream;
pub mod settings;
pub mod share;
pub mod sort_type;
pub mod source_type;
pub mod sql;
pub mod types;
pub mod utils;
pub mod view_type;
pub mod xtream;

pub fn map_err_frontend(e: Error) -> String {
    log::log(format!("{e:?}"));
    format!("{e:?}")
}

pub fn initialize_database() -> Result<(), String> {
    sql::create_or_initialize_db().map_err(map_err_frontend)
}

pub fn get_m3u8(source: Source) -> Result<(), String> {
    m3u::read_m3u8(source, false).map_err(map_err_frontend)
}

pub async fn get_m3u8_from_link(source: Source) -> Result<(), String> {
    m3u::get_m3u8_from_link(source, false)
        .await
        .map_err(map_err_frontend)
}

pub async fn play(
    channel: Channel,
    record: bool,
    record_path: Option<String>,
    state: &Mutex<AppState>,
) -> Result<(), String> {
    mpv::play(channel, record, record_path, state)
        .await
        .map_err(map_err_frontend)
}

pub fn get_settings() -> Result<Settings, String> {
    settings::get_settings().map_err(map_err_frontend)
}

pub fn update_settings(settings: Settings) -> Result<(), String> {
    settings::update_settings(settings).map_err(map_err_frontend)
}

pub fn search(filters: Filters) -> Result<Vec<Channel>, String> {
    sql::search(filters).map_err(map_err_frontend)
}

pub fn bulk_update(filters: Filters, action: u8) -> Result<(), String> {
    sql::bulk_update(filters, action).map_err(map_err_frontend)
}

pub async fn get_xtream(source: Source) -> Result<(), String> {
    xtream::get_xtream(source, false)
        .await
        .map_err(map_err_frontend)
}

pub async fn refresh_source(source: Source) -> Result<(), String> {
    utils::refresh_source(source)
        .await
        .map_err(map_err_frontend)
}

pub async fn refresh_all() -> Result<(), String> {
    utils::refresh_all().await.map_err(map_err_frontend)
}

pub async fn get_episodes(channel: Channel) -> Result<(), String> {
    xtream::get_episodes(channel)
        .await
        .map_err(map_err_frontend)
}

pub fn favorite_channel(channel_id: i64) -> Result<(), String> {
    sql::favorite_channel(channel_id, true).map_err(map_err_frontend)
}

pub fn unfavorite_channel(channel_id: i64) -> Result<(), String> {
    sql::favorite_channel(channel_id, false).map_err(map_err_frontend)
}

pub fn hide_channel(id: i64, hidden: bool) -> Result<(), String> {
    sql::hide_channel(id, hidden).map_err(map_err_frontend)
}

pub fn hide_group(id: i64, hidden: bool) -> Result<(), String> {
    sql::hide_group(id, hidden).map_err(map_err_frontend)
}

pub fn remove_from_history(id: i64) -> Result<(), String> {
    sql::remove_last_watched(id).map_err(map_err_frontend)
}

pub fn source_name_exists(name: String) -> Result<bool, String> {
    sql::source_name_exists(&name).map_err(map_err_frontend)
}

pub fn get_sources() -> Result<Vec<Source>, String> {
    sql::get_sources().map_err(map_err_frontend)
}

pub fn get_enabled_sources() -> Result<Vec<Source>, String> {
    sql::get_enabled_sources().map_err(map_err_frontend)
}

pub fn delete_source(id: i64) -> Result<(), String> {
    sql::delete_source(id).map_err(map_err_frontend)
}

pub fn toggle_source(value: bool, source_id: i64) -> Result<(), String> {
    sql::set_source_enabled(value, source_id).map_err(map_err_frontend)
}

pub fn delete_database() -> Result<(), String> {
    utils::create_nuke_request().map_err(map_err_frontend)
}

pub fn add_custom_channel(channel: CustomChannel) -> Result<(), String> {
    sql::do_tx(|tx| sql::add_custom_channel(tx, channel)).map_err(map_err_frontend)
}

pub fn edit_custom_channel(channel: CustomChannel) -> Result<(), String> {
    sql::edit_custom_channel(channel).map_err(map_err_frontend)
}

pub fn delete_custom_channel(id: i64) -> Result<(), String> {
    sql::delete_custom_channel(id).map_err(map_err_frontend)
}

pub fn get_custom_channel_extra_data(
    id: i64,
    group_id: Option<i64>,
) -> Result<CustomChannelExtraData, String> {
    sql::get_custom_channel_extra_data(id, group_id).map_err(map_err_frontend)
}

pub fn add_custom_source(name: String) -> Result<(), String> {
    sql::do_tx(|tx| sql::create_or_find_source_by_name(tx, &mut sql::get_custom_source(name)))
        .map_err(map_err_frontend)?;
    Ok(())
}

pub fn share_custom_channel(channel: Channel, path: String) -> Result<(), String> {
    share::share_custom_channel(channel, path).map_err(map_err_frontend)
}

pub fn group_auto_complete(query: Option<String>, source_id: i64) -> Result<Vec<IdName>, String> {
    sql::group_auto_complete(query, source_id).map_err(map_err_frontend)
}

pub fn edit_custom_group(group: Group) -> Result<(), String> {
    sql::edit_custom_group(group).map_err(map_err_frontend)
}

pub fn add_custom_group(group: Group) -> Result<(), String> {
    sql::do_tx(|tx| {
        sql::add_custom_group(tx, group)?;
        Ok(())
    })
    .map_err(map_err_frontend)
}

pub fn delete_custom_group(
    id: i64,
    new_id: Option<i64>,
    do_channels_update: bool,
) -> Result<(), String> {
    sql::delete_custom_group(id, new_id, do_channels_update).map_err(map_err_frontend)
}

pub fn group_not_empty(id: i64) -> Result<bool, String> {
    sql::group_not_empty(id).map_err(map_err_frontend)
}

pub fn group_exists(name: String, source_id: i64) -> Result<bool, String> {
    sql::group_exists(&name, source_id).map_err(map_err_frontend)
}

pub fn share_custom_group(group: Channel, path: String) -> Result<(), String> {
    share::share_custom_group(group, path).map_err(map_err_frontend)
}

pub fn share_custom_source(source: Source, path: String) -> Result<(), String> {
    share::share_custom_source(source, path).map_err(map_err_frontend)
}

pub fn import(
    path: String,
    source_id: Option<i64>,
    name_override: Option<String>,
) -> Result<(), String> {
    share::import(path, source_id, name_override).map_err(map_err_frontend)
}

pub fn channel_exists(name: String, url: String, source_id: i64) -> Result<bool, String> {
    sql::channel_exists(&name, &url, source_id).map_err(map_err_frontend)
}

pub fn update_source(source: Source) -> Result<(), String> {
    sql::update_source(source).map_err(map_err_frontend)
}

pub async fn get_epg(channel: Channel) -> Result<Vec<EPG>, String> {
    xtream::get_epg(channel).await.map_err(map_err_frontend)
}

pub async fn download(
    state: &Mutex<AppState>,
    progress: tokio::sync::mpsc::UnboundedSender<f64>,
    channel: Channel,
    download_id: String,
    path: Option<String>,
) -> Result<(), String> {
    utils::download(state, progress, channel, &download_id, path)
        .await
        .map_err(map_err_frontend)
}

pub async fn abort_download(
    state: &Mutex<AppState>,
    source_id: i64,
    download_id: String,
) -> Result<(), String> {
    mpv::cancel_play(source_id, download_id, state)
        .await
        .map_err(map_err_frontend)
}

pub async fn add_epg(state: &Mutex<AppState>, epg: EPGNotify) -> Result<(), String> {
    epg::add_epg(state, epg).await.map_err(map_err_frontend)
}

pub async fn remove_epg(state: &Mutex<AppState>, epg_id: String) -> Result<(), String> {
    epg::remove_epg(state, epg_id)
        .await
        .map_err(map_err_frontend)
}

pub fn get_epg_ids() -> Result<Vec<String>, String> {
    sql::get_epg_ids().map_err(map_err_frontend)
}

pub async fn on_start_check_epg(state: &Mutex<AppState>) -> Result<(), String> {
    epg::on_start_check_epg(state)
        .await
        .map_err(map_err_frontend)
}

pub async fn start_restream(
    port: u16,
    state: &Mutex<AppState>,
    on_started: tokio::sync::oneshot::Sender<()>,
    channel: Channel,
) -> Result<(), String> {
    crate::restream::start_restream(port, state, on_started, channel)
        .await
        .map_err(map_err_frontend)
}

pub async fn stop_restream(state: &Mutex<AppState>) -> Result<(), String> {
    crate::restream::stop_restream(state)
        .await
        .map_err(map_err_frontend)
}

pub async fn watch_self(port: u16, state: &Mutex<AppState>) -> Result<(), String> {
    restream::watch_self(port, state)
        .await
        .map_err(map_err_frontend)
}

pub async fn get_network_info() -> Result<NetworkInfo, String> {
    restream::get_network_info().await.map_err(map_err_frontend)
}

pub fn share_restream(address: String, channel: Channel, path: String) -> Result<(), String> {
    restream::share_restream(address, channel, path).map_err(map_err_frontend)
}

pub fn add_last_watched(id: i64) -> Result<(), String> {
    sql::add_last_watched(id).map_err(map_err_frontend)
}

pub fn backup_favs(id: i64, path: String) -> Result<(), String> {
    utils::backup_favs(id, path).map_err(map_err_frontend)
}

pub fn restore_favs(id: i64, path: String) -> Result<(), String> {
    utils::restore_favs(id, path).map_err(map_err_frontend)
}

pub fn clear_history() -> Result<(), String> {
    sql::clear_history().map_err(map_err_frontend)
}

pub fn is_container() -> Result<bool, String> {
    Ok(utils::is_container())
}

pub async fn cancel_play(
    source_id: i64,
    channel_id: i64,
    state: &Mutex<AppState>,
) -> Result<(), String> {
    mpv::cancel_play(source_id, channel_id.to_string(), state)
        .await
        .map_err(map_err_frontend)
}

pub async fn get_all_expiries() -> Result<HashMap<i64, i64>, String> {
    xtream::get_all_expiries().await.map_err(map_err_frontend)
}
