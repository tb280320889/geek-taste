//! RankingView / RankingSnapshot 的 SQLite CRUD 实现

use anyhow::Result;
use domain::ranking::{
    RankingFilters, RankingMode, RankingSnapshot, RankingSnapshotItem, RankingView, SnapshotStats,
};
use rusqlite::{params, Connection};
use serde_rusqlite::from_row;

/// 创建榜单视图
pub fn create_ranking_view(conn: &Connection, view: &RankingView) -> Result<RankingView> {
    let filters_json = serde_json::to_string(&view.filters)?;
    conn.execute(
        "INSERT INTO ranking_views
         (ranking_view_id, name, view_kind, query_template, filters_json, ranking_mode,
          k_value, is_pinned, created_at, updated_at, last_snapshot_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            view.ranking_view_id,
            view.name,
            view.view_kind,
            view.query_template,
            filters_json,
            view.ranking_mode.to_string(),
            view.k_value,
            view.is_pinned as i32,
            view.created_at,
            view.updated_at,
            view.last_snapshot_at,
        ],
    )?;
    Ok(view.clone())
}

/// 更新榜单视图
pub fn update_ranking_view(conn: &Connection, view: &RankingView) -> Result<RankingView> {
    let filters_json = serde_json::to_string(&view.filters)?;
    conn.execute(
        "UPDATE ranking_views SET name = ?, view_kind = ?, query_template = ?,
         filters_json = ?, ranking_mode = ?, k_value = ?, is_pinned = ?,
         updated_at = ?, last_snapshot_at = ?
         WHERE ranking_view_id = ?",
        params![
            view.name,
            view.view_kind,
            view.query_template,
            filters_json,
            view.ranking_mode.to_string(),
            view.k_value,
            view.is_pinned as i32,
            view.updated_at,
            view.last_snapshot_at,
            view.ranking_view_id,
        ],
    )?;
    Ok(view.clone())
}

/// 删除榜单视图（同时删除关联的 snapshots）
pub fn delete_ranking_view(conn: &Connection, view_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM ranking_snapshots WHERE ranking_view_id = ?",
        params![view_id],
    )?;
    conn.execute(
        "DELETE FROM ranking_views WHERE ranking_view_id = ?",
        params![view_id],
    )?;
    Ok(())
}

/// 列出所有榜单视图（pinned 优先，按 updated_at 降序）
pub fn list_ranking_views(conn: &Connection) -> Result<Vec<RankingView>> {
    let mut stmt = conn.prepare(
        "SELECT ranking_view_id, name, view_kind, query_template, filters_json,
                ranking_mode, k_value, is_pinned, created_at, updated_at, last_snapshot_at
         FROM ranking_views ORDER BY is_pinned DESC, updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let r: RankingViewRow = from_row(row).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;
        Ok(r)
    })?;

    let mut results = Vec::new();
    for row in rows {
        let r = row?;
        results.push(row_to_view(r)?);
    }
    Ok(results)
}

/// 获取单个榜单视图
pub fn get_ranking_view(conn: &Connection, view_id: &str) -> Result<Option<RankingView>> {
    let mut stmt = conn.prepare(
        "SELECT ranking_view_id, name, view_kind, query_template, filters_json,
                ranking_mode, k_value, is_pinned, created_at, updated_at, last_snapshot_at
         FROM ranking_views WHERE ranking_view_id = ?",
    )?;
    let mut rows = stmt.query(params![view_id])?;
    if let Some(row) = rows.next()? {
        let r: RankingViewRow = from_row(row)?;
        Ok(Some(row_to_view(r)?))
    } else {
        Ok(None)
    }
}

/// 保存 ranking snapshot（同时更新 ranking_views.last_snapshot_at）
pub fn save_ranking_snapshot(
    conn: &Connection,
    snapshot: &RankingSnapshot,
) -> Result<RankingSnapshot> {
    let items_json = serde_json::to_string(&snapshot.items)?;
    let stats_json = serde_json::to_string(&snapshot.stats)?;

    conn.execute(
        "INSERT INTO ranking_snapshots
         (ranking_snapshot_id, ranking_view_id, snapshot_at, ranking_mode, items_json, stats_json)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            snapshot.ranking_snapshot_id,
            snapshot.ranking_view_id,
            snapshot.snapshot_at,
            snapshot.ranking_mode.to_string(),
            items_json,
            stats_json,
        ],
    )?;

    // 更新 ranking_views.last_snapshot_at
    conn.execute(
        "UPDATE ranking_views SET last_snapshot_at = ? WHERE ranking_view_id = ?",
        params![snapshot.snapshot_at, snapshot.ranking_view_id],
    )?;

    Ok(snapshot.clone())
}

/// 获取最新的 ranking snapshot
pub fn get_latest_ranking_snapshot(
    conn: &Connection,
    view_id: &str,
) -> Result<Option<RankingSnapshot>> {
    let mut stmt = conn.prepare(
        "SELECT ranking_snapshot_id, ranking_view_id, snapshot_at, ranking_mode,
                items_json, stats_json
         FROM ranking_snapshots WHERE ranking_view_id = ?
         ORDER BY snapshot_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query(params![view_id])?;
    if let Some(row) = rows.next()? {
        let r: SnapshotRow = from_row(row)?;
        let items: Vec<RankingSnapshotItem> = serde_json::from_str(&r.items_json)?;
        let stats: SnapshotStats = serde_json::from_str(&r.stats_json)?;
        let ranking_mode = RankingMode::from_str(&r.ranking_mode)
            .ok_or_else(|| anyhow::anyhow!("Invalid ranking_mode: {}", r.ranking_mode))?;
        Ok(Some(RankingSnapshot {
            ranking_snapshot_id: r.ranking_snapshot_id,
            ranking_view_id: r.ranking_view_id,
            snapshot_at: r.snapshot_at,
            ranking_mode,
            items,
            stats,
        }))
    } else {
        Ok(None)
    }
}

/// 切换 pin 状态
pub fn toggle_pin(conn: &Connection, view_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE ranking_views SET is_pinned = NOT is_pinned WHERE ranking_view_id = ?",
        params![view_id],
    )?;
    Ok(())
}

// ── 内部映射结构 ──────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct RankingViewRow {
    ranking_view_id: String,
    name: String,
    view_kind: String,
    query_template: String,
    filters_json: String,
    ranking_mode: String,
    k_value: i32,
    is_pinned: i32,
    created_at: String,
    updated_at: String,
    last_snapshot_at: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SnapshotRow {
    ranking_snapshot_id: String,
    ranking_view_id: String,
    snapshot_at: String,
    ranking_mode: String,
    items_json: String,
    stats_json: String,
}

fn row_to_view(r: RankingViewRow) -> Result<RankingView> {
    let filters: RankingFilters = serde_json::from_str(&r.filters_json)?;
    let ranking_mode = RankingMode::from_str(&r.ranking_mode)
        .ok_or_else(|| anyhow::anyhow!("Invalid ranking_mode: {}", r.ranking_mode))?;
    Ok(RankingView {
        ranking_view_id: r.ranking_view_id,
        name: r.name,
        view_kind: r.view_kind,
        query_template: r.query_template,
        filters,
        ranking_mode,
        k_value: r.k_value,
        is_pinned: r.is_pinned != 0,
        created_at: r.created_at,
        updated_at: r.updated_at,
        last_snapshot_at: r.last_snapshot_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_db;

    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();
        conn
    }

    fn sample_view(id: &str, name: &str) -> RankingView {
        RankingView {
            ranking_view_id: id.into(),
            name: name.into(),
            view_kind: "PRESET".into(),
            query_template: "language:rust stars:>100".into(),
            filters: RankingFilters {
                language: vec!["Rust".into()],
                exclude_archived: true,
                exclude_forks: true,
                min_stars: Some(100),
                updated_since_days: None,
                topic: vec![],
            },
            ranking_mode: RankingMode::StarsDesc,
            k_value: 50,
            is_pinned: false,
            created_at: "2026-03-23T00:00:00Z".into(),
            updated_at: "2026-03-23T00:00:00Z".into(),
            last_snapshot_at: None,
        }
    }

    #[test]
    fn create_and_get_view() {
        let conn = setup_db();
        let view = sample_view("rv_01", "Rust Hot");
        create_ranking_view(&conn, &view).unwrap();

        let got = get_ranking_view(&conn, "rv_01").unwrap().unwrap();
        assert_eq!(got.name, "Rust Hot");
        assert_eq!(got.ranking_mode, RankingMode::StarsDesc);
        assert_eq!(got.filters.language, vec!["Rust"]);
        assert_eq!(got.k_value, 50);
    }

    #[test]
    fn get_view_not_found() {
        let conn = setup_db();
        let got = get_ranking_view(&conn, "nonexistent").unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn update_view() {
        let conn = setup_db();
        let mut view = sample_view("rv_01", "Rust Hot");
        create_ranking_view(&conn, &view).unwrap();

        view.name = "Rust Updated".into();
        view.k_value = 30;
        view.updated_at = "2026-03-23T12:00:00Z".into();
        update_ranking_view(&conn, &view).unwrap();

        let got = get_ranking_view(&conn, "rv_01").unwrap().unwrap();
        assert_eq!(got.name, "Rust Updated");
        assert_eq!(got.k_value, 30);
    }

    #[test]
    fn delete_view_removes_associated_snapshots() {
        let conn = setup_db();
        let view = sample_view("rv_01", "Rust Hot");
        create_ranking_view(&conn, &view).unwrap();

        // 创建关联 snapshot
        let snapshot = sample_snapshot("rv_01", "rs_01");
        save_ranking_snapshot(&conn, &snapshot).unwrap();

        // 删除视图
        delete_ranking_view(&conn, "rv_01").unwrap();

        // 视图和 snapshot 都应删除
        assert!(get_ranking_view(&conn, "rv_01").unwrap().is_none());
        assert!(get_latest_ranking_snapshot(&conn, "rv_01")
            .unwrap()
            .is_none());
    }

    #[test]
    fn list_views_ordered_by_pinned_then_updated() {
        let conn = setup_db();

        let mut v1 = sample_view("rv_01", "First");
        v1.is_pinned = false;
        v1.updated_at = "2026-03-23T10:00:00Z".into();
        create_ranking_view(&conn, &v1).unwrap();

        let mut v2 = sample_view("rv_02", "Pinned");
        v2.is_pinned = true;
        v2.updated_at = "2026-03-23T09:00:00Z".into();
        create_ranking_view(&conn, &v2).unwrap();

        let mut v3 = sample_view("rv_03", "Recent");
        v3.is_pinned = false;
        v3.updated_at = "2026-03-23T11:00:00Z".into();
        create_ranking_view(&conn, &v3).unwrap();

        let list = list_ranking_views(&conn).unwrap();
        assert_eq!(list.len(), 3);
        // pinned first
        assert_eq!(list[0].ranking_view_id, "rv_02");
        // then by updated_at DESC
        assert_eq!(list[1].ranking_view_id, "rv_03");
        assert_eq!(list[2].ranking_view_id, "rv_01");
    }

    fn sample_snapshot(view_id: &str, snap_id: &str) -> RankingSnapshot {
        RankingSnapshot {
            ranking_snapshot_id: snap_id.into(),
            ranking_view_id: view_id.into(),
            snapshot_at: "2026-03-23T06:00:00Z".into(),
            ranking_mode: RankingMode::StarsDesc,
            items: vec![
                RankingSnapshotItem {
                    repo_id: 100,
                    full_name: "owner/top-repo".into(),
                    rank: 1,
                    score: 0.95,
                    is_subscribed: false,
                },
                RankingSnapshotItem {
                    repo_id: 200,
                    full_name: "owner/second-repo".into(),
                    rank: 2,
                    score: 0.80,
                    is_subscribed: true,
                },
            ],
            stats: SnapshotStats {
                total_count: 50,
                new_count: 3,
                changed_count: 7,
            },
        }
    }

    #[test]
    fn save_and_get_snapshot() {
        let conn = setup_db();
        create_ranking_view(&conn, &sample_view("rv_01", "Test")).unwrap();

        let snap = sample_snapshot("rv_01", "rs_01");
        save_ranking_snapshot(&conn, &snap).unwrap();

        let got = get_latest_ranking_snapshot(&conn, "rv_01")
            .unwrap()
            .unwrap();
        assert_eq!(got.ranking_snapshot_id, "rs_01");
        assert_eq!(got.items.len(), 2);
        assert_eq!(got.items[0].rank, 1);
        assert_eq!(got.items[0].full_name, "owner/top-repo");
        assert_eq!(got.stats.total_count, 50);
        assert_eq!(got.stats.new_count, 3);
        assert_eq!(got.ranking_mode, RankingMode::StarsDesc);
    }

    #[test]
    fn snapshot_updates_last_snapshot_at() {
        let conn = setup_db();
        create_ranking_view(&conn, &sample_view("rv_01", "Test")).unwrap();

        let snap = sample_snapshot("rv_01", "rs_01");
        save_ranking_snapshot(&conn, &snap).unwrap();

        let view = get_ranking_view(&conn, "rv_01").unwrap().unwrap();
        assert_eq!(view.last_snapshot_at, Some("2026-03-23T06:00:00Z".into()));
    }

    #[test]
    fn get_latest_snapshot_empty() {
        let conn = setup_db();
        let result = get_latest_ranking_snapshot(&conn, "rv_01").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_toggle_pin() {
        let conn = setup_db();
        let view = sample_view("rv_01", "Test");
        create_ranking_view(&conn, &view).unwrap();

        let got = get_ranking_view(&conn, "rv_01").unwrap().unwrap();
        assert!(!got.is_pinned);

        toggle_pin(&conn, "rv_01").unwrap();
        let got = get_ranking_view(&conn, "rv_01").unwrap().unwrap();
        assert!(got.is_pinned);

        toggle_pin(&conn, "rv_01").unwrap();
        let got = get_ranking_view(&conn, "rv_01").unwrap().unwrap();
        assert!(!got.is_pinned);
    }
}
