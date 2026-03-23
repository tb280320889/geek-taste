//! Ranking DTO — 前端展示用契约对象

use serde::{Deserialize, Serialize};

use domain::ranking::{RankingFilters, RankingView};

/// 榜单视图规格 DTO（前端展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingViewSpecDto {
    pub ranking_view_id: String,
    pub name: String,
    pub view_kind: String,
    pub filters: FiltersDto,
    pub ranking_mode: String,
    pub k_value: i32,
    pub is_pinned: bool,
    pub created_at: String,
}

/// 过滤条件 DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FiltersDto {
    pub language: Vec<String>,
    pub exclude_archived: bool,
    pub exclude_forks: bool,
    pub min_stars: Option<i64>,
    pub updated_since_days: Option<i32>,
    pub topic: Vec<String>,
}

/// 榜单项 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingItemDto {
    pub repo_id: i64,
    pub full_name: String,
    pub html_url: String,
    pub description: Option<String>,
    pub primary_language: Option<String>,
    pub stars: i64,
    pub forks: i64,
    pub rank: i32,
    pub score: f64,
    pub score_breakdown: Option<ScoreBreakdownDto>,
    /// +N / -N / None（首次）
    pub rank_change: Option<i32>,
    pub is_subscribed: bool,
}

/// 评分细分 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdownDto {
    pub star_delta: f64,
    pub fork_delta: f64,
    pub updated_recency: f64,
}

/// 创建榜单视图请求 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRankingViewRequest {
    pub name: String,
    pub filters: FiltersDto,
    pub ranking_mode: String,
    pub k_value: i32,
}

// ── From 转换 ──────────────────────────────────────────

impl From<RankingFilters> for FiltersDto {
    fn from(f: RankingFilters) -> Self {
        Self {
            language: f.language,
            exclude_archived: f.exclude_archived,
            exclude_forks: f.exclude_forks,
            min_stars: f.min_stars,
            updated_since_days: f.updated_since_days,
            topic: f.topic,
        }
    }
}

impl From<RankingView> for RankingViewSpecDto {
    fn from(v: RankingView) -> Self {
        Self {
            ranking_view_id: v.ranking_view_id,
            name: v.name,
            view_kind: v.view_kind,
            filters: v.filters.into(),
            ranking_mode: v.ranking_mode.to_string(),
            k_value: v.k_value,
            is_pinned: v.is_pinned,
            created_at: v.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ranking::{RankingFilters, RankingMode, RankingView};

    #[test]
    fn filters_dto_from_ranking_filters() {
        let filters = RankingFilters {
            language: vec!["Rust".into()],
            exclude_archived: true,
            exclude_forks: true,
            min_stars: Some(100),
            updated_since_days: Some(30),
            topic: vec!["cli".into()],
        };
        let dto: FiltersDto = filters.into();
        assert_eq!(dto.language, vec!["Rust"]);
        assert!(dto.exclude_archived);
        assert_eq!(dto.min_stars, Some(100));
        assert_eq!(dto.updated_since_days, Some(30));
        assert_eq!(dto.topic, vec!["cli"]);
    }

    #[test]
    fn ranking_view_spec_dto_from_ranking_view() {
        let view = RankingView {
            ranking_view_id: "rv_01".into(),
            name: "Rust Hot".into(),
            view_kind: "PRESET".into(),
            query_template: "language:rust stars:>100".into(),
            filters: RankingFilters {
                language: vec!["Rust".into()],
                exclude_archived: true,
                exclude_forks: true,
                min_stars: Some(100),
                updated_since_days: None,
                topic: vec![],
            },
            ranking_mode: RankingMode::StarsDesc,
            k_value: 50,
            is_pinned: true,
            created_at: "2026-03-23T00:00:00Z".into(),
            updated_at: "2026-03-23T00:00:00Z".into(),
            last_snapshot_at: None,
        };
        let dto: RankingViewSpecDto = view.into();
        assert_eq!(dto.ranking_view_id, "rv_01");
        assert_eq!(dto.name, "Rust Hot");
        assert_eq!(dto.ranking_mode, "STARS_DESC");
        assert!(dto.is_pinned);
        assert_eq!(dto.filters.language, vec!["Rust"]);
        assert_eq!(dto.k_value, 50);
    }

    #[test]
    fn dto_serialization_round_trip() {
        let dto = RankingViewSpecDto {
            ranking_view_id: "rv_02".into(),
            name: "Test".into(),
            view_kind: "CUSTOM".into(),
            filters: FiltersDto {
                language: vec![],
                exclude_archived: false,
                exclude_forks: false,
                min_stars: None,
                updated_since_days: None,
                topic: vec![],
            },
            ranking_mode: "MOMENTUM_7D".into(),
            k_value: 30,
            is_pinned: false,
            created_at: "2026-03-23T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: RankingViewSpecDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back.ranking_view_id, "rv_02");
        assert_eq!(back.ranking_mode, "MOMENTUM_7D");
    }

    #[test]
    fn ranking_item_dto_serializes() {
        let item = RankingItemDto {
            repo_id: 123,
            full_name: "owner/repo".into(),
            html_url: "https://github.com/owner/repo".into(),
            description: Some("A cool project".into()),
            primary_language: Some("Rust".into()),
            stars: 500,
            forks: 50,
            rank: 1,
            score: 0.95,
            score_breakdown: Some(ScoreBreakdownDto {
                star_delta: 0.48,
                fork_delta: 0.12,
                updated_recency: 0.35,
            }),
            rank_change: Some(3),
            is_subscribed: false,
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: RankingItemDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rank, 1);
        assert_eq!(back.rank_change, Some(3));
        assert!(back.score_breakdown.is_some());
        let bd = back.score_breakdown.unwrap();
        assert!((bd.star_delta - 0.48).abs() < 1e-10);
    }

    #[test]
    fn create_ranking_view_request_serializes() {
        let req = CreateRankingViewRequest {
            name: "My View".into(),
            filters: FiltersDto {
                language: vec!["Go".into()],
                exclude_archived: true,
                exclude_forks: true,
                min_stars: Some(50),
                updated_since_days: Some(7),
                topic: vec![],
            },
            ranking_mode: "MOMENTUM_7D".into(),
            k_value: 30,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: CreateRankingViewRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "My View");
        assert_eq!(back.ranking_mode, "MOMENTUM_7D");
    }
}
