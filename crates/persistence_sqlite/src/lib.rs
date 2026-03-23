//! persistence_sqlite — SQLite repository impl

pub mod migrations;
pub mod repo_repository;

/// 初始化数据库：执行 migrations + 配置 WAL/busy_timeout
pub fn init_db(conn: &mut rusqlite::Connection) -> anyhow::Result<()> {
    migrations::migrations().to_latest(conn)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn init_db_creates_all_tables() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();

        // 验证表存在
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('repositories', 'repo_snapshots', 'ranking_views', 'ranking_snapshots')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn init_db_configures_busy_timeout() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();

        // 验证 busy_timeout（内存数据库中可正常设置）
        let timeout: i64 = conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .unwrap();
        assert_eq!(timeout, 5000);
    }

    #[test]
    fn init_db_configures_wal_on_file_db() {
        let dir = std::env::temp_dir().join("geek_taste_test_wal");
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test_wal.db");
        let mut conn = Connection::open(&db_path).unwrap();
        init_db(&mut conn).unwrap();

        // 文件数据库应启用 WAL
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode, "wal");

        // 清理
        drop(conn);
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir(&dir);
    }
}
