//! Subscription Tauri IPC 命令 — 薄封装层

use shared_contracts::subscription_dto::{CreateSubscriptionRequest, SubscriptionRowDto};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use super::helpers::{get_db_connection, load_token};

const SETTINGS_KEY: &str = "settings";

#[tauri::command]
pub async fn subscribe(
    app: AppHandle,
    repo_id: i64,
    tracking_mode: Option<String>,
    event_types: Option<Vec<String>>,
    digest_window: Option<String>,
    notify_high_immediately: Option<bool>,
) -> Result<SubscriptionRowDto, String> {
    let conn = get_db_connection(&app)?;
    let request = CreateSubscriptionRequest {
        repo_id,
        tracking_mode,
        event_types,
        digest_window,
        notify_high_immediately,
    };
    let sub = application::subscription::subscribe(&conn, repo_id, &request)
        .map_err(|e| e.to_string())?;

    // 返回完整 DTO（含 repo 信息）
    let list = application::subscription::list_subscriptions(&conn).map_err(|e| e.to_string())?;
    list.into_iter()
        .find(|d| d.subscription_id == sub.subscription_id)
        .ok_or_else(|| "Subscription created but not found in list".to_string())
}

#[tauri::command]
pub async fn unsubscribe(app: AppHandle, subscription_id: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    application::subscription::unsubscribe(&conn, &subscription_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_subscription(app: AppHandle, subscription_id: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    application::subscription::pause_subscription(&conn, &subscription_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_subscriptions(app: AppHandle) -> Result<Vec<SubscriptionRowDto>, String> {
    let conn = get_db_connection(&app)?;
    application::subscription::list_subscriptions(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_subscriptions(app: AppHandle) -> Result<usize, String> {
    let conn = get_db_connection(&app)?;
    let token = load_token()?;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let settings: shared_contracts::settings_dto::SettingsDto = match store.get(SETTINGS_KEY) {
        Some(val) => serde_json::from_value(val).map_err(|e| e.to_string())?,
        None => {
            shared_contracts::settings_dto::SettingsDto::from(domain::settings::Settings::default())
        }
    };

    let (synced, notifications) =
        application::subscription::sync_subscriptions(&conn, &token, &settings)
            .await
            .map_err(|e| e.to_string())?;

    for item in &notifications {
        notification_adapter::send_high_signal_notification(
            &app,
            &item.repo_full_name,
            &item.signal_type_text,
            &item.title,
        )?;
    }

    Ok(synced)
}
