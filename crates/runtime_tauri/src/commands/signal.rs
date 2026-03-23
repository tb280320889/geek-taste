//! Signal Tauri IPC 命令 — 薄封装层

use shared_contracts::signal_dto::{SignalDto, UnreadCountsDto};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use super::helpers::get_db_connection;

const SETTINGS_KEY: &str = "settings";

#[tauri::command]
pub async fn list_signals(
    app: AppHandle,
    filter_state: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<SignalDto>, String> {
    let conn = get_db_connection(&app)?;
    application::signal::list_signals(&conn, filter_state, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_home_signals(
    app: AppHandle,
    since: Option<String>,
) -> Result<Vec<SignalDto>, String> {
    let conn = get_db_connection(&app)?;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let settings: shared_contracts::settings_dto::SettingsDto = match store.get(SETTINGS_KEY) {
        Some(val) => serde_json::from_value(val).map_err(|e| e.to_string())?,
        None => {
            shared_contracts::settings_dto::SettingsDto::from(domain::settings::Settings::default())
        }
    };

    application::signal::list_home_signals(&conn, since.as_deref(), &settings.language_interests)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ack_signal(app: AppHandle, signal_id: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    application::signal::ack_signal(&conn, &signal_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mark_signal_seen(app: AppHandle, signal_id: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    application::signal::mark_seen(&conn, &signal_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_unread_counts(app: AppHandle) -> Result<UnreadCountsDto, String> {
    let conn = get_db_connection(&app)?;
    application::signal::get_unread_counts(&conn).map_err(|e| e.to_string())
}
