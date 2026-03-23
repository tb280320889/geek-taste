use keyring::Entry;
use shared_contracts::auth_dto::{UserDto, ValidateTokenResponse};
use shared_contracts::repo_dto::RepoBasicInfo;
use tauri::AppHandle;

const SERVICE: &str = "geek-taste";
const TOKEN_KEY: &str = "github-pat";

#[tauri::command]
pub async fn validate_github_token(token: String) -> Result<ValidateTokenResponse, String> {
    match github_adapter::auth::validate_token(&token).await {
        Ok(user) => Ok(ValidateTokenResponse {
            success: true,
            user: Some(UserDto::from(user)),
            error: None,
        }),
        Err(e) => Ok(ValidateTokenResponse {
            success: false,
            user: None,
            error: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn store_github_token(_app: AppHandle, token: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE, TOKEN_KEY).map_err(|e| e.to_string())?;
    entry.set_password(&token).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn load_github_token() -> Result<String, String> {
    let entry = Entry::new(SERVICE, TOKEN_KEY).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_github_token() -> Result<(), String> {
    let entry = Entry::new(SERVICE, TOKEN_KEY).map_err(|e| e.to_string())?;
    entry.delete_credential().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_current_user() -> Result<Option<UserDto>, String> {
    let entry = Entry::new(SERVICE, TOKEN_KEY).map_err(|e| e.to_string())?;
    let token = match entry.get_password() {
        Ok(t) => t,
        Err(keyring::Error::NoEntry) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };

    match github_adapter::auth::validate_token(&token).await {
        Ok(user) => Ok(Some(UserDto::from(user))),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub async fn fetch_repo_info(owner: String, repo: String) -> Result<RepoBasicInfo, String> {
    let entry = Entry::new(SERVICE, TOKEN_KEY).map_err(|e| e.to_string())?;
    let token = entry.get_password().map_err(|e| e.to_string())?;
    github_adapter::auth::fetch_repo_info(&token, &owner, &repo).await
}
