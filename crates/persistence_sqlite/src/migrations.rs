//! V001 migration — 创建 Phase 2 所需的 4 张表 + 索引

use rusqlite_migration::{Migrations, M};

/// 构建所有 migrations（当前仅 V001）
pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(
        r#"
        -- repositories
        CREATE TABLE IF NOT EXISTS repositories (
            repo_id INTEGER PRIMARY KEY,
            full_name TEXT UNIQUE NOT NULL,
            owner TEXT NOT NULL,
            name TEXT NOT NULL,
            html_url TEXT NOT NULL,
            description TEXT,
            default_branch TEXT NOT NULL DEFAULT 'main',
            primary_language TEXT,
            topics_json TEXT NOT NULL DEFAULT '[]',
            archived INTEGER NOT NULL DEFAULT 0,
            disabled INTEGER NOT NULL DEFAULT 0,
            stargazers_count INTEGER NOT NULL DEFAULT 0,
            forks_count INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL,
            pushed_at TEXT,
            last_synced_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_repos_language ON repositories(primary_language);
        CREATE INDEX IF NOT EXISTS idx_repos_updated ON repositories(updated_at);
        CREATE INDEX IF NOT EXISTS idx_repos_stars ON repositories(stargazers_count);

        -- repo_snapshots
        CREATE TABLE IF NOT EXISTS repo_snapshots (
            snapshot_id TEXT PRIMARY KEY,
            repo_id INTEGER NOT NULL REFERENCES repositories(repo_id),
            snapshot_at TEXT NOT NULL,
            stargazers_count INTEGER NOT NULL,
            forks_count INTEGER NOT NULL,
            updated_at TEXT NOT NULL,
            pushed_at TEXT,
            release_count INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_repo_snap_repo ON repo_snapshots(repo_id, snapshot_at DESC);

        -- ranking_views
        CREATE TABLE IF NOT EXISTS ranking_views (
            ranking_view_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            view_kind TEXT NOT NULL,
            query_template TEXT NOT NULL,
            filters_json TEXT NOT NULL,
            ranking_mode TEXT NOT NULL,
            k_value INTEGER NOT NULL,
            is_pinned INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_snapshot_at TEXT
        );

        -- ranking_snapshots
        CREATE TABLE IF NOT EXISTS ranking_snapshots (
            ranking_snapshot_id TEXT PRIMARY KEY,
            ranking_view_id TEXT NOT NULL REFERENCES ranking_views(ranking_view_id),
            snapshot_at TEXT NOT NULL,
            ranking_mode TEXT NOT NULL,
            items_json TEXT NOT NULL,
            stats_json TEXT NOT NULL
        );
        "#,
    )])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn v001_creates_all_tables() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrations().to_latest(&mut conn).unwrap();

        // 验证 4 张表已创建
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"repositories".to_string()));
        assert!(tables.contains(&"repo_snapshots".to_string()));
        assert!(tables.contains(&"ranking_views".to_string()));
        assert!(tables.contains(&"ranking_snapshots".to_string()));
    }

    #[test]
    fn v001_creates_indexes() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrations().to_latest(&mut conn).unwrap();

        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(indexes.contains(&"idx_repos_language".to_string()));
        assert!(indexes.contains(&"idx_repos_updated".to_string()));
        assert!(indexes.contains(&"idx_repos_stars".to_string()));
        assert!(indexes.contains(&"idx_repo_snap_repo".to_string()));
    }
}
