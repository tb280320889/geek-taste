use shared_contracts::settings_dto::{SettingsDto, UpdateSettingsRequest};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const SETTINGS_KEY: &str = "settings";

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<SettingsDto, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    match store.get(SETTINGS_KEY) {
        Some(val) => {
            let settings: SettingsDto = serde_json::from_value(val).map_err(|e| e.to_string())?;
            Ok(settings)
        }
        None => Ok(SettingsDto::from(domain::settings::Settings::default())),
    }
}

#[tauri::command]
pub async fn update_settings(
    app: AppHandle,
    settings: UpdateSettingsRequest,
) -> Result<SettingsDto, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    let mut current: SettingsDto = match store.get(SETTINGS_KEY) {
        Some(val) => serde_json::from_value(val).map_err(|e| e.to_string())?,
        None => SettingsDto::from(domain::settings::Settings::default()),
    };

    if let Some(nf) = settings.notification_frequency {
        current.notification_frequency = nf;
    }
    if let Some(li) = settings.language_interests {
        current.language_interests = li;
    }
    if let Some(qh) = settings.quiet_hours {
        current.quiet_hours = qh;
    }

    let val = serde_json::to_value(&current).map_err(|e| e.to_string())?;
    store.set(SETTINGS_KEY, val);

    Ok(current)
}
