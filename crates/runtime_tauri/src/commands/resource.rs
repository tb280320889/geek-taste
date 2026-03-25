//! Resource Tauri IPC 命令 — 薄封装层

use domain::resource::ResourceKind;
use shared_contracts::resource_dto::{CurateResourceRequest, ResourceCardDto, ResourceListRequest};
use tauri::AppHandle;

use super::helpers::get_db_connection;

#[tauri::command]
pub async fn list_resources(
    app: AppHandle,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<ResourceCardDto>, String> {
    let conn = get_db_connection(&app)?;
    application::resource::list_resources(&conn, limit.unwrap_or(50), offset.unwrap_or(0))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_resources(
    app: AppHandle,
    request: ResourceListRequest,
) -> Result<Vec<ResourceCardDto>, String> {
    let conn = get_db_connection(&app)?;
    let kind = request
        .resource_kind
        .and_then(|k| ResourceKind::from_str(&k));
    application::resource::search_resources(
        &conn,
        request.tag_type.as_deref(),
        request.tag_value.as_deref(),
        kind,
        request.language.as_deref(),
        request.limit.unwrap_or(50),
        request.offset.unwrap_or(0),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn curate_resource(
    app: AppHandle,
    request: CurateResourceRequest,
) -> Result<ResourceCardDto, String> {
    let conn = get_db_connection(&app)?;
    application::resource::curate_resource(&conn, &request.resource_id, &request.action)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn deactivate_resource(app: AppHandle, resource_id: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    application::resource::deactivate_resource(&conn, &resource_id).map_err(|e| e.to_string())
}
