//! desktop-ui-tauri — Tauri application entry point

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::init())
        .invoke_handler(tauri::generate_handler![
            // auth + settings
            runtime_tauri::commands::validate_github_token,
            runtime_tauri::commands::store_github_token,
            runtime_tauri::commands::load_github_token,
            runtime_tauri::commands::remove_github_token,
            runtime_tauri::commands::get_current_user,
            runtime_tauri::commands::fetch_repo_info,
            runtime_tauri::commands::get_settings,
            runtime_tauri::commands::update_settings,
            // topk
            runtime_tauri::commands::list_ranking_views,
            runtime_tauri::commands::create_ranking_view,
            runtime_tauri::commands::delete_ranking_view,
            runtime_tauri::commands::toggle_pin_ranking_view,
            runtime_tauri::commands::execute_ranking,
            // signal
            runtime_tauri::commands::list_signals,
            runtime_tauri::commands::list_home_signals,
            runtime_tauri::commands::ack_signal,
            runtime_tauri::commands::mark_signal_seen,
            runtime_tauri::commands::get_unread_counts,
            // subscription
            runtime_tauri::commands::subscribe,
            runtime_tauri::commands::unsubscribe,
            runtime_tauri::commands::pause_subscription,
            runtime_tauri::commands::list_subscriptions,
            runtime_tauri::commands::sync_subscriptions,
            // resource
            runtime_tauri::commands::list_resources,
            runtime_tauri::commands::search_resources,
            runtime_tauri::commands::curate_resource,
            runtime_tauri::commands::deactivate_resource,
            // sync_status
            runtime_tauri::commands::get_sync_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
