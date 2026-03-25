//! persistence_sqlite — SQLite repository impl

pub mod migrations;
pub mod ranking_repository;
pub mod repo_repository;
pub mod resource_repository;
pub mod signal_repository;
pub mod subscription_repository;

use rusqlite::Connection;

/// 初始化数据库：执行所有 migrations
pub fn init_db(conn: &mut Connection) -> anyhow::Result<()> {
    migrations::migrations().to_latest(conn)?;
    Ok(())
}
