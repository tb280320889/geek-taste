//! notification_adapter — 桌面通知

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub fn send_high_signal_notification(
    app: &AppHandle,
    repo_full_name: &str,
    signal_type_text: &str,
    title: &str,
) -> Result<(), String> {
    let body = format!("{}: {} — {}", repo_full_name, signal_type_text, title);

    app.notification()
        .builder()
        .title("Geek Taste")
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}
