//! V001 + V002 + V003 migration — Phase 2 (4 tables) + Phase 3 (3 tables) + Phase 4 (2 tables)

use rusqlite_migration::{Migrations, M};

/// 构建所有 migrations
pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // V001: Phase 2 基础表
        M::up(
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
        ),
        // V002: Phase 3 订阅 + 信号 + 投递表
        M::up(
            r#"
            -- subscriptions
            CREATE TABLE IF NOT EXISTS subscriptions (
                subscription_id TEXT PRIMARY KEY,
                repo_id INTEGER NOT NULL REFERENCES repositories(repo_id),
                state TEXT NOT NULL DEFAULT 'ACTIVE',
                tracking_mode TEXT NOT NULL DEFAULT 'STANDARD',
                event_types_json TEXT NOT NULL,
                digest_window TEXT NOT NULL DEFAULT '24h',
                notify_high_immediately INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_successful_sync_at TEXT,
                cursor_release_id TEXT,
                cursor_tag_name TEXT,
                cursor_branch_sha TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_subs_repo ON subscriptions(repo_id);
            CREATE UNIQUE INDEX IF NOT EXISTS idx_subs_active_repo ON subscriptions(repo_id) WHERE state = 'ACTIVE';

            -- signals
            CREATE TABLE IF NOT EXISTS signals (
                signal_id TEXT PRIMARY KEY,
                signal_key TEXT NOT NULL UNIQUE,
                signal_type TEXT NOT NULL,
                source_kind TEXT NOT NULL,
                repo_id INTEGER REFERENCES repositories(repo_id),
                ranking_view_id TEXT,
                resource_id TEXT,
                priority TEXT NOT NULL,
                state TEXT NOT NULL DEFAULT 'NEW',
                title TEXT NOT NULL,
                summary TEXT,
                evidence_json TEXT NOT NULL,
                occurred_at TEXT NOT NULL,
                bucket_start_at TEXT,
                bucket_end_at TEXT,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_signals_repo_time ON signals(repo_id, occurred_at DESC);
            CREATE INDEX IF NOT EXISTS idx_signals_state_priority ON signals(state, priority);

            -- deliveries
            CREATE TABLE IF NOT EXISTS deliveries (
                delivery_id TEXT PRIMARY KEY,
                signal_id TEXT NOT NULL REFERENCES signals(signal_id),
                channel TEXT NOT NULL,
                delivery_state TEXT NOT NULL,
                scheduled_at TEXT,
                attempted_at TEXT,
                delivered_at TEXT,
                error_code TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_deliveries_signal ON deliveries(signal_id);
            "#,
        ),
        // V003: Phase 4 资源表
        M::up(
            r#"
            CREATE TABLE IF NOT EXISTS resources (
                resource_id TEXT PRIMARY KEY,
                source_repo_id INTEGER REFERENCES repositories(repo_id),
                resource_kind TEXT NOT NULL,
                title TEXT NOT NULL,
                summary TEXT,
                source_url TEXT NOT NULL,
                languages_json TEXT NOT NULL DEFAULT '[]',
                framework_tags_json TEXT NOT NULL DEFAULT '[]',
                agent_tags_json TEXT NOT NULL DEFAULT '[]',
                curation_level TEXT NOT NULL DEFAULT 'SYSTEM_DISCOVERED',
                last_scored_at TEXT,
                is_active INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS resource_tags (
                resource_id TEXT NOT NULL REFERENCES resources(resource_id),
                tag_type TEXT NOT NULL,
                tag_value TEXT NOT NULL,
                PRIMARY KEY (resource_id, tag_type, tag_value)
            );

            CREATE INDEX IF NOT EXISTS idx_resource_tags_type_value ON resource_tags(tag_type, tag_value);
            CREATE INDEX IF NOT EXISTS idx_resource_tags_resource ON resource_tags(resource_id);
            CREATE INDEX IF NOT EXISTS idx_resources_kind ON resources(resource_kind);
            CREATE INDEX IF NOT EXISTS idx_resources_active ON resources(is_active);
            "#,
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn v001_creates_all_tables() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrations().to_latest(&mut conn).unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        // V001 tables
        assert!(tables.contains(&"repositories".to_string()));
        assert!(tables.contains(&"repo_snapshots".to_string()));
        assert!(tables.contains(&"ranking_views".to_string()));
        assert!(tables.contains(&"ranking_snapshots".to_string()));
        // V002 tables
        assert!(tables.contains(&"subscriptions".to_string()));
        assert!(tables.contains(&"signals".to_string()));
        assert!(tables.contains(&"deliveries".to_string()));
        // V003 tables
        assert!(tables.contains(&"resources".to_string()));
        assert!(tables.contains(&"resource_tags".to_string()));
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
