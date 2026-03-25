//! 订阅管理用例 — subscribe / unsubscribe / list / pause / sync

use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Timelike, Utc};
use domain::signal::Signal;
use domain::subscription::{Subscription, SubscriptionState, TrackingMode};
use github_adapter::releases::{fetch_latest_releases, fetch_latest_tags};
use persistence_sqlite::signal_repository;
use persistence_sqlite::subscription_repository;
use rusqlite::Connection;
use shared_contracts::settings_dto::{QuietHoursDto, SettingsDto};
use shared_contracts::subscription_dto::{CreateSubscriptionRequest, SubscriptionRowDto};

/// 生成 ULID 风格 ID
fn generate_id(prefix: &str) -> String {
    let now = Utc::now().timestamp_millis();
    let rand_suffix: u64 = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        hasher.finish()
    };
    format!("{}_{:013x}{:08x}", prefix, now, rand_suffix & 0xFFFF_FFFF)
}

#[derive(Debug, Clone)]
pub enum SyncCandidate {
    Release {
        release_id: String,
        title: String,
        occurred_at: String,
        evidence: serde_json::Value,
    },
    Tag {
        tag_name: String,
        title: String,
        occurred_at: String,
        evidence: serde_json::Value,
    },
    Digest {
        bucket_start: String,
        bucket_end: String,
        title: String,
        occurred_at: String,
        evidence: serde_json::Value,
    },
}

fn parse_rfc3339(ts: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn parse_digest_hours(window: &str) -> i64 {
    match window {
        "12h" => 12,
        _ => 24,
    }
}

fn build_digest_bucket(
    occurred_at: DateTime<Utc>,
    digest_window: &str,
) -> Option<(String, String)> {
    let hours = parse_digest_hours(digest_window);
    let seconds = hours * 3600;
    if seconds <= 0 {
        return None;
    }

    let ts = occurred_at.timestamp();
    let bucket_start_ts = ts - (ts.rem_euclid(seconds));
    let bucket_end_ts = bucket_start_ts + seconds;
    let start = Utc.timestamp_opt(bucket_start_ts, 0).single()?;
    let end = Utc.timestamp_opt(bucket_end_ts, 0).single()?;
    Some((start.to_rfc3339(), end.to_rfc3339()))
}

fn should_generate_digest(
    sub: &Subscription,
    repo: &domain::repository::Repository,
    candidates: &[SyncCandidate],
) -> Option<SyncCandidate> {
    let has_release_or_tag = candidates.iter().any(|candidate| {
        matches!(
            candidate,
            SyncCandidate::Release { .. } | SyncCandidate::Tag { .. }
        )
    });
    if has_release_or_tag {
        return None;
    }

    let pushed_at = parse_rfc3339(repo.pushed_at.as_deref()?)?;
    if let Some(last_synced_at) = sub
        .last_successful_sync_at
        .as_deref()
        .and_then(parse_rfc3339)
    {
        if pushed_at <= last_synced_at {
            return None;
        }
    }

    let (bucket_start, bucket_end) = build_digest_bucket(pushed_at, &sub.digest_window)?;
    Some(SyncCandidate::Digest {
        bucket_start: bucket_start.clone(),
        bucket_end: bucket_end.clone(),
        title: format!("{} 默认分支在该时间窗有更新", repo.full_name),
        occurred_at: pushed_at.to_rfc3339(),
        evidence: serde_json::json!({
            "digestWindow": sub.digest_window,
            "bucketStart": bucket_start,
            "bucketEnd": bucket_end,
            "pushedAt": repo.pushed_at,
        }),
    })
}

fn resolve_sync_signals(repo_id: i64, candidates: Vec<SyncCandidate>) -> Vec<Signal> {
    // U1-U4 冲突消解：RELEASE > TAG > DIGEST
    let has_release = candidates
        .iter()
        .any(|candidate| matches!(candidate, SyncCandidate::Release { .. }));
    let has_tag = candidates
        .iter()
        .any(|candidate| matches!(candidate, SyncCandidate::Tag { .. }));

    let selected: Vec<SyncCandidate> = if has_release {
        candidates
            .into_iter()
            .filter(|candidate| matches!(candidate, SyncCandidate::Release { .. }))
            .collect()
    } else if has_tag {
        candidates
            .into_iter()
            .filter(|candidate| matches!(candidate, SyncCandidate::Tag { .. }))
            .collect()
    } else {
        candidates
            .into_iter()
            .filter(|candidate| matches!(candidate, SyncCandidate::Digest { .. }))
            .collect()
    };

    selected
        .into_iter()
        .map(|candidate| {
            let mut signal = match candidate {
                SyncCandidate::Release {
                    release_id,
                    title,
                    occurred_at,
                    evidence,
                } => {
                    let mut signal = Signal::new_release(repo_id, &release_id, title);
                    signal.evidence = evidence;
                    signal.occurred_at = occurred_at;
                    signal
                }
                SyncCandidate::Tag {
                    tag_name,
                    title,
                    occurred_at,
                    evidence,
                } => {
                    let mut signal = Signal::new_tag(repo_id, &tag_name, title);
                    signal.evidence = evidence;
                    signal.occurred_at = occurred_at;
                    signal
                }
                SyncCandidate::Digest {
                    bucket_start,
                    bucket_end,
                    title,
                    occurred_at,
                    evidence,
                } => {
                    let mut signal = Signal::new_digest(repo_id, &bucket_start, &bucket_end, title);
                    signal.evidence = evidence;
                    signal.occurred_at = occurred_at;
                    signal
                }
            };
            signal.signal_id = generate_id("sig");
            signal
        })
        .collect()
}

pub fn is_within_quiet_hours(quiet_hours: Option<&QuietHoursDto>, now: DateTime<Utc>) -> bool {
    let quiet_hours = match quiet_hours {
        Some(q) => q,
        None => return false,
    };

    fn parse_minutes(hhmm: &str) -> Option<i32> {
        let mut parts = hhmm.split(':');
        let hour: i32 = parts.next()?.parse().ok()?;
        let minute: i32 = parts.next()?.parse().ok()?;
        if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
            return None;
        }
        Some(hour * 60 + minute)
    }

    let start = match parse_minutes(&quiet_hours.start) {
        Some(v) => v,
        None => return false,
    };
    let end = match parse_minutes(&quiet_hours.end) {
        Some(v) => v,
        None => return false,
    };
    let current = (now.hour() as i32) * 60 + (now.minute() as i32);

    if start == end {
        return false;
    }
    if start < end {
        current >= start && current < end
    } else {
        current >= start || current < end
    }
}

pub fn format_signal_type_text(signal_type: &domain::signal::SignalType) -> &'static str {
    match signal_type {
        domain::signal::SignalType::ReleasePublished => "Release Published",
        domain::signal::SignalType::ReleasePrereleased => "Release Prereleased",
        domain::signal::SignalType::TagPublished => "Tag Published",
        domain::signal::SignalType::DefaultBranchActivityDigest => "Default Branch Activity Digest",
        domain::signal::SignalType::PrMergedDigest => "PR Merged Digest",
        domain::signal::SignalType::TopkViewChanged => "TopK View Changed",
        domain::signal::SignalType::ResourceEmerged => "Resource Emerged",
        domain::signal::SignalType::ResourceReranked => "Resource Reranked",
    }
}

/// 创建订阅
pub fn subscribe(
    conn: &Connection,
    repo_id: i64,
    request: &CreateSubscriptionRequest,
) -> Result<Subscription> {
    // 检查是否已有活跃订阅
    if let Some(existing) =
        subscription_repository::get_active_subscription_by_repo_id(conn, repo_id)?
    {
        anyhow::bail!(
            "Already subscribed to repo {} (subscription_id: {})",
            repo_id,
            existing.subscription_id
        );
    }

    let now = Utc::now().to_rfc3339();
    let event_types = request
        .event_types
        .clone()
        .unwrap_or_else(domain::subscription::default_event_types);

    let sub = Subscription {
        subscription_id: generate_id("sub"),
        repo_id,
        state: SubscriptionState::Active,
        tracking_mode: match request.tracking_mode.as_deref() {
            Some("ADVANCED") => TrackingMode::Advanced,
            _ => TrackingMode::Standard,
        },
        event_types,
        digest_window: request
            .digest_window
            .clone()
            .unwrap_or_else(|| "24h".into()),
        notify_high_immediately: request.notify_high_immediately.unwrap_or(true),
        created_at: now.clone(),
        updated_at: now,
        last_successful_sync_at: None,
        cursor_release_id: None,
        cursor_tag_name: None,
        cursor_branch_sha: None,
    };

    subscription_repository::create_subscription(conn, &sub)?;
    Ok(sub)
}

/// 取消订阅（设为 ARCHIVED）
pub fn unsubscribe(conn: &Connection, subscription_id: &str) -> Result<()> {
    subscription_repository::update_subscription_state(
        conn,
        subscription_id,
        &SubscriptionState::Archived,
    )?;
    Ok(())
}

/// 暂停/恢复订阅
pub fn pause_subscription(conn: &Connection, subscription_id: &str) -> Result<()> {
    let sub = subscription_repository::get_subscription_by_id(conn, subscription_id)?
        .context("Subscription not found")?;

    let new_state = match sub.state {
        SubscriptionState::Active => SubscriptionState::Paused,
        SubscriptionState::Paused => SubscriptionState::Active,
        _ => anyhow::bail!("Cannot toggle state from {:?}", sub.state),
    };

    subscription_repository::update_subscription_state(conn, subscription_id, &new_state)?;
    Ok(())
}

/// 列出所有订阅（带 repo 信息，排除 ARCHIVED）
pub fn list_subscriptions(conn: &Connection) -> Result<Vec<SubscriptionRowDto>> {
    let rows = subscription_repository::list_subscriptions(conn)?;
    let dtos = rows
        .into_iter()
        .filter(|r| r.state_str != "ARCHIVED")
        .map(|r| SubscriptionRowDto {
            subscription_id: r.subscription_id,
            repo_id: r.repo_id,
            full_name: r.full_name,
            html_url: r.html_url,
            description: r.description,
            primary_language: r.primary_language,
            stargazers_count: r.stargazers_count,
            state: r.state_str,
            tracking_mode: r.tracking_mode_str,
            event_types: serde_json::from_str(&r.event_types_json).unwrap_or_default(),
            digest_window: r.digest_window,
            notify_high_immediately: r.notify_high_immediately,
            last_successful_sync_at: r.last_successful_sync_at,
            created_at: r.created_at,
        })
        .collect();
    Ok(dtos)
}

#[derive(Debug, Clone)]
pub struct HighSignalNotification {
    pub repo_full_name: String,
    pub title: String,
    pub signal_type_text: String,
}

/// 加载同步上下文（同步，不跨 await）
pub fn load_sync_context(
    conn: &Connection,
) -> Result<Vec<(Subscription, domain::repository::Repository)>> {
    let subs = subscription_repository::list_active_subscriptions(conn)?;
    let mut pairs = Vec::new();
    for sub in subs {
        let repo = match persistence_sqlite::repo_repository::get_repository(conn, sub.repo_id)? {
            Some(r) => r,
            None => {
                eprintln!(
                    "Warn: Repo {} not found for subscription {}",
                    sub.repo_id, sub.subscription_id
                );
                continue;
            }
        };
        pairs.push((sub, repo));
    }
    Ok(pairs)
}

/// 单个订阅的同步候选结果
#[derive(Debug, Clone)]
pub struct SyncFetchResult {
    pub subscription_id: String,
    pub repo_id: i64,
    pub repo_full_name: String,
    pub candidates: Vec<SyncCandidate>,
    pub latest_release_id: Option<String>,
    pub latest_tag: Option<String>,
}

/// Async：获取所有订阅的 GitHub 更新（不持有 conn）
pub async fn fetch_all_updates(
    token: &str,
    pairs: &[(Subscription, domain::repository::Repository)],
) -> Vec<SyncFetchResult> {
    let mut results = Vec::new();

    for (sub, repo) in pairs {
        let parts: Vec<&str> = repo.full_name.splitn(2, '/').collect();
        if parts.len() != 2 {
            continue;
        }
        let (owner, name) = (parts[0], parts[1]);
        let mut candidates: Vec<SyncCandidate> = Vec::new();
        let mut latest_release_id = sub.cursor_release_id.clone();
        let mut latest_tag = sub.cursor_tag_name.clone();

        // Fetch releases
        if let Ok(releases) =
            fetch_latest_releases(token, owner, name, sub.cursor_release_id.as_deref()).await
        {
            for release in &releases {
                candidates.push(SyncCandidate::Release {
                    release_id: release.release_id.to_string(),
                    title: format!(
                        "{} 发布 {}{}",
                        repo.full_name,
                        release.tag_name,
                        if release.prerelease {
                            " (预发布)"
                        } else {
                            ""
                        }
                    ),
                    occurred_at: release
                        .published_at
                        .clone()
                        .unwrap_or_else(|| Utc::now().to_rfc3339()),
                    evidence: serde_json::json!({
                        "releaseId": release.release_id,
                        "tagName": release.tag_name,
                        "releaseUrl": release.html_url,
                    }),
                });
                latest_release_id = Some(release.release_id.to_string());
            }
        }

        // Fetch tags
        if let Ok(tags) =
            fetch_latest_tags(token, owner, name, sub.cursor_tag_name.as_deref()).await
        {
            for tag in &tags {
                candidates.push(SyncCandidate::Tag {
                    tag_name: tag.name.clone(),
                    title: format!("{} 新标签 {}", repo.full_name, tag.name),
                    occurred_at: Utc::now().to_rfc3339(),
                    evidence: serde_json::json!({
                        "tagName": tag.name,
                        "sha": tag.sha,
                    }),
                });
                latest_tag = Some(tag.name.clone());
            }
        }

        results.push(SyncFetchResult {
            subscription_id: sub.subscription_id.clone(),
            repo_id: sub.repo_id,
            repo_full_name: repo.full_name.clone(),
            candidates,
            latest_release_id,
            latest_tag,
        });
    }

    results
}

/// 同步：处理获取到的更新，生成信号，更新游标，保存到 DB
pub fn process_sync_results(
    conn: &Connection,
    pairs: &[(Subscription, domain::repository::Repository)],
    fetch_results: &[SyncFetchResult],
    settings: &SettingsDto,
) -> Result<(usize, Vec<HighSignalNotification>)> {
    let mut synced_count = 0;
    let mut notifications: Vec<HighSignalNotification> = Vec::new();

    // 构建 subscription 映射
    let sub_map: std::collections::HashMap<String, &Subscription> = pairs
        .iter()
        .map(|(sub, _)| (sub.subscription_id.clone(), sub))
        .collect();

    for fetch in fetch_results {
        let sub = match sub_map.get(&fetch.subscription_id) {
            Some(s) => *s,
            None => continue,
        };

        let mut candidates = fetch.candidates.clone();

        // 更新 release cursor
        if fetch.latest_release_id.is_some() && fetch.latest_release_id != sub.cursor_release_id {
            subscription_repository::update_subscription_cursors(
                conn,
                &sub.subscription_id,
                fetch.latest_release_id.as_deref(),
                sub.cursor_tag_name.as_deref(),
                sub.cursor_branch_sha.as_deref(),
            )?;
        }

        // 更新 tag cursor
        if fetch.latest_tag.is_some() && fetch.latest_tag != sub.cursor_tag_name {
            subscription_repository::update_subscription_cursors(
                conn,
                &sub.subscription_id,
                sub.cursor_release_id.as_deref(),
                fetch.latest_tag.as_deref(),
                sub.cursor_branch_sha.as_deref(),
            )?;
        }

        // 获取 repo 用于 digest 判断
        let repo = match persistence_sqlite::repo_repository::get_repository(conn, fetch.repo_id)? {
            Some(r) => r,
            None => continue,
        };

        if let Some(digest_candidate) = should_generate_digest(sub, &repo, &candidates) {
            candidates.push(digest_candidate);
        }

        let resolved_signals = resolve_sync_signals(fetch.repo_id, candidates);
        for signal in resolved_signals {
            let inserted = signal_repository::insert_signal(conn, &signal)?;
            if inserted
                && sub.notify_high_immediately
                && signal.priority == domain::signal::SignalPriority::High
                && !is_within_quiet_hours(settings.quiet_hours.as_ref(), Utc::now())
            {
                notifications.push(HighSignalNotification {
                    repo_full_name: fetch.repo_full_name.clone(),
                    title: signal.title.clone(),
                    signal_type_text: format_signal_type_text(&signal.signal_type).to_string(),
                });
            }
        }

        synced_count += 1;
    }

    Ok((synced_count, notifications))
}

#[cfg(test)]
mod tests {
    use super::*;
    use persistence_sqlite::init_db;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();
        // Insert a test repo
        persistence_sqlite::repo_repository::upsert_repository(
            &conn,
            &domain::repository::Repository {
                repo_id: 100,
                full_name: "owner/test-repo".into(),
                owner: "owner".into(),
                name: "test-repo".into(),
                html_url: "https://github.com/owner/test-repo".into(),
                description: Some("Test".into()),
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
            },
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_subscribe_and_list() {
        let conn = setup_db();
        let req = CreateSubscriptionRequest {
            repo_id: 100,
            tracking_mode: None,
            event_types: None,
            digest_window: None,
            notify_high_immediately: None,
        };
        let sub = subscribe(&conn, 100, &req).unwrap();
        assert_eq!(sub.repo_id, 100);
        assert_eq!(sub.state, SubscriptionState::Active);

        let list = list_subscriptions(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].full_name, "owner/test-repo");
    }

    #[test]
    fn test_subscribe_duplicate_fails() {
        let conn = setup_db();
        let req = CreateSubscriptionRequest {
            repo_id: 100,
            tracking_mode: None,
            event_types: None,
            digest_window: None,
            notify_high_immediately: None,
        };
        subscribe(&conn, 100, &req).unwrap();
        let result = subscribe(&conn, 100, &req);
        assert!(result.is_err());
    }

    #[test]
    fn test_unsubscribe() {
        let conn = setup_db();
        let req = CreateSubscriptionRequest {
            repo_id: 100,
            tracking_mode: None,
            event_types: None,
            digest_window: None,
            notify_high_immediately: None,
        };
        let sub = subscribe(&conn, 100, &req).unwrap();
        unsubscribe(&conn, &sub.subscription_id).unwrap();

        let list = list_subscriptions(&conn).unwrap();
        assert_eq!(list.len(), 0); // archived subs not in list
    }

    #[test]
    fn test_pause_and_resume() {
        let conn = setup_db();
        let req = CreateSubscriptionRequest {
            repo_id: 100,
            tracking_mode: None,
            event_types: None,
            digest_window: None,
            notify_high_immediately: None,
        };
        let sub = subscribe(&conn, 100, &req).unwrap();

        pause_subscription(&conn, &sub.subscription_id).unwrap();
        let got = subscription_repository::get_subscription_by_id(&conn, &sub.subscription_id)
            .unwrap()
            .unwrap();
        assert_eq!(got.state, SubscriptionState::Paused);

        pause_subscription(&conn, &sub.subscription_id).unwrap();
        let got = subscription_repository::get_subscription_by_id(&conn, &sub.subscription_id)
            .unwrap()
            .unwrap();
        assert_eq!(got.state, SubscriptionState::Active);
    }

    #[test]
    fn resolve_sync_signals_prefers_release_over_tag_and_digest() {
        let candidates = vec![
            SyncCandidate::Release {
                release_id: "r1".into(),
                title: "release".into(),
                occurred_at: "2026-03-23T01:00:00Z".into(),
                evidence: serde_json::json!({}),
            },
            SyncCandidate::Tag {
                tag_name: "v1.0.0".into(),
                title: "tag".into(),
                occurred_at: "2026-03-23T01:00:00Z".into(),
                evidence: serde_json::json!({}),
            },
            SyncCandidate::Digest {
                bucket_start: "2026-03-23T00:00:00Z".into(),
                bucket_end: "2026-03-23T12:00:00Z".into(),
                title: "digest".into(),
                occurred_at: "2026-03-23T01:00:00Z".into(),
                evidence: serde_json::json!({}),
            },
        ];

        let resolved = resolve_sync_signals(100, candidates);
        assert_eq!(resolved.len(), 1);
        assert!(matches!(
            resolved[0].signal_type,
            domain::signal::SignalType::ReleasePublished
        ));
    }

    #[test]
    fn resolve_sync_signals_uses_tag_when_release_missing() {
        let candidates = vec![
            SyncCandidate::Tag {
                tag_name: "v1.0.1".into(),
                title: "tag".into(),
                occurred_at: "2026-03-23T02:00:00Z".into(),
                evidence: serde_json::json!({}),
            },
            SyncCandidate::Digest {
                bucket_start: "2026-03-23T00:00:00Z".into(),
                bucket_end: "2026-03-23T12:00:00Z".into(),
                title: "digest".into(),
                occurred_at: "2026-03-23T02:00:00Z".into(),
                evidence: serde_json::json!({}),
            },
        ];

        let resolved = resolve_sync_signals(100, candidates);
        assert_eq!(resolved.len(), 1);
        assert!(matches!(
            resolved[0].signal_type,
            domain::signal::SignalType::TagPublished
        ));
    }

    #[test]
    fn resolve_sync_signals_generates_digest_only_when_no_release_or_tag() {
        let candidates = vec![SyncCandidate::Digest {
            bucket_start: "2026-03-23T00:00:00Z".into(),
            bucket_end: "2026-03-23T12:00:00Z".into(),
            title: "digest".into(),
            occurred_at: "2026-03-23T03:00:00Z".into(),
            evidence: serde_json::json!({}),
        }];

        let resolved = resolve_sync_signals(100, candidates);
        assert_eq!(resolved.len(), 1);
        assert!(matches!(
            resolved[0].signal_type,
            domain::signal::SignalType::DefaultBranchActivityDigest
        ));
        assert_eq!(
            resolved[0].signal_key,
            "100:DEFAULT_BRANCH_ACTIVITY_DIGEST:2026-03-23T00:00:00Z:2026-03-23T12:00:00Z"
        );
    }
}
