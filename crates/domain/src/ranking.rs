//! Ranking 领域对象 + Momentum 评分计算

use std::fmt;

/// 排序模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RankingMode {
    /// 按星数降序
    StarsDesc,
    /// 按更新时间降序
    UpdatedDesc,
    /// 24h 动量
    Momentum24h,
    /// 7d 动量
    Momentum7d,
}

impl fmt::Display for RankingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StarsDesc => write!(f, "STARS_DESC"),
            Self::UpdatedDesc => write!(f, "UPDATED_DESC"),
            Self::Momentum24h => write!(f, "MOMENTUM_24H"),
            Self::Momentum7d => write!(f, "MOMENTUM_7D"),
        }
    }
}

impl RankingMode {
    /// 从字符串解析 RankingMode
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "STARS_DESC" => Some(Self::StarsDesc),
            "UPDATED_DESC" => Some(Self::UpdatedDesc),
            "MOMENTUM_24H" => Some(Self::Momentum24h),
            "MOMENTUM_7D" => Some(Self::Momentum7d),
            _ => None,
        }
    }
}

/// 榜单过滤条件
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RankingFilters {
    /// 按语言过滤
    pub language: Vec<String>,
    /// 排除 archived
    pub exclude_archived: bool,
    /// 排除 forks
    pub exclude_forks: bool,
    /// 最小星数阈值
    pub min_stars: Option<i64>,
    /// 最近 N 天内更新
    pub updated_since_days: Option<i32>,
    /// 按 topic 过滤
    pub topic: Vec<String>,
}

impl RankingFilters {
    /// 创建默认过滤器（排除 archived 和 forks）
    pub fn new() -> Self {
        Self {
            exclude_archived: true,
            exclude_forks: true,
            ..Default::default()
        }
    }
}

/// 榜单视图 — 对应 ranking_views 表
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RankingView {
    pub ranking_view_id: String,
    pub name: String,
    /// "PRESET" | "CUSTOM"
    pub view_kind: String,
    pub query_template: String,
    pub filters: RankingFilters,
    pub ranking_mode: RankingMode,
    /// K 值 (1-1000)
    pub k_value: i32,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_snapshot_at: Option<String>,
}

/// 快照中的单项
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RankingSnapshotItem {
    pub repo_id: i64,
    pub full_name: String,
    pub rank: i32,
    pub score: f64,
    pub is_subscribed: bool,
}

/// 快照统计
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotStats {
    pub total_count: i32,
    pub new_count: i32,
    pub changed_count: i32,
}

/// 快照 — 对应 ranking_snapshots 表
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RankingSnapshot {
    pub ranking_snapshot_id: String,
    pub ranking_view_id: String,
    pub snapshot_at: String,
    pub ranking_mode: RankingMode,
    pub items: Vec<RankingSnapshotItem>,
    pub stats: SnapshotStats,
}

/// Momentum 评分结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MomentumScore {
    /// 综合分 (0.0-1.0)
    pub total: f64,
    /// star 增量归一化 (0.0-1.0)
    pub star_delta_norm: f64,
    /// fork 增量归一化 (0.0-1.0)
    pub fork_delta_norm: f64,
    /// 更新近度归一化 (0.0-1.0)
    pub updated_recency_norm: f64,
}

/// 将值 clamp 到 [0.0, 1.0]
fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

/// 计算 Momentum 评分
///
/// 公式: 0.5 × star_delta_norm + 0.2 × fork_delta_norm + 0.3 × updated_recency_norm
pub fn compute_momentum(
    current_stars: i64,
    prev_stars: i64,
    current_forks: i64,
    prev_forks: i64,
    days_since_update: f64,
    max_delta: f64,
) -> MomentumScore {
    let star_delta_norm = clamp01((current_stars - prev_stars) as f64 / max_delta);
    let fork_delta_norm = clamp01((current_forks - prev_forks) as f64 / max_delta);
    let updated_recency_norm = clamp01(1.0 - days_since_update / 30.0);

    let total = 0.5 * star_delta_norm + 0.2 * fork_delta_norm + 0.3 * updated_recency_norm;

    MomentumScore {
        total,
        star_delta_norm,
        fork_delta_norm,
        updated_recency_norm,
    }
}

impl MomentumScore {
    /// 使用默认 max_delta (1000.0) 的便捷构造
    pub fn compute(
        current_stars: i64,
        prev_stars: i64,
        current_forks: i64,
        prev_forks: i64,
        days_since_update: f64,
    ) -> Self {
        compute_momentum(
            current_stars,
            prev_stars,
            current_forks,
            prev_forks,
            days_since_update,
            1000.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranking_mode_display() {
        assert_eq!(RankingMode::StarsDesc.to_string(), "STARS_DESC");
        assert_eq!(RankingMode::UpdatedDesc.to_string(), "UPDATED_DESC");
        assert_eq!(RankingMode::Momentum24h.to_string(), "MOMENTUM_24H");
        assert_eq!(RankingMode::Momentum7d.to_string(), "MOMENTUM_7D");
    }

    #[test]
    fn ranking_mode_from_str_round_trip() {
        for mode in [
            RankingMode::StarsDesc,
            RankingMode::UpdatedDesc,
            RankingMode::Momentum24h,
            RankingMode::Momentum7d,
        ] {
            let s = mode.to_string();
            let back = RankingMode::from_str(&s);
            assert_eq!(back, Some(mode), "round-trip failed for {s}");
        }
    }

    #[test]
    fn ranking_mode_from_str_invalid() {
        assert_eq!(RankingMode::from_str("INVALID"), None);
        assert_eq!(RankingMode::from_str(""), None);
    }

    #[test]
    fn ranking_filters_default_excludes_archived_and_forks() {
        let filters = RankingFilters::new();
        assert!(filters.exclude_archived);
        assert!(filters.exclude_forks);
        assert!(filters.language.is_empty());
        assert!(filters.min_stars.is_none());
    }

    #[test]
    fn momentum_zero_growth() {
        // 没有增长，很久没更新 → 最低分
        let score = MomentumScore::compute(100, 100, 10, 10, 30.0);
        assert_eq!(score.star_delta_norm, 0.0);
        assert_eq!(score.fork_delta_norm, 0.0);
        assert_eq!(score.updated_recency_norm, 0.0);
        assert_eq!(score.total, 0.0);
    }

    #[test]
    fn momentum_max_score() {
        // 刚更新 + 大量增长 → 最高分
        let score = MomentumScore::compute(2000, 0, 200, 0, 0.0);
        // star: 2000/1000 = 2.0 → clamp to 1.0
        assert_eq!(score.star_delta_norm, 1.0);
        // fork: 200/1000 = 0.2
        assert!((score.fork_delta_norm - 0.2).abs() < 1e-10);
        // recency: 1.0 - 0/30 = 1.0
        assert_eq!(score.updated_recency_norm, 1.0);
        // total: 0.5*1.0 + 0.2*0.2 + 0.3*1.0 = 0.5 + 0.04 + 0.3 = 0.84
        assert!((score.total - 0.84).abs() < 1e-10);
    }

    #[test]
    fn momentum_recency_decay() {
        // 刚更新的 recency 高，30 天前的 recency 为 0
        let recent = MomentumScore::compute(0, 0, 0, 0, 0.0);
        assert_eq!(recent.updated_recency_norm, 1.0);

        let old = MomentumScore::compute(0, 0, 0, 0, 30.0);
        assert_eq!(old.updated_recency_norm, 0.0);

        let mid = MomentumScore::compute(0, 0, 0, 0, 15.0);
        assert!((mid.updated_recency_norm - 0.5).abs() < 1e-10);
    }

    #[test]
    fn momentum_negative_delta_clamps_to_zero() {
        let score = MomentumScore::compute(50, 100, 5, 10, 5.0);
        assert_eq!(score.star_delta_norm, 0.0);
        assert_eq!(score.fork_delta_norm, 0.0);
    }

    #[test]
    fn momentum_custom_max_delta() {
        let score = compute_momentum(50, 0, 10, 0, 0.0, 50.0);
        // star: 50/50 = 1.0
        assert!((score.star_delta_norm - 1.0).abs() < 1e-10);
        // fork: 10/50 = 0.2
        assert!((score.fork_delta_norm - 0.2).abs() < 1e-10);
    }

    #[test]
    fn ranking_view_serializes() {
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
            is_pinned: false,
            created_at: "2026-03-23T00:00:00Z".into(),
            updated_at: "2026-03-23T00:00:00Z".into(),
            last_snapshot_at: None,
        };
        let json = serde_json::to_string(&view).unwrap();
        let back: RankingView = serde_json::from_str(&json).unwrap();
        assert_eq!(back.ranking_view_id, "rv_01");
        assert_eq!(back.ranking_mode, RankingMode::StarsDesc);
    }

    #[test]
    fn ranking_snapshot_serializes() {
        let snap = RankingSnapshot {
            ranking_snapshot_id: "rs_01".into(),
            ranking_view_id: "rv_01".into(),
            snapshot_at: "2026-03-23T06:00:00Z".into(),
            ranking_mode: RankingMode::Momentum7d,
            items: vec![RankingSnapshotItem {
                repo_id: 123,
                full_name: "owner/repo".into(),
                rank: 1,
                score: 0.95,
                is_subscribed: false,
            }],
            stats: SnapshotStats {
                total_count: 50,
                new_count: 3,
                changed_count: 7,
            },
        };
        let json = serde_json::to_string(&snap).unwrap();
        let back: RankingSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].rank, 1);
        assert_eq!(back.stats.total_count, 50);
    }
}
