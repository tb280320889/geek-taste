//! Repository / RepoSnapshot 的 SQLite CRUD 实现

use anyhow::Result;
use domain::repository::{RepoSnapshot, Repository};
use rusqlite::{params, Connection};
use serde_rusqlite::from_row;

/// 搜索过滤条件
#[derive(Debug, Clone)]
pub struct SearchFilters {
    pub language: Option<String>,
    pub min_stars: Option<i64>,
    pub topic: Option<String>,
    pub limit: i64,
    pub offset: i64,
    pub sort_by: String, // "stars" | "updated" | "created"
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            language: None,
            min_stars: None,
            topic: None,
            limit: 30,
            offset: 0,
            sort_by: "stars".into(),
        }
    }
}

/// upsert 仓库（INSERT OR REPLACE）
pub fn upsert_repository(conn: &Connection, repo: &Repository) -> Result<()> {
    let topics_json = serde_json::to_string(&repo.topics)?;
    conn.execute(
        "INSERT OR REPLACE INTO repositories
         (repo_id, full_name, owner, name, html_url, description, default_branch,
          primary_language, topics_json, archived, disabled, stargazers_count,
          forks_count, updated_at, pushed_at, last_synced_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            repo.repo_id,
            repo.full_name,
            repo.owner,
            repo.name,
            repo.html_url,
            repo.description,
            repo.default_branch,
            repo.primary_language,
            topics_json,
            repo.archived as i32,
            repo.disabled as i32,
            repo.stargazers_count,
            repo.forks_count,
            repo.updated_at,
            repo.pushed_at,
            repo.last_synced_at,
        ],
    )?;
    Ok(())
}

/// 按 repo_id 获取仓库
pub fn get_repository(conn: &Connection, repo_id: i64) -> Result<Option<Repository>> {
    let mut stmt = conn.prepare(
        "SELECT repo_id, full_name, owner, name, html_url, description, default_branch,
                primary_language, topics_json, archived, disabled, stargazers_count,
                forks_count, updated_at, pushed_at, last_synced_at
         FROM repositories WHERE repo_id = ?",
    )?;
    let mut rows = stmt.query(params![repo_id])?;
    if let Some(row) = rows.next()? {
        let repo: RepositoryRow = from_row(row)?;
        let topics: Vec<String> = serde_json::from_str(&repo.topics_json).unwrap_or_default();
        Ok(Some(Repository {
            repo_id: repo.repo_id,
            full_name: repo.full_name,
            owner: repo.owner,
            name: repo.name,
            html_url: repo.html_url,
            description: repo.description,
            default_branch: repo.default_branch,
            primary_language: repo.primary_language,
            topics,
            archived: repo.archived != 0,
            disabled: repo.disabled != 0,
            stargazers_count: repo.stargazers_count,
            forks_count: repo.forks_count,
            updated_at: repo.updated_at,
            pushed_at: repo.pushed_at,
            last_synced_at: repo.last_synced_at,
        }))
    } else {
        Ok(None)
    }
}

/// 搜索仓库（支持过滤、排序、分页）
pub fn search_repositories(conn: &Connection, filters: &SearchFilters) -> Result<Vec<Repository>> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref lang) = filters.language {
        conditions.push("primary_language = ?".to_string());
        params_vec.push(Box::new(lang.clone()));
    }
    if let Some(min_stars) = filters.min_stars {
        conditions.push("stargazers_count >= ?".to_string());
        params_vec.push(Box::new(min_stars));
    }
    if let Some(ref topic) = filters.topic {
        conditions.push("topics_json LIKE ?".to_string());
        params_vec.push(Box::new(format!("%\"{}\"%", topic)));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let order_clause = match filters.sort_by.as_str() {
        "updated" => "ORDER BY updated_at DESC",
        "created" => "ORDER BY created_at DESC",
        _ => "ORDER BY stargazers_count DESC",
    };

    let sql = format!(
        "SELECT repo_id, full_name, owner, name, html_url, description, default_branch,
                primary_language, topics_json, archived, disabled, stargazers_count,
                forks_count, updated_at, pushed_at, last_synced_at
         FROM repositories {} {} LIMIT ? OFFSET ?",
        where_clause, order_clause
    );

    let params_ref: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let mut all_params: Vec<&dyn rusqlite::ToSql> = params_ref;
    all_params.push(&filters.limit);
    all_params.push(&filters.offset);

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(all_params.as_slice(), |row| {
        let r: RepositoryRow = from_row(row).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;
        Ok(r)
    })?;

    let mut results = Vec::new();
    for row in rows {
        let r = row?;
        let topics: Vec<String> = serde_json::from_str(&r.topics_json).unwrap_or_default();
        results.push(Repository {
            repo_id: r.repo_id,
            full_name: r.full_name,
            owner: r.owner,
            name: r.name,
            html_url: r.html_url,
            description: r.description,
            default_branch: r.default_branch,
            primary_language: r.primary_language,
            topics,
            archived: r.archived != 0,
            disabled: r.disabled != 0,
            stargazers_count: r.stargazers_count,
            forks_count: r.forks_count,
            updated_at: r.updated_at,
            pushed_at: r.pushed_at,
            last_synced_at: r.last_synced_at,
        });
    }
    Ok(results)
}

/// 插入仓库快照
pub fn insert_repo_snapshot(conn: &Connection, snapshot: &RepoSnapshot) -> Result<()> {
    conn.execute(
        "INSERT INTO repo_snapshots (snapshot_id, repo_id, snapshot_at, stargazers_count, forks_count, updated_at, pushed_at, release_count)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            snapshot.snapshot_id,
            snapshot.repo_id,
            snapshot.snapshot_at,
            snapshot.stargazers_count,
            snapshot.forks_count,
            snapshot.updated_at,
            snapshot.pushed_at,
            snapshot.release_count,
        ],
    )?;
    Ok(())
}

/// 获取仓库最新快照
pub fn get_latest_repo_snapshot(conn: &Connection, repo_id: i64) -> Result<Option<RepoSnapshot>> {
    let mut stmt = conn.prepare(
        "SELECT snapshot_id, repo_id, snapshot_at, stargazers_count, forks_count, updated_at, pushed_at, release_count
         FROM repo_snapshots WHERE repo_id = ? ORDER BY snapshot_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query(params![repo_id])?;
    if let Some(row) = rows.next()? {
        let snap: RepoSnapshot = from_row(row)?;
        Ok(Some(snap))
    } else {
        Ok(None)
    }
}

// ── 内部映射结构 ──────────────────────────────────────────

/// serde_rusqlite 反序列化用的中间结构（topics_json 是字符串，bool 是 i32）
#[derive(Debug, serde::Deserialize)]
struct RepositoryRow {
    repo_id: i64,
    full_name: String,
    owner: String,
    name: String,
    html_url: String,
    description: Option<String>,
    default_branch: String,
    primary_language: Option<String>,
    topics_json: String,
    archived: i32,
    disabled: i32,
    stargazers_count: i64,
    forks_count: i64,
    updated_at: String,
    pushed_at: Option<String>,
    last_synced_at: String,
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

    fn sample_repo(id: i64, name: &str) -> Repository {
        Repository {
            repo_id: id,
            full_name: format!("owner/{name}"),
            owner: "owner".into(),
            name: name.into(),
            html_url: format!("https://github.com/owner/{name}"),
            description: Some("A cool repo".into()),
            default_branch: "main".into(),
            primary_language: Some("Rust".into()),
            topics: vec!["rust".into(), "cli".into()],
            archived: false,
            disabled: false,
            stargazers_count: 100,
            forks_count: 20,
            updated_at: "2026-03-23T00:00:00Z".into(),
            pushed_at: Some("2026-03-22T12:00:00Z".into()),
            last_synced_at: "2026-03-23T06:00:00Z".into(),
        }
    }

    #[test]
    fn upsert_and_get_repository() {
        let conn = setup_db();
        let repo = sample_repo(1, "test-repo");
        upsert_repository(&conn, &repo).unwrap();

        let got = get_repository(&conn, 1).unwrap().unwrap();
        assert_eq!(got.repo_id, 1);
        assert_eq!(got.full_name, "owner/test-repo");
        assert_eq!(got.topics, vec!["rust", "cli"]);
        assert_eq!(got.stargazers_count, 100);
    }

    #[test]
    fn get_repository_not_found() {
        let conn = setup_db();
        let got = get_repository(&conn, 999).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn upsert_updates_existing() {
        let conn = setup_db();
        let mut repo = sample_repo(1, "test-repo");
        upsert_repository(&conn, &repo).unwrap();

        repo.stargazers_count = 200;
        upsert_repository(&conn, &repo).unwrap();

        let got = get_repository(&conn, 1).unwrap().unwrap();
        assert_eq!(got.stargazers_count, 200);
    }

    #[test]
    fn search_by_language() {
        let conn = setup_db();
        upsert_repository(&conn, &sample_repo(1, "rust-repo")).unwrap();

        let mut repo2 = sample_repo(2, "go-repo");
        repo2.primary_language = Some("Go".into());
        upsert_repository(&conn, &repo2).unwrap();

        let results = search_repositories(
            &conn,
            &SearchFilters {
                language: Some("Rust".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].repo_id, 1);
    }

    #[test]
    fn search_by_min_stars() {
        let conn = setup_db();
        let mut repo = sample_repo(1, "popular");
        repo.stargazers_count = 500;
        upsert_repository(&conn, &repo).unwrap();
        let mut repo2 = sample_repo(2, "small");
        repo2.stargazers_count = 10;
        upsert_repository(&conn, &repo2).unwrap();

        let results = search_repositories(
            &conn,
            &SearchFilters {
                min_stars: Some(100),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].repo_id, 1);
    }

    #[test]
    fn search_by_topic() {
        let conn = setup_db();
        upsert_repository(&conn, &sample_repo(1, "rust-cli")).unwrap();
        let mut repo2 = sample_repo(2, "go-web");
        repo2.topics = vec!["go".into(), "web".into()];
        upsert_repository(&conn, &repo2).unwrap();

        let results = search_repositories(
            &conn,
            &SearchFilters {
                topic: Some("cli".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].repo_id, 1);
    }

    #[test]
    fn search_pagination() {
        let conn = setup_db();
        for i in 1..=5 {
            upsert_repository(&conn, &sample_repo(i, &format!("repo-{i}"))).unwrap();
        }

        let page1 = search_repositories(
            &conn,
            &SearchFilters {
                limit: 2,
                offset: 0,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = search_repositories(
            &conn,
            &SearchFilters {
                limit: 2,
                offset: 2,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(page2.len(), 2);

        let page3 = search_repositories(
            &conn,
            &SearchFilters {
                limit: 2,
                offset: 4,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[test]
    fn insert_and_get_latest_repo_snapshot() {
        let conn = setup_db();
        upsert_repository(&conn, &sample_repo(42, "snap-repo")).unwrap();

        let snap = RepoSnapshot {
            snapshot_id: "snap1".into(),
            repo_id: 42,
            snapshot_at: "2026-03-23T06:00:00Z".into(),
            stargazers_count: 500,
            forks_count: 50,
            updated_at: "2026-03-22T18:00:00Z".into(),
            pushed_at: Some("2026-03-22T18:00:00Z".into()),
            release_count: Some(10),
        };
        insert_repo_snapshot(&conn, &snap).unwrap();

        // 插入更早的快照
        let older_snap = RepoSnapshot {
            snapshot_id: "snap0".into(),
            repo_id: 42,
            snapshot_at: "2026-03-22T06:00:00Z".into(),
            stargazers_count: 400,
            forks_count: 40,
            updated_at: "2026-03-21T18:00:00Z".into(),
            pushed_at: None,
            release_count: None,
        };
        insert_repo_snapshot(&conn, &older_snap).unwrap();

        let latest = get_latest_repo_snapshot(&conn, 42).unwrap().unwrap();
        assert_eq!(latest.snapshot_id, "snap1");
        assert_eq!(latest.stargazers_count, 500);
        assert_eq!(latest.release_count, Some(10));
    }

    #[test]
    fn get_latest_snapshot_empty() {
        let conn = setup_db();
        let result = get_latest_repo_snapshot(&conn, 999).unwrap();
        assert!(result.is_none());
    }
}
