//! TopK 用例编排 — execute_ranking / manage_views / create_snapshot

use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::Utc;
use domain::ranking::{
    MomentumScore, RankingFilters, RankingMode, RankingSnapshot, RankingSnapshotItem, RankingView,
    SnapshotStats,
};
use persistence_sqlite::ranking_repository;
use persistence_sqlite::repo_repository;
use rusqlite::Connection;
use shared_contracts::ranking_dto::{
    CreateRankingViewRequest, RankingItemDto, RankingResultDto, RankingViewSpecDto,
    ScoreBreakdownDto,
};

/// 生成 ULID 风格的 ID（时间排序 + 随机后缀）
fn generate_id(prefix: &str) -> String {
    let now = Utc::now().timestamp_millis();
    let rand_suffix: u64 = rand_u64();
    format!("{}_{:013x}{:08x}", prefix, now, rand_suffix & 0xFFFF_FFFF)
}

fn rand_u64() -> u64 {
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
}

/// 从 RankingView 的 filters 构建 GitHub SearchQuery
pub fn build_search_query(view: &RankingView) -> github_adapter::search::SearchQuery {
    use github_adapter::search::{SearchQuery, SearchSort, SortOrder};

    let (sort, order) = match view.ranking_mode {
        RankingMode::StarsDesc => (SearchSort::Stars, SortOrder::Desc),
        RankingMode::UpdatedDesc => (SearchSort::Updated, SortOrder::Desc),
        // Momentum 模式按 stars 获取候选集，后续自行计算评分
        RankingMode::Momentum24h | RankingMode::Momentum7d => (SearchSort::Stars, SortOrder::Desc),
    };

    // per_page 不超过 100（GitHub Search API 上限）
    let per_page = (view.k_value as u8).min(100);

    SearchQuery {
        language: view.filters.language.first().cloned(),
        topic: view.filters.topic.first().cloned(),
        min_stars: view.filters.min_stars.map(|v| v as u32),
        sort,
        order,
        per_page,
        page: 1,
    }
}

/// 执行排名：从 GitHub 搜索候选仓库，按 RankingMode 排序，返回前 k 条
pub async fn execute_ranking(
    conn: &Connection,
    token: &str,
    view_id: &str,
) -> Result<RankingResultDto> {
    // 1. 获取 RankingView
    let view = ranking_repository::get_ranking_view(conn, view_id)?
        .context(format!("RankingView not found: {view_id}"))?;

    // 2. 构建 SearchQuery 并调用 GitHub Search API
    let query = build_search_query(&view);
    let budget = github_adapter::rate_limit::RateBudget::new();
    let search_result = github_adapter::search::search_repositories(token, &query, &budget)
        .await
        .map_err(|e| anyhow::anyhow!("GitHub search failed: {e}"))?;

    // 3. Upsert 搜索结果到 repositories 表
    for repo in &search_result.items {
        repo_repository::upsert_repository(conn, repo)?;
    }

    // 4. 获取上一次 snapshot（用于 momentum 计算和排名变化）
    let prev_snapshot = ranking_repository::get_latest_ranking_snapshot(conn, view_id)?;

    // 5. 暖机检测：Momentum 模式无历史快照 → 暖机降级
    let is_warmup = prev_snapshot.is_none()
        && matches!(
            view.ranking_mode,
            RankingMode::Momentum24h | RankingMode::Momentum7d
        );

    // 6. 按 RankingMode 排序
    let mut items = search_result.items;
    let score_breakdowns: HashMap<i64, MomentumScore> = match view.ranking_mode {
        RankingMode::StarsDesc => {
            items.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));
            HashMap::new()
        }
        RankingMode::UpdatedDesc => {
            items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            HashMap::new()
        }
        RankingMode::Momentum24h | RankingMode::Momentum7d => {
            // 降级检查：无历史快照 → UPDATED_DESC
            if prev_snapshot.is_none() {
                items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                HashMap::new()
            } else {
                // 使用 repo_snapshots 获取上一次的 stars/forks
                let mut breakdowns = HashMap::new();
                for repo in &items {
                    let prev_snap = repo_repository::get_latest_repo_snapshot(conn, repo.repo_id)?;
                    let (prev_stars, prev_forks) = prev_snap
                        .map(|s| (s.stargazers_count, s.forks_count))
                        .unwrap_or((0, 0));

                    // 计算 days_since_update
                    let updated_dt = chrono::DateTime::parse_from_rfc3339(&repo.updated_at)
                        .unwrap_or_else(|_| Utc::now().into());
                    let days_since = (Utc::now() - updated_dt.with_timezone(&Utc)).num_seconds()
                        as f64
                        / 86400.0;

                    let score = MomentumScore::compute(
                        repo.stargazers_count,
                        prev_stars,
                        repo.forks_count,
                        prev_forks,
                        days_since,
                    );
                    breakdowns.insert(repo.repo_id, score);
                }

                items.sort_by(|a, b| {
                    let sa = breakdowns.get(&a.repo_id).map(|s| s.total).unwrap_or(0.0);
                    let sb = breakdowns.get(&b.repo_id).map(|s| s.total).unwrap_or(0.0);
                    sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                });
                breakdowns
            }
        }
    };

    // 7. 截取前 k 条
    items.truncate(view.k_value as usize);

    // 8. 构建上一次 snapshot 的排名映射（用于 rank_change）
    let prev_rank_map: HashMap<i64, i32> = prev_snapshot
        .as_ref()
        .map(|snap| {
            snap.items
                .iter()
                .map(|item| (item.repo_id, item.rank))
                .collect()
        })
        .unwrap_or_default();

    // 9. 查询活跃订阅 repo_id（用于 is_subscribed）
    let subscribed_repo_ids: std::collections::HashSet<i64> =
        persistence_sqlite::subscription_repository::list_active_repo_ids(conn)?
            .into_iter()
            .collect();

    // 10. 转换为 RankingItemDto
    let dtos: Vec<RankingItemDto> = items
        .into_iter()
        .enumerate()
        .map(|(idx, repo)| {
            let rank = (idx + 1) as i32;
            let score = score_breakdowns
                .get(&repo.repo_id)
                .map(|s| s.total)
                .unwrap_or_else(|| {
                    // 非 momentum 模式，用 stars 作为 score（归一化到 0-1 范围近似）
                    if view.ranking_mode == RankingMode::StarsDesc {
                        (repo.stargazers_count as f64 / 100_000.0).min(1.0)
                    } else {
                        0.0
                    }
                });

            let score_breakdown = score_breakdowns
                .get(&repo.repo_id)
                .map(|s| ScoreBreakdownDto {
                    star_delta: s.star_delta_norm,
                    fork_delta: s.fork_delta_norm,
                    updated_recency: s.updated_recency_norm,
                });

            let rank_change = prev_rank_map.get(&repo.repo_id).map(|&prev_rank| {
                prev_rank - rank // 正值 = 排名上升
            });

            RankingItemDto {
                repo_id: repo.repo_id,
                full_name: repo.full_name,
                html_url: repo.html_url,
                description: repo.description,
                primary_language: repo.primary_language,
                stars: repo.stargazers_count,
                forks: repo.forks_count,
                rank,
                score,
                score_breakdown,
                rank_change,
                is_subscribed: subscribed_repo_ids.contains(&repo.repo_id),
            }
        })
        .collect();

    Ok(RankingResultDto {
        items: dtos,
        warmup: is_warmup,
    })
}

/// 列出所有榜单视图
pub fn list_views(conn: &Connection) -> Result<Vec<RankingViewSpecDto>> {
    let views = ranking_repository::list_ranking_views(conn)?;
    Ok(views.into_iter().map(RankingViewSpecDto::from).collect())
}

/// 创建榜单视图
pub fn create_view(
    conn: &Connection,
    request: CreateRankingViewRequest,
) -> Result<RankingViewSpecDto> {
    let now = Utc::now().to_rfc3339();
    let filters = RankingFilters {
        language: request.filters.language,
        exclude_archived: request.filters.exclude_archived,
        exclude_forks: request.filters.exclude_forks,
        min_stars: request.filters.min_stars,
        updated_since_days: request.filters.updated_since_days,
        topic: request.filters.topic,
    };
    let ranking_mode =
        RankingMode::from_str(&request.ranking_mode).unwrap_or(RankingMode::StarsDesc);

    let view = RankingView {
        ranking_view_id: generate_id("rv"),
        name: request.name,
        view_kind: "CUSTOM".into(),
        query_template: String::new(), // v1: filters 直接构建 query
        filters,
        ranking_mode,
        k_value: request.k_value.clamp(1, 1000),
        is_pinned: false,
        created_at: now.clone(),
        updated_at: now,
        last_snapshot_at: None,
    };

    let saved = ranking_repository::create_ranking_view(conn, &view)?;
    Ok(RankingViewSpecDto::from(saved))
}

/// 删除榜单视图
pub fn delete_view(conn: &Connection, view_id: &str) -> Result<()> {
    ranking_repository::delete_ranking_view(conn, view_id)
}

/// 切换 pin 状态
pub fn toggle_pin_view(conn: &Connection, view_id: &str) -> Result<()> {
    ranking_repository::toggle_pin(conn, view_id)
}

/// 创建快照
pub fn create_snapshot(conn: &Connection, view_id: &str, items: &[RankingItemDto]) -> Result<()> {
    let view = ranking_repository::get_ranking_view(conn, view_id)?
        .context(format!("RankingView not found: {view_id}"))?;

    let snapshot_items: Vec<RankingSnapshotItem> = items
        .iter()
        .map(|dto| RankingSnapshotItem {
            repo_id: dto.repo_id,
            full_name: dto.full_name.clone(),
            rank: dto.rank,
            score: dto.score,
            is_subscribed: dto.is_subscribed,
        })
        .collect();

    let new_count = snapshot_items
        .iter()
        .filter(|item| item.rank == 1) // 简化统计
        .count() as i32;
    let changed_count = snapshot_items
        .iter()
        .filter(|item| item.score > 0.5)
        .count() as i32;

    let snapshot = RankingSnapshot {
        ranking_snapshot_id: generate_id("rs"),
        ranking_view_id: view_id.to_string(),
        snapshot_at: Utc::now().to_rfc3339(),
        ranking_mode: view.ranking_mode,
        items: snapshot_items,
        stats: SnapshotStats {
            total_count: items.len() as i32,
            new_count,
            changed_count,
        },
    };

    ranking_repository::save_ranking_snapshot(conn, &snapshot)?;
    Ok(())
}

/// 获取排名变化：对比当前排名与上次快照
pub fn get_rank_change(
    conn: &Connection,
    view_id: &str,
    current_items: &[RankingItemDto],
) -> Result<Vec<Option<i32>>> {
    let prev_snapshot = ranking_repository::get_latest_ranking_snapshot(conn, view_id)?;

    let prev_rank_map: HashMap<i64, i32> = prev_snapshot
        .map(|snap| {
            snap.items
                .into_iter()
                .map(|item| (item.repo_id, item.rank))
                .collect()
        })
        .unwrap_or_default();

    let changes: Vec<Option<i32>> = current_items
        .iter()
        .map(|item| {
            prev_rank_map
                .get(&item.repo_id)
                .map(|&prev_rank| prev_rank - item.rank)
        })
        .collect();

    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use persistence_sqlite::init_db;
    use shared_contracts::ranking_dto::FiltersDto;

    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();
        conn
    }

    fn sample_request() -> CreateRankingViewRequest {
        CreateRankingViewRequest {
            name: "Test View".into(),
            filters: FiltersDto {
                language: vec!["Rust".into()],
                exclude_archived: true,
                exclude_forks: true,
                min_stars: Some(100),
                updated_since_days: None,
                topic: vec![],
            },
            ranking_mode: "STARS_DESC".into(),
            k_value: 50,
        }
    }

    #[test]
    fn create_and_list_views() {
        let conn = setup_db();
        let dto = create_view(&conn, sample_request()).unwrap();
        assert_eq!(dto.name, "Test View");
        assert_eq!(dto.ranking_mode, "STARS_DESC");
        assert_eq!(dto.k_value, 50);
        assert!(dto.ranking_view_id.starts_with("rv_"));

        let views = list_views(&conn).unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].ranking_view_id, dto.ranking_view_id);
    }

    #[test]
    fn delete_view_works() {
        let conn = setup_db();
        let dto = create_view(&conn, sample_request()).unwrap();
        super::delete_view(&conn, &dto.ranking_view_id).unwrap();
        let views = list_views(&conn).unwrap();
        assert!(views.is_empty());
    }

    #[test]
    fn toggle_pin() {
        let conn = setup_db();
        let dto = create_view(&conn, sample_request()).unwrap();
        assert!(!dto.is_pinned);

        toggle_pin_view(&conn, &dto.ranking_view_id).unwrap();
        let views = list_views(&conn).unwrap();
        assert!(views[0].is_pinned);

        toggle_pin_view(&conn, &dto.ranking_view_id).unwrap();
        let views = list_views(&conn).unwrap();
        assert!(!views[0].is_pinned);
    }

    #[test]
    fn build_search_query_stars() {
        let view = RankingView {
            ranking_view_id: "rv_1".into(),
            name: "Test".into(),
            view_kind: "CUSTOM".into(),
            query_template: "".into(),
            filters: RankingFilters {
                language: vec!["Rust".into()],
                exclude_archived: true,
                exclude_forks: true,
                min_stars: Some(100),
                updated_since_days: None,
                topic: vec!["cli".into()],
            },
            ranking_mode: RankingMode::StarsDesc,
            k_value: 50,
            is_pinned: false,
            created_at: "".into(),
            updated_at: "".into(),
            last_snapshot_at: None,
        };
        let query = build_search_query(&view);
        assert_eq!(query.language, Some("Rust".into()));
        assert_eq!(query.min_stars, Some(100));
        assert_eq!(query.topic, Some("cli".into()));
        assert_eq!(query.per_page, 50);
    }

    #[test]
    fn build_search_query_momentum() {
        let view = RankingView {
            ranking_view_id: "rv_2".into(),
            name: "Momentum".into(),
            view_kind: "CUSTOM".into(),
            query_template: "".into(),
            filters: RankingFilters::new(),
            ranking_mode: RankingMode::Momentum7d,
            k_value: 30,
            is_pinned: false,
            created_at: "".into(),
            updated_at: "".into(),
            last_snapshot_at: None,
        };
        let query = build_search_query(&view);
        // Momentum 模式按 Stars DESC 获取候选
        assert_eq!(query.sort, github_adapter::search::SearchSort::Stars);
        assert_eq!(query.per_page, 30);
    }

    #[test]
    fn create_snapshot_and_get_rank_change() {
        let conn = setup_db();
        let dto = create_view(&conn, sample_request()).unwrap();

        let items = vec![
            RankingItemDto {
                repo_id: 1,
                full_name: "o/a".into(),
                html_url: "https://github.com/o/a".into(),
                description: None,
                primary_language: Some("Rust".into()),
                stars: 500,
                forks: 50,
                rank: 1,
                score: 0.9,
                score_breakdown: None,
                rank_change: None,
                is_subscribed: false,
            },
            RankingItemDto {
                repo_id: 2,
                full_name: "o/b".into(),
                html_url: "https://github.com/o/b".into(),
                description: None,
                primary_language: Some("Rust".into()),
                stars: 300,
                forks: 30,
                rank: 2,
                score: 0.7,
                score_breakdown: None,
                rank_change: None,
                is_subscribed: false,
            },
        ];

        create_snapshot(&conn, &dto.ranking_view_id, &items).unwrap();

        // 模拟第二次排名，repo 2 升到第 1
        let items2 = vec![
            RankingItemDto {
                repo_id: 2,
                full_name: "o/b".into(),
                html_url: "https://github.com/o/b".into(),
                description: None,
                primary_language: Some("Rust".into()),
                stars: 600,
                forks: 60,
                rank: 1,
                score: 0.95,
                score_breakdown: None,
                rank_change: None,
                is_subscribed: false,
            },
            RankingItemDto {
                repo_id: 1,
                full_name: "o/a".into(),
                html_url: "https://github.com/o/a".into(),
                description: None,
                primary_language: Some("Rust".into()),
                stars: 500,
                forks: 50,
                rank: 2,
                score: 0.8,
                score_breakdown: None,
                rank_change: None,
                is_subscribed: false,
            },
        ];

        let changes = get_rank_change(&conn, &dto.ranking_view_id, &items2).unwrap();
        assert_eq!(changes.len(), 2);
        // repo 2: prev_rank=2, current=1 → change = +1
        assert_eq!(changes[0], Some(1));
        // repo 1: prev_rank=1, current=2 → change = -1
        assert_eq!(changes[1], Some(-1));
    }

    #[test]
    fn get_rank_change_no_history() {
        let conn = setup_db();
        let dto = create_view(&conn, sample_request()).unwrap();

        let items = vec![RankingItemDto {
            repo_id: 1,
            full_name: "o/a".into(),
            html_url: "https://github.com/o/a".into(),
            description: None,
            primary_language: None,
            stars: 100,
            forks: 10,
            rank: 1,
            score: 0.5,
            score_breakdown: None,
            rank_change: None,
            is_subscribed: false,
        }];

        let changes = get_rank_change(&conn, &dto.ranking_view_id, &items).unwrap();
        assert_eq!(changes, vec![None]); // 首次无历史
    }

    #[test]
    fn create_view_clamps_k_value() {
        let conn = setup_db();
        let mut req = sample_request();
        req.k_value = 0;
        let dto = create_view(&conn, req).unwrap();
        assert_eq!(dto.k_value, 1);

        let mut req2 = sample_request();
        req2.name = "Big K".into();
        req2.k_value = 2000;
        let dto2 = create_view(&conn, req2).unwrap();
        assert_eq!(dto2.k_value, 1000);
    }
}
