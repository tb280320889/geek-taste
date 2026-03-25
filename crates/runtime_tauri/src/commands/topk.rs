//! TopK Tauri IPC 命令 — 薄封装层
//!
//! 重构说明：rusqlite::Connection 不是 Sync，不能持有 &conn 跨 .await 点。
//! 策略：同步读 DB → 释放 conn → .await API 调用 → 同步写 DB。

use shared_contracts::ranking_dto::{
    CreateRankingViewRequest, RankingResultDto, RankingViewSpecDto,
};
use tauri::AppHandle;

use super::helpers::{get_db_connection, github_api_enabled, load_token};

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
    // 同步：创建视图
    let view_id = {
        let conn = get_db_connection(&app)?;
        let dto = application::topk::create_view(&conn, request).map_err(|e| e.to_string())?;
        dto.ranking_view_id.clone()
    };

    // API 关闭时：仅创建视图，不触发 GitHub 预热请求
    if !github_api_enabled(&app)? {
        let conn3 = get_db_connection(&app)?;
        let views = application::topk::list_views(&conn3).map_err(|e| e.to_string())?;
        return views
            .into_iter()
            .find(|v| v.ranking_view_id == view_id)
            .ok_or_else(|| "View not found after creation".to_string());
    }

    // 同步：加载上下文（在 await 前释放连接）
    let (view, prev_snapshot, repo_snapshots) = {
        let conn2 = get_db_connection(&app)?;
        application::topk::load_ranking_context(&conn2, &view_id).map_err(|e| e.to_string())?
    };

    // 暖机：异步获取候选 + 同步计算 + 同步保存快照
    let token = load_token()?;
    match application::topk::fetch_ranking_candidates(&token, &view).await {
        Ok(items) => {
            let conn2 = get_db_connection(&app)?;
            let result = application::topk::compute_ranking_result(
                &conn2,
                &view,
                items,
                prev_snapshot.as_ref(),
                &repo_snapshots,
            )
            .map_err(|e| e.to_string());
            if let Ok(ref r) = result {
                let _ = application::topk::create_snapshot(&conn2, &view_id, &r.items);
            }
        }
        Err(e) => {
            tracing::warn!("Warm-up snapshot failed for view {}: {}", view_id, e);
        }
    }

    // 返回创建的视图
    let conn3 = get_db_connection(&app)?;
    let views = application::topk::list_views(&conn3).map_err(|e| e.to_string())?;
    views
        .into_iter()
        .find(|v| v.ranking_view_id == view_id)
        .ok_or_else(|| "View not found after creation".to_string())
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
pub async fn execute_ranking(app: AppHandle, view_id: String) -> Result<RankingResultDto, String> {
    // API 关闭：从本地缓存构建结果，避免触发 GitHub 限流
    if !github_api_enabled(&app)? {
        let conn = get_db_connection(&app)?;
        return application::topk::execute_ranking_from_cache(&conn, &view_id)
            .map_err(|e| e.to_string());
    }

    // 同步：加载上下文（conn 在此作用域内）
    let (view, prev_snapshot, repo_snapshots) = {
        let conn = get_db_connection(&app)?;
        application::topk::load_ranking_context(&conn, &view_id).map_err(|e| e.to_string())?
    };
    // conn 已释放

    // 异步：调用 GitHub API（不持有 conn）
    let token = load_token()?;
    let items = application::topk::fetch_ranking_candidates(&token, &view)
        .await
        .map_err(|e| e.to_string())?;

    // 同步：计算排名 + 保存快照 + 排名变化
    let conn2 = get_db_connection(&app)?;
    let mut result = application::topk::compute_ranking_result(
        &conn2,
        &view,
        items,
        prev_snapshot.as_ref(),
        &repo_snapshots,
    )
    .map_err(|e| e.to_string())?;

    // 保存快照
    let _ = application::topk::create_snapshot(&conn2, &view_id, &result.items);

    // 计算排名变化
    let rank_changes = application::topk::get_rank_change(&conn2, &view_id, &result.items)
        .map_err(|e| e.to_string())?;

    for (item, change) in result.items.iter_mut().zip(rank_changes.iter()) {
        item.rank_change = *change;
    }

    Ok(result)
}
