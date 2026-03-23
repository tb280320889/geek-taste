//! desktop-ui-tauri — Tauri application entry point

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            runtime_tauri::commands::validate_github_token,
            runtime_tauri::commands::store_github_token,
            runtime_tauri::commands::load_github_token,
            runtime_tauri::commands::remove_github_token,
            runtime_tauri::commands::get_current_user,
            runtime_tauri::commands::fetch_repo_info,
            runtime_tauri::commands::get_settings,
            runtime_tauri::commands::update_settings,
            runtime_tauri::commands::list_ranking_views,
            runtime_tauri::commands::create_ranking_view,
            runtime_tauri::commands::delete_ranking_view,
            runtime_tauri::commands::toggle_pin_ranking_view,
            runtime_tauri::commands::execute_ranking,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
