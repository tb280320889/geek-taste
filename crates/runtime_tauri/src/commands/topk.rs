//! TopK Tauri IPC 命令 — 薄封装层

use keyring::Entry;
use rusqlite::Connection;
use shared_contracts::ranking_dto::{CreateRankingViewRequest, RankingItemDto, RankingViewSpecDto};
use tauri::{AppHandle, Manager};

const SERVICE: &str = "geek-taste";
const TOKEN_KEY: &str = "github-pat";

/// 获取 DB 连接（每次调用打开独立连接，WAL 模式支持并发）
fn get_db_connection(app: &AppHandle) -> Result<Connection, String> {
    let db_path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("geek-taste.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).ok();
    let mut conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    persistence_sqlite::init_db(&mut conn).map_err(|e| e.to_string())?;
    Ok(conn)
}

/// 从 keyring 加载 GitHub token
fn load_token() -> Result<String, String> {
    let entry = Entry::new(SERVICE, TOKEN_KEY).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_ranking_views(app: AppHandle) -> Result<Vec<RankingViewSpecDto>, String> {
    let conn = get_db_connection(&app)?;
    application::topk::list_views(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_ranking_view(
    app: AppHandle,
    request: CreateRankingViewRequest,
) -> Result<RankingViewSpecDto, String> {
    let conn = get_db_connection(&app)?;
    let dto = application::topk::create_view(&conn, request).map_err(|e| e.to_string())?;

    // 暖机快照：创建视图后立即触发首次排名
    let token = load_token()?;
    match application::topk::execute_ranking(&conn, &token, &dto.ranking_view_id).await {
        Ok(items) => {
            let _ = application::topk::create_snapshot(&conn, &dto.ranking_view_id, &items);
        }
        Err(e) => {
            tracing::warn!(
                "Warm-up snapshot failed for view {}: {}",
                dto.ranking_view_id,
                e
            );
        }
    }

    Ok(dto)
}

#[tauri::command]
pub async fn delete_ranking_view(app: AppHandle, view_id: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    application::topk::delete_view(&conn, &view_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_pin_ranking_view(app: AppHandle, view_id: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    application::topk::toggle_pin_view(&conn, &view_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_ranking(
    app: AppHandle,
    view_id: String,
) -> Result<Vec<RankingItemDto>, String> {
    let conn = get_db_connection(&app)?;
    let token = load_token()?;
    let items = application::topk::execute_ranking(&conn, &token, &view_id)
        .await
        .map_err(|e| e.to_string())?;

    // 执行后自动保存快照
    let _ = application::topk::create_snapshot(&conn, &view_id, &items);

    // 计算排名变化
    let rank_changes =
        application::topk::get_rank_change(&conn, &view_id, &items).map_err(|e| e.to_string())?;

    // 将排名变化合并到 DTO 中
    let mut result = items;
    for (item, change) in result.iter_mut().zip(rank_changes.iter()) {
        item.rank_change = *change;
    }

    Ok(result)
}
