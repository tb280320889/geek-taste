//! Command helpers — shared DB connection + token loading

use keyring::Entry;
use rusqlite::Connection;
use tauri::{AppHandle, Manager};

const SERVICE: &str = "geek-taste";
const TOKEN_KEY: &str = "github-pat";

/// 获取 DB 连接（每次调用打开独立连接，WAL 模式支持并发）
pub fn get_db_connection(app: &AppHandle) -> Result<Connection, String> {
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
pub fn load_token() -> Result<String, String> {
    let entry = Entry::new(SERVICE, TOKEN_KEY).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}
