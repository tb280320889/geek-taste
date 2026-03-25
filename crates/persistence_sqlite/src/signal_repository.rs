//! Signal 的 SQLite CRUD 实现

use anyhow::Result;
use domain::signal::{Signal, SignalPriority, SignalState};
use rusqlite::{params, Connection};

/// 幂等插入信号 — INSERT OR IGNORE, 返回是否实际插入
pub fn insert_signal(conn: &Connection, signal: &Signal) -> Result<bool> {
    let evidence_json = serde_json::to_string(&signal.evidence)?;
    let signal_type = serde_json::to_string(&signal.signal_type)?
        .trim_matches('"')
        .to_string();
    let source_kind = serde_json::to_string(&signal.source_kind)?
        .trim_matches('"')
        .to_string();
    let priority = serde_json::to_string(&signal.priority)?
        .trim_matches('"')
        .to_string();
    let state_str = serde_json::to_string(&signal.state)?
        .trim_matches('"')
        .to_string();

    let rows = conn.execute(
        "INSERT OR IGNORE INTO signals
         (signal_id, signal_key, signal_type, source_kind, repo_id, ranking_view_id, resource_id,
          priority, state, title, summary, evidence_json, occurred_at, bucket_start_at, bucket_end_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            signal.signal_id,
            signal.signal_key,
            signal_type,
            source_kind,
            signal.repo_id,
            signal.ranking_view_id,
            signal.resource_id,
            priority,
            state_str,
            signal.title,
            signal.summary,
            evidence_json,
            signal.occurred_at,
            signal.bucket_start_at,
            signal.bucket_end_at,
            signal.created_at,
        ],
    )?;
    Ok(rows > 0)
}

/// 列出信号（可选 state/priority 过滤）
pub fn list_signals(
    conn: &Connection,
    filter_state: Option<&SignalState>,
    filter_priority: Option<&SignalPriority>,
    limit: i64,
) -> Result<Vec<Signal>> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(state) = filter_state {
        let state_str = serde_json::to_string(state)?.trim_matches('"').to_string();
        conditions.push("s.state = ?".to_string());
        params_vec.push(Box::new(state_str));
    }
    if let Some(priority) = filter_priority {
        let pri_str = serde_json::to_string(priority)?
            .trim_matches('"')
            .to_string();
        conditions.push("s.priority = ?".to_string());
        params_vec.push(Box::new(pri_str));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT s.signal_id, s.signal_key, s.signal_type, s.source_kind,
                s.repo_id, s.ranking_view_id, s.resource_id, s.priority, s.state,
                s.title, s.summary, s.evidence_json, s.occurred_at,
                s.bucket_start_at, s.bucket_end_at, s.created_at
         FROM signals s {} ORDER BY
            CASE s.priority WHEN 'HIGH' THEN 3 WHEN 'MEDIUM' THEN 2 ELSE 1 END DESC,
            s.occurred_at DESC
         LIMIT ?",
        where_clause
    );

    let params_ref: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let mut all_params: Vec<&dyn rusqlite::ToSql> = params_ref;
    all_params.push(&limit);

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(all_params.as_slice(), row_to_signal)?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 列出 Home 页信号（state=NEW 或 SEEN，支持 since + 多因子排序）
pub fn list_home_signals(
    conn: &Connection,
    since: Option<&str>,
    limit: i64,
    language_interests: &[String],
) -> Result<Vec<Signal>> {
    let mut conditions: Vec<String> = vec!["s.state IN ('NEW', 'SEEN')".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(since_ts) = since {
        conditions.push("s.occurred_at > ?".to_string());
        params_vec.push(Box::new(since_ts.to_string()));
    }

    let interests_lower: Vec<String> = language_interests
        .iter()
        .map(|lang| lang.to_lowercase())
        .collect();

    let affinity_clause = if interests_lower.is_empty() {
        "CASE WHEN 1=1 THEN 0 ELSE 0 END".to_string()
    } else {
        let placeholders = vec!["?"; interests_lower.len()].join(",");
        for lang in &interests_lower {
            params_vec.push(Box::new(lang.clone()));
        }
        format!(
            "CASE WHEN lower(COALESCE(r.primary_language, '')) IN ({}) THEN 1 ELSE 0 END",
            placeholders
        )
    };

    let where_clause = format!("WHERE {}", conditions.join(" AND "));
    let sql = format!(
        "SELECT s.signal_id, s.signal_key, s.signal_type, s.source_kind,
                s.repo_id, s.ranking_view_id, s.resource_id, s.priority, s.state,
                s.title, s.summary, s.evidence_json, s.occurred_at,
                s.bucket_start_at, s.bucket_end_at, s.created_at
         FROM signals s
         LEFT JOIN repositories r ON s.repo_id = r.repo_id
         {}
         ORDER BY
            CASE s.priority WHEN 'HIGH' THEN 3 WHEN 'MEDIUM' THEN 2 ELSE 1 END DESC,
            s.occurred_at DESC,
            CASE s.source_kind WHEN 'REPOSITORY' THEN 3 WHEN 'RANKING_VIEW' THEN 2 ELSE 1 END DESC,
            {} DESC
         LIMIT ?",
        where_clause, affinity_clause
    );

    let params_ref: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let mut all_params: Vec<&dyn rusqlite::ToSql> = params_ref;
    all_params.push(&limit);

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(all_params.as_slice(), row_to_signal)?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 按 repo_id 列出信号
pub fn list_signals_by_repo(conn: &Connection, repo_id: i64, limit: i64) -> Result<Vec<Signal>> {
    let mut stmt = conn.prepare(
        "SELECT signal_id, signal_key, signal_type, source_kind,
                repo_id, ranking_view_id, resource_id, priority, state,
                title, summary, evidence_json, occurred_at,
                bucket_start_at, bucket_end_at, created_at
         FROM signals WHERE repo_id = ?
         ORDER BY occurred_at DESC LIMIT ?",
    )?;
    let rows = stmt.query_map(params![repo_id, limit], row_to_signal)?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 按 signal_id 获取信号
pub fn get_signal_by_id(conn: &Connection, signal_id: &str) -> Result<Option<Signal>> {
    let mut stmt = conn.prepare(
        "SELECT signal_id, signal_key, signal_type, source_kind,
                repo_id, ranking_view_id, resource_id, priority, state,
                title, summary, evidence_json, occurred_at,
                bucket_start_at, bucket_end_at, created_at
         FROM signals WHERE signal_id = ?",
    )?;
    let mut rows = stmt.query(params![signal_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_signal(row)?))
    } else {
        Ok(None)
    }
}

/// 标记为 SEEN
pub fn mark_signal_seen(conn: &Connection, signal_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE signals SET state = 'SEEN' WHERE signal_id = ? AND state = 'NEW'",
        params![signal_id],
    )?;
    Ok(())
}

/// 标记为 ACKED
pub fn mark_signal_acked(conn: &Connection, signal_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE signals SET state = 'ACKED' WHERE signal_id = ? AND state IN ('NEW', 'SEEN')",
        params![signal_id],
    )?;
    Ok(())
}

/// 未读计数
pub fn count_unread(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM signals WHERE state = 'NEW'",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// 获取所有信号中最新创建时间（用于同步状态检测）
pub fn get_last_signal_time(conn: &Connection) -> Result<Option<String>> {
    let time: Option<String> =
        conn.query_row("SELECT MAX(created_at) FROM signals", [], |row| row.get(0))?;
    Ok(time)
}

/// 按优先级统计未读数 — 返回 (high, medium, low)
pub fn count_unread_by_priority(conn: &Connection) -> Result<(i64, i64, i64)> {
    let high: i64 = conn.query_row(
        "SELECT COUNT(*) FROM signals WHERE state = 'NEW' AND priority = 'HIGH'",
        [],
        |row| row.get(0),
    )?;
    let medium: i64 = conn.query_row(
        "SELECT COUNT(*) FROM signals WHERE state = 'NEW' AND priority = 'MEDIUM'",
        [],
        |row| row.get(0),
    )?;
    let low: i64 = conn.query_row(
        "SELECT COUNT(*) FROM signals WHERE state = 'NEW' AND priority = 'LOW'",
        [],
        |row| row.get(0),
    )?;
    Ok((high, medium, low))
}

// ── 内部映射 ──────────────────────────────────────────────

fn parse_signal_state(s: &str) -> SignalState {
    match s {
        "SEEN" => SignalState::Seen,
        "ACKED" => SignalState::Acked,
        "ARCHIVED" => SignalState::Archived,
        _ => SignalState::New,
    }
}

fn parse_signal_priority(s: &str) -> SignalPriority {
    match s {
        "HIGH" => SignalPriority::High,
        "LOW" => SignalPriority::Low,
        _ => SignalPriority::Medium,
    }
}

fn parse_signal_type(s: &str) -> domain::signal::SignalType {
    match s {
        "RELEASE_PUBLISHED" => domain::signal::SignalType::ReleasePublished,
        "RELEASE_PRERELEASED" => domain::signal::SignalType::ReleasePrereleased,
        "TAG_PUBLISHED" => domain::signal::SignalType::TagPublished,
        "DEFAULT_BRANCH_ACTIVITY_DIGEST" => domain::signal::SignalType::DefaultBranchActivityDigest,
        "PR_MERGED_DIGEST" => domain::signal::SignalType::PrMergedDigest,
        "TOPK_VIEW_CHANGED" => domain::signal::SignalType::TopkViewChanged,
        "RESOURCE_EMERGED" => domain::signal::SignalType::ResourceEmerged,
        "RESOURCE_RERANKED" => domain::signal::SignalType::ResourceReranked,
        _ => domain::signal::SignalType::DefaultBranchActivityDigest,
    }
}

fn parse_source_kind(s: &str) -> domain::signal::SourceKind {
    match s {
        "RANKING_VIEW" => domain::signal::SourceKind::RankingView,
        "RESOURCE" => domain::signal::SourceKind::Resource,
        _ => domain::signal::SourceKind::Repository,
    }
}

fn row_to_signal(row: &rusqlite::Row) -> rusqlite::Result<Signal> {
    let signal_type_str: String = row.get(2)?;
    let source_kind_str: String = row.get(3)?;
    let priority_str: String = row.get(7)?;
    let state_str: String = row.get(8)?;
    let evidence_json: String = row.get(11)?;

    let evidence: serde_json::Value =
        serde_json::from_str(&evidence_json).unwrap_or(serde_json::Value::Null);

    Ok(Signal {
        signal_id: row.get(0)?,
        signal_key: row.get(1)?,
        signal_type: parse_signal_type(&signal_type_str),
        source_kind: parse_source_kind(&source_kind_str),
        repo_id: row.get(4)?,
        ranking_view_id: row.get(5)?,
        resource_id: row.get(6)?,
        priority: parse_signal_priority(&priority_str),
        state: parse_signal_state(&state_str),
        title: row.get(9)?,
        summary: row.get(10)?,
        evidence,
        occurred_at: row.get(12)?,
        bucket_start_at: row.get(13)?,
        bucket_end_at: row.get(14)?,
        created_at: row.get(15)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_db;
    use crate::repo_repository::upsert_repository;
    use domain::repository::Repository;

    fn build_release_signal(
        signal_id: &str,
        repo_id: i64,
        release_id: &str,
        title: &str,
    ) -> Signal {
        let mut signal = Signal::new_release(repo_id, release_id, title.to_string());
        signal.signal_id = signal_id.to_string();
        signal
    }

    fn build_tag_signal(signal_id: &str, repo_id: i64, tag_name: &str, title: &str) -> Signal {
        let mut signal = Signal::new_tag(repo_id, tag_name, title.to_string());
        signal.signal_id = signal_id.to_string();
        signal
    }

    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();
        conn
    }

    fn ensure_repo(conn: &Connection, id: i64) {
        upsert_repository(
            conn,
            &Repository {
                repo_id: id,
                full_name: format!("owner/repo-{id}"),
                owner: "owner".into(),
                name: format!("repo-{id}"),
                html_url: format!("https://github.com/owner/repo-{id}"),
                description: None,
                default_branch: "main".into(),
                primary_language: None,
                topics: vec![],
                archived: false,
                disabled: false,
                stargazers_count: 0,
                forks_count: 0,
                updated_at: "2026-03-23T00:00:00Z".into(),
                pushed_at: None,
                last_synced_at: "2026-03-23T00:00:00Z".into(),
            },
        )
        .unwrap();
    }

    #[test]
    fn insert_idempotent() {
        let conn = setup_db();
        ensure_repo(&conn, 1);
        let signal = build_release_signal("sig_1", 1, "rel_1", "v1.0");
        assert!(insert_signal(&conn, &signal).unwrap());
        assert!(!insert_signal(&conn, &signal).unwrap()); // 重复插入返回 false
    }

    #[test]
    fn list_home_signals_ordering() {
        let conn = setup_db();
        ensure_repo(&conn, 1);
        ensure_repo(&conn, 2);

        let low = build_tag_signal("sig_low", 1, "v0.1", "Tag");
        let high = build_release_signal("sig_high", 2, "rel_1", "Release");
        insert_signal(&conn, &low).unwrap();
        insert_signal(&conn, &high).unwrap();

        let signals = list_home_signals(&conn, None, 10, &[]).unwrap();
        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].priority, SignalPriority::High); // HIGH 排前面
    }

    #[test]
    fn mark_seen_and_acked() {
        let conn = setup_db();
        ensure_repo(&conn, 1);
        let signal = build_release_signal("sig_1", 1, "rel_1", "v1.0");
        insert_signal(&conn, &signal).unwrap();

        mark_signal_seen(&conn, "sig_1").unwrap();
        let signals = list_home_signals(&conn, None, 10, &[]).unwrap();
        assert_eq!(signals[0].state, SignalState::Seen);

        mark_signal_acked(&conn, "sig_1").unwrap();
        let signals = list_home_signals(&conn, None, 10, &[]).unwrap();
        assert_eq!(signals.len(), 0); // ACKED 不在 home signals 里

        let got = get_signal_by_id(&conn, "sig_1").unwrap().unwrap();
        assert_eq!(got.state, SignalState::Acked);
    }

    #[test]
    fn unread_counts() {
        let conn = setup_db();
        ensure_repo(&conn, 1);
        let high = build_release_signal("sig_h", 1, "r1", "H");
        let medium = build_tag_signal("sig_m", 1, "t1", "M");
        insert_signal(&conn, &high).unwrap();
        insert_signal(&conn, &medium).unwrap();

        assert_eq!(count_unread(&conn).unwrap(), 2);
        let (h, m, l) = count_unread_by_priority(&conn).unwrap();
        assert_eq!(h, 1);
        assert_eq!(m, 1);
        assert_eq!(l, 0);
    }

    #[test]
    fn list_home_signals_since_and_sorting_factors() {
        let conn = setup_db();

        upsert_repository(
            &conn,
            &Repository {
                repo_id: 10,
                full_name: "owner/repo-rust".into(),
                owner: "owner".into(),
                name: "repo-rust".into(),
                html_url: "https://github.com/owner/repo-rust".into(),
                description: None,
                default_branch: "main".into(),
                primary_language: Some("Rust".into()),
                topics: vec![],
                archived: false,
                disabled: false,
                stargazers_count: 0,
                forks_count: 0,
                updated_at: "2026-03-23T12:00:00Z".into(),
                pushed_at: None,
                last_synced_at: "2026-03-23T12:00:00Z".into(),
            },
        )
        .unwrap();
        upsert_repository(
            &conn,
            &Repository {
                repo_id: 11,
                full_name: "owner/repo-ts".into(),
                owner: "owner".into(),
                name: "repo-ts".into(),
                html_url: "https://github.com/owner/repo-ts".into(),
                description: None,
                default_branch: "main".into(),
                primary_language: Some("TypeScript".into()),
                topics: vec![],
                archived: false,
                disabled: false,
                stargazers_count: 0,
                forks_count: 0,
                updated_at: "2026-03-23T12:00:00Z".into(),
                pushed_at: None,
                last_synced_at: "2026-03-23T12:00:00Z".into(),
            },
        )
        .unwrap();

        let mut a = build_tag_signal("sig_a", 10, "v1.0.0", "A");
        a.occurred_at = "2026-03-23T12:30:00Z".into();
        a.source_kind = domain::signal::SourceKind::Repository;
        insert_signal(&conn, &a).unwrap();

        let mut b = build_tag_signal("sig_b", 11, "v1.0.1", "B");
        b.occurred_at = "2026-03-23T12:30:00Z".into();
        b.source_kind = domain::signal::SourceKind::Resource;
        insert_signal(&conn, &b).unwrap();

        let mut old = build_release_signal("sig_old", 10, "rel_old", "old");
        old.occurred_at = "2026-03-22T12:30:00Z".into();
        insert_signal(&conn, &old).unwrap();

        let signals = list_home_signals(
            &conn,
            Some("2026-03-23T00:00:00Z"),
            10,
            &["Rust".to_string()],
        )
        .unwrap();

        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].signal_id, "sig_a");
        assert_eq!(signals[1].signal_id, "sig_b");
    }
}
