//! Subscription 的 SQLite CRUD 实现

use anyhow::Result;
use domain::subscription::{Subscription, SubscriptionState, TrackingMode};
use rusqlite::{params, Connection};

/// 创建订阅
pub fn create_subscription(conn: &Connection, sub: &Subscription) -> Result<()> {
    let event_types_json = serde_json::to_string(&sub.event_types)?;
    conn.execute(
        "INSERT INTO subscriptions
         (subscription_id, repo_id, state, tracking_mode, event_types_json,
          digest_window, notify_high_immediately, created_at, updated_at,
          last_successful_sync_at, cursor_release_id, cursor_tag_name, cursor_branch_sha)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            sub.subscription_id,
            sub.repo_id,
            serde_json::to_string(&sub.state)?.trim_matches('"'),
            serde_json::to_string(&sub.tracking_mode)?.trim_matches('"'),
            event_types_json,
            sub.digest_window,
            sub.notify_high_immediately as i32,
            sub.created_at,
            sub.updated_at,
            sub.last_successful_sync_at,
            sub.cursor_release_id,
            sub.cursor_tag_name,
            sub.cursor_branch_sha,
        ],
    )?;
    Ok(())
}

/// 列出所有订阅（JOIN repositories 获取 repo 信息）
pub fn list_subscriptions(conn: &Connection) -> Result<Vec<SubscriptionRow>> {
    let mut stmt = conn.prepare(
        "SELECT s.subscription_id, s.repo_id, s.state, s.tracking_mode,
                s.event_types_json, s.digest_window, s.notify_high_immediately,
                s.created_at, s.updated_at, s.last_successful_sync_at,
                s.cursor_release_id, s.cursor_tag_name, s.cursor_branch_sha,
                r.full_name, r.html_url, r.description, r.primary_language, r.stargazers_count
         FROM subscriptions s
         JOIN repositories r ON s.repo_id = r.repo_id
         ORDER BY s.created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let event_types_json: String = row.get(4)?;
        Ok(SubscriptionRow {
            subscription_id: row.get(0)?,
            repo_id: row.get(1)?,
            state_str: row.get(2)?,
            tracking_mode_str: row.get(3)?,
            event_types_json,
            digest_window: row.get(5)?,
            notify_high_immediately: row.get::<_, i32>(6)? != 0,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            last_successful_sync_at: row.get(9)?,
            cursor_release_id: row.get(10)?,
            cursor_tag_name: row.get(11)?,
            cursor_branch_sha: row.get(12)?,
            full_name: row.get(13)?,
            html_url: row.get(14)?,
            description: row.get(15)?,
            primary_language: row.get(16)?,
            stargazers_count: row.get(17)?,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 按 subscription_id 获取
pub fn get_subscription_by_id(
    conn: &Connection,
    subscription_id: &str,
) -> Result<Option<Subscription>> {
    let mut stmt = conn.prepare(
        "SELECT subscription_id, repo_id, state, tracking_mode, event_types_json,
                digest_window, notify_high_immediately, created_at, updated_at,
                last_successful_sync_at, cursor_release_id, cursor_tag_name, cursor_branch_sha
         FROM subscriptions WHERE subscription_id = ?",
    )?;
    let mut rows = stmt.query(params![subscription_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_subscription(row)?))
    } else {
        Ok(None)
    }
}

/// 按 repo_id 获取活跃订阅
pub fn get_subscription_by_repo_id(
    conn: &Connection,
    repo_id: i64,
) -> Result<Option<Subscription>> {
    let mut stmt = conn.prepare(
        "SELECT subscription_id, repo_id, state, tracking_mode, event_types_json,
                digest_window, notify_high_immediately, created_at, updated_at,
                last_successful_sync_at, cursor_release_id, cursor_tag_name, cursor_branch_sha
         FROM subscriptions WHERE repo_id = ? AND state = 'ACTIVE'",
    )?;
    let mut rows = stmt.query(params![repo_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_subscription(row)?))
    } else {
        Ok(None)
    }
}

/// 兼容旧调用方：按 repo_id 获取活跃订阅
pub fn get_active_subscription_by_repo_id(
    conn: &Connection,
    repo_id: i64,
) -> Result<Option<Subscription>> {
    get_subscription_by_repo_id(conn, repo_id)
}

/// 更新订阅状态
pub fn update_subscription_state(
    conn: &Connection,
    subscription_id: &str,
    state: &SubscriptionState,
) -> Result<()> {
    let state_str = serde_json::to_string(state)?.trim_matches('"').to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE subscriptions SET state = ?, updated_at = ? WHERE subscription_id = ?",
        params![state_str, now, subscription_id],
    )?;
    Ok(())
}

/// 删除订阅
pub fn delete_subscription(conn: &Connection, subscription_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM subscriptions WHERE subscription_id = ?",
        params![subscription_id],
    )?;
    Ok(())
}

/// 更新游标 + 同步时间
pub fn update_subscription_cursors(
    conn: &Connection,
    subscription_id: &str,
    cursor_release_id: Option<&str>,
    cursor_tag_name: Option<&str>,
    cursor_branch_sha: Option<&str>,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE subscriptions SET cursor_release_id = ?, cursor_tag_name = ?, cursor_branch_sha = ?, last_successful_sync_at = ?, updated_at = ? WHERE subscription_id = ?",
        params![cursor_release_id, cursor_tag_name, cursor_branch_sha, now, now, subscription_id],
    )?;
    Ok(())
}

/// 列出所有活跃订阅
pub fn list_active_subscriptions(conn: &Connection) -> Result<Vec<Subscription>> {
    let mut stmt = conn.prepare(
        "SELECT subscription_id, repo_id, state, tracking_mode, event_types_json,
                digest_window, notify_high_immediately, created_at, updated_at,
                last_successful_sync_at, cursor_release_id, cursor_tag_name, cursor_branch_sha
         FROM subscriptions WHERE state = 'ACTIVE'",
    )?;
    let rows = stmt.query_map([], |row| row_to_subscription(row))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 列出所有活跃订阅的 repo_id 集合（用于 TopK is_subscribed 交叉查询）
pub fn list_active_repo_ids(conn: &Connection) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT repo_id FROM subscriptions WHERE state = 'ACTIVE'")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

// ── 内部结构 ──────────────────────────────────────────────

/// list_subscriptions 返回的带 repo 信息的行
pub struct SubscriptionRow {
    pub subscription_id: String,
    pub repo_id: i64,
    pub state_str: String,
    pub tracking_mode_str: String,
    pub event_types_json: String,
    pub digest_window: String,
    pub notify_high_immediately: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_successful_sync_at: Option<String>,
    pub cursor_release_id: Option<String>,
    pub cursor_tag_name: Option<String>,
    pub cursor_branch_sha: Option<String>,
    // repo info
    pub full_name: String,
    pub html_url: String,
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub stargazers_count: i64,
}

fn parse_state(s: &str) -> SubscriptionState {
    match s {
        "ACTIVE" => SubscriptionState::Active,
        "PAUSED" => SubscriptionState::Paused,
        "ARCHIVED" => SubscriptionState::Archived,
        _ => SubscriptionState::Active,
    }
}

fn parse_tracking_mode(s: &str) -> TrackingMode {
    match s {
        "ADVANCED" => TrackingMode::Advanced,
        _ => TrackingMode::Standard,
    }
}

fn row_to_subscription(row: &rusqlite::Row) -> rusqlite::Result<Subscription> {
    let state_str: String = row.get(2)?;
    let tracking_mode_str: String = row.get(3)?;
    let event_types_json: String = row.get(4)?;
    let notify_high: i32 = row.get(6)?;

    let event_types: Vec<String> = serde_json::from_str(&event_types_json).unwrap_or_default();

    Ok(Subscription {
        subscription_id: row.get(0)?,
        repo_id: row.get(1)?,
        state: parse_state(&state_str),
        tracking_mode: parse_tracking_mode(&tracking_mode_str),
        event_types,
        digest_window: row.get(5)?,
        notify_high_immediately: notify_high != 0,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        last_successful_sync_at: row.get(9)?,
        cursor_release_id: row.get(10)?,
        cursor_tag_name: row.get(11)?,
        cursor_branch_sha: row.get(12)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_db;
    use crate::repo_repository::upsert_repository;
    use domain::repository::Repository;

    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();
        conn
    }

    fn sample_repo(id: i64) -> Repository {
        Repository {
            repo_id: id,
            full_name: format!("owner/repo-{id}"),
            owner: "owner".into(),
            name: format!("repo-{id}"),
            html_url: format!("https://github.com/owner/repo-{id}"),
            description: Some("A repo".into()),
            default_branch: "main".into(),
            primary_language: Some("Rust".into()),
            topics: vec![],
            archived: false,
            disabled: false,
            stargazers_count: 100,
            forks_count: 10,
            updated_at: "2026-03-23T00:00:00Z".into(),
            pushed_at: None,
            last_synced_at: "2026-03-23T00:00:00Z".into(),
        }
    }

    fn sample_sub(repo_id: i64) -> Subscription {
        Subscription {
            subscription_id: format!("sub_{repo_id}"),
            repo_id,
            state: SubscriptionState::Active,
            tracking_mode: TrackingMode::Standard,
            event_types: domain::subscription::default_event_types(),
            digest_window: "24h".into(),
            notify_high_immediately: true,
            created_at: "2026-03-23T00:00:00Z".into(),
            updated_at: "2026-03-23T00:00:00Z".into(),
            last_successful_sync_at: None,
            cursor_release_id: None,
            cursor_tag_name: None,
            cursor_branch_sha: None,
        }
    }

    #[test]
    fn create_and_get_subscription() {
        let conn = setup_db();
        upsert_repository(&conn, &sample_repo(1)).unwrap();
        create_subscription(&conn, &sample_sub(1)).unwrap();

        let got = get_subscription_by_id(&conn, "sub_1").unwrap().unwrap();
        assert_eq!(got.repo_id, 1);
        assert_eq!(got.state, SubscriptionState::Active);
    }

    #[test]
    fn get_active_by_repo_id() {
        let conn = setup_db();
        upsert_repository(&conn, &sample_repo(1)).unwrap();
        create_subscription(&conn, &sample_sub(1)).unwrap();

        let got = get_subscription_by_repo_id(&conn, 1).unwrap();
        assert!(got.is_some());

        let not_found = get_subscription_by_repo_id(&conn, 999).unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn update_state() {
        let conn = setup_db();
        upsert_repository(&conn, &sample_repo(1)).unwrap();
        create_subscription(&conn, &sample_sub(1)).unwrap();

        update_subscription_state(&conn, "sub_1", &SubscriptionState::Paused).unwrap();
        let got = get_subscription_by_id(&conn, "sub_1").unwrap().unwrap();
        assert_eq!(got.state, SubscriptionState::Paused);
    }

    #[test]
    fn test_delete_subscription() {
        let conn = setup_db();
        upsert_repository(&conn, &sample_repo(1)).unwrap();
        create_subscription(&conn, &sample_sub(1)).unwrap();
        delete_subscription(&conn, "sub_1").unwrap();
        assert!(get_subscription_by_id(&conn, "sub_1").unwrap().is_none());
    }

    #[test]
    fn test_list_active_repo_ids() {
        let conn = setup_db();
        upsert_repository(&conn, &sample_repo(1)).unwrap();
        upsert_repository(&conn, &sample_repo(2)).unwrap();
        create_subscription(&conn, &sample_sub(1)).unwrap();
        create_subscription(&conn, &sample_sub(2)).unwrap();

        let mut paused = sample_sub(2);
        paused.subscription_id = "sub_2".into();
        update_subscription_state(&conn, "sub_2", &SubscriptionState::Paused).unwrap();

        let ids = list_active_repo_ids(&conn).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], 1);
    }

    #[test]
    fn update_cursors() {
        let conn = setup_db();
        upsert_repository(&conn, &sample_repo(1)).unwrap();
        create_subscription(&conn, &sample_sub(1)).unwrap();

        update_subscription_cursors(&conn, "sub_1", Some("rel_123"), Some("v1.0.0"), None).unwrap();
        let got = get_subscription_by_id(&conn, "sub_1").unwrap().unwrap();
        assert_eq!(got.cursor_release_id, Some("rel_123".into()));
        assert_eq!(got.cursor_tag_name, Some("v1.0.0".into()));
        assert!(got.last_successful_sync_at.is_some());
    }
}
