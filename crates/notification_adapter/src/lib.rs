//! notification_adapter — 桌面通知

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// 发送高信号通知（Release、Tag 等重要变更）
pub fn send_high_signal_notification(
    app: &AppHandle,
    repo_full_name: &str,
    signal_type: &str,
    title: &str,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(format!("[{}] {}", repo_full_name, signal_type))
        .body(title)
        .show()
        .map_err(|e| e.to_string())?;
    Ok(())
}
