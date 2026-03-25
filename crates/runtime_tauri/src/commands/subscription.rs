//! Subscription Tauri IPC 命令 — 薄封装层
//!
//! 重构说明：同步加载 → 异步获取 GitHub 数据 → 同步处理，避免 conn 跨 .await。

use shared_contracts::subscription_dto::{CreateSubscriptionRequest, SubscriptionRowDto};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use super::helpers::{get_db_connection, github_api_enabled, load_token};

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
    if !github_api_enabled(&app)? {
        // 测试模式：禁用远程同步，返回 0（不报错）
        return Ok(0);
    }

    // 获取 settings（需要 store，在 conn 之前获取）
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let settings: shared_contracts::settings_dto::SettingsDto = match store.get(SETTINGS_KEY) {
        Some(val) => serde_json::from_value(val).map_err(|e| e.to_string())?,
        None => {
            shared_contracts::settings_dto::SettingsDto::from(domain::settings::Settings::default())
        }
    };

    // 同步：加载上下文（conn 在此作用域内）
    let pairs = {
        let conn = get_db_connection(&app)?;
        application::subscription::load_sync_context(&conn).map_err(|e| e.to_string())?
    };
    // conn 已释放

    // 异步：获取 GitHub 更新（不持有 conn）
    let token = load_token()?;
    let fetch_results = application::subscription::fetch_all_updates(&token, &pairs).await;

    // 同步：处理结果，生成信号，保存到 DB
    let conn2 = get_db_connection(&app)?;
    let (synced, notifications) =
        application::subscription::process_sync_results(&conn2, &pairs, &fetch_results, &settings)
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
