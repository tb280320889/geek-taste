//! 同步状态 Tauri 命令 — 返回各数据源最后同步时间

use serde::Serialize;
use tauri::AppHandle;

use super::helpers::get_db_connection;

#[derive(Debug, Clone, Serialize)]
pub struct SyncStatusDto {
    pub is_online: bool,
    pub last_topk_sync: Option<String>,
    pub last_signal_sync: Option<String>,
}

#[tauri::command]
pub async fn get_sync_status(app: AppHandle) -> Result<SyncStatusDto, String> {
    let conn = get_db_connection(&app)?;
    let last_topk = persistence_sqlite::ranking_repository::get_last_snapshot_time(&conn).ok();
    let last_signal = persistence_sqlite::signal_repository::get_last_signal_time(&conn).ok();
    // is_online 由前端检测后通过 IPC 更新，后端默认 true
    Ok(SyncStatusDto {
        is_online: true,
        last_topk_sync: last_topk.flatten(),
        last_signal_sync: last_signal.flatten(),
    })
}
