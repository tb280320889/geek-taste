//! 信号管理用例 — list / ack / mark_seen / unread_count

use anyhow::Result;
use persistence_sqlite::signal_repository;
use rusqlite::Connection;
use shared_contracts::signal_dto::{SignalDto, UnreadCountsDto};

/// 列出 Home 页信号（state=NEW 或 SEEN，支持 since + 多因子排序）
pub fn list_home_signals(
    conn: &Connection,
    since: Option<&str>,
    language_interests: &[String],
) -> Result<Vec<SignalDto>> {
    let signals = signal_repository::list_home_signals(conn, since, 50, language_interests)?;
    let dtos = signals.into_iter().map(|s| s.into()).collect();
    Ok(dtos)
}

/// 列出信号（可选过滤）
pub fn list_signals(
    conn: &Connection,
    filter_state: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<SignalDto>> {
    let state_filter = filter_state.as_deref().and_then(|s| match s {
        "NEW" => Some(domain::signal::SignalState::New),
        "SEEN" => Some(domain::signal::SignalState::Seen),
        "ACKED" => Some(domain::signal::SignalState::Acked),
        _ => None,
    });
    let signals =
        signal_repository::list_signals(conn, state_filter.as_ref(), None, limit.unwrap_or(50))?;
    let dtos = signals.into_iter().map(|s| s.into()).collect();
    Ok(dtos)
}

/// 标记为已读
pub fn mark_seen(conn: &Connection, signal_id: &str) -> Result<()> {
    signal_repository::mark_signal_seen(conn, signal_id)?;
    Ok(())
}

/// 标记为已处理
pub fn ack_signal(conn: &Connection, signal_id: &str) -> Result<()> {
    signal_repository::mark_signal_acked(conn, signal_id)?;
    Ok(())
}

/// 获取未读计数
pub fn get_unread_counts(conn: &Connection) -> Result<UnreadCountsDto> {
    let (high, medium, low) = signal_repository::count_unread_by_priority(conn)?;
    Ok(UnreadCountsDto {
        total: high + medium + low,
        high,
        medium,
        low,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::signal::Signal;
    use persistence_sqlite::init_db;
    use rusqlite::Connection;

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
        persistence_sqlite::repo_repository::upsert_repository(
            &conn,
            &domain::repository::Repository {
                repo_id: 1,
                full_name: "o/r".into(),
                owner: "o".into(),
                name: "r".into(),
                html_url: "https://github.com/o/r".into(),
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
        conn
    }

    #[test]
    fn test_list_home_and_ack() {
        let conn = setup_db();
        let signal = build_release_signal("sig1", 1, "rel1", "v1.0");
        signal_repository::insert_signal(&conn, &signal).unwrap();

        let home = list_home_signals(&conn, None, &[]).unwrap();
        assert_eq!(home.len(), 1);
        assert_eq!(home[0].priority, "HIGH");

        ack_signal(&conn, "sig1").unwrap();
        let home = list_home_signals(&conn, None, &[]).unwrap();
        assert_eq!(home.len(), 0);
    }

    #[test]
    fn test_unread_counts() {
        let conn = setup_db();
        let high = build_release_signal("s1", 1, "r1", "H");
        let med = build_tag_signal("s2", 1, "t1", "M");
        signal_repository::insert_signal(&conn, &high).unwrap();
        signal_repository::insert_signal(&conn, &med).unwrap();

        let counts = get_unread_counts(&conn).unwrap();
        assert_eq!(counts.total, 2);
        assert_eq!(counts.high, 1);
        assert_eq!(counts.medium, 1);
        assert_eq!(counts.low, 0);
    }
}
