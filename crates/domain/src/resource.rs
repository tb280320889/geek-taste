//! Resource 领域对象 — Agent 资源雷达

use std::fmt;

/// 资源类型
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ResourceKind {
    McpServer,
    Skill,
    AgentFramework,
    Other(String),
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::McpServer => write!(f, "MCP_SERVER"),
            Self::Skill => write!(f, "SKILL"),
            Self::AgentFramework => write!(f, "AGENT_FRAMEWORK"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}

impl ResourceKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "MCP_SERVER" => Some(Self::McpServer),
            "SKILL" => Some(Self::Skill),
            "AGENT_FRAMEWORK" => Some(Self::AgentFramework),
            _ => Some(Self::Other(s.to_string())),
        }
    }
}

/// 精选级别
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CurationLevel {
    UserCurated,
    SystemDiscovered,
}

impl fmt::Display for CurationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserCurated => write!(f, "USER_CURATED"),
            Self::SystemDiscovered => write!(f, "SYSTEM_DISCOVERED"),
        }
    }
}

impl CurationLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "USER_CURATED" => Some(Self::UserCurated),
            "SYSTEM_DISCOVERED" => Some(Self::SystemDiscovered),
            _ => None,
        }
    }
}

/// 资源标签
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceTag {
    pub resource_id: String,
    pub tag_type: String,
    pub tag_value: String,
}

/// 资源领域对象 — 对应 resources 表
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Resource {
    pub resource_id: String,
    pub source_repo_id: Option<i64>,
    pub resource_kind: ResourceKind,
    pub title: String,
    pub summary: Option<String>,
    pub source_url: String,
    pub languages: Vec<String>,
    pub framework_tags: Vec<String>,
    pub agent_tags: Vec<String>,
    pub curation_level: CurationLevel,
    pub last_scored_at: Option<String>,
    pub is_active: bool,
}

impl Resource {
    /// 生成去重键（用于 RESOURCE_EMERGED 信号去重）
    pub fn signal_key(&self) -> String {
        format!("{}:RESOURCE_EMERGED", self.resource_id)
    }
}

/// 资源评分结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceScore {
    /// 综合分 (0.0-1.0)
    pub total: f64,
    /// 技术栈相关度 (0.0-1.0)
    pub stack_relevance: f64,
    /// star 增量归一化 (0.0-1.0)
    pub star_delta_norm: f64,
    /// 更新近度归一化 (0.0-1.0)
    pub recency_norm: f64,
}

/// 将值 clamp 到 [0.0, 1.0]
fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

/// 计算资源评分
///
/// 公式: 0.4 × stack_relevance + 0.35 × star_delta_norm + 0.25 × recency_norm
pub fn compute_resource_score(
    stack_relevance: f64,
    star_delta_norm: f64,
    recency_norm: f64,
) -> ResourceScore {
    let stack_relevance = clamp01(stack_relevance);
    let star_delta_norm = clamp01(star_delta_norm);
    let recency_norm = clamp01(recency_norm);

    let total = 0.4 * stack_relevance + 0.35 * star_delta_norm + 0.25 * recency_norm;

    ResourceScore {
        total,
        stack_relevance,
        star_delta_norm,
        recency_norm,
    }
}

/// 推荐理由
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RecommendationReason {
    LanguageMatch(String),
    FrameworkMatch(String),
    GrowthSignal(i64),
    UserCurated,
}

impl RecommendationReason {
    pub fn to_template(&self, resource_title: &str) -> String {
        match self {
            Self::LanguageMatch(lang) => {
                format!("你关注 {lang}，这是 {lang} 生态的资源")
            }
            Self::FrameworkMatch(framework) => {
                format!("这个资源适用于 {framework}")
            }
            Self::GrowthSignal(delta) => {
                format!("近期 star 增长 {delta}")
            }
            Self::UserCurated => {
                format!("你已将 {resource_title} 加入精选列表")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── ResourceKind ──────────────────────────────────────

    #[test]
    fn resource_kind_display() {
        assert_eq!(ResourceKind::McpServer.to_string(), "MCP_SERVER");
        assert_eq!(ResourceKind::Skill.to_string(), "SKILL");
        assert_eq!(ResourceKind::AgentFramework.to_string(), "AGENT_FRAMEWORK");
        assert_eq!(ResourceKind::Other("custom".into()).to_string(), "custom");
    }

    #[test]
    fn resource_kind_from_str_round_trip() {
        for kind in [
            ResourceKind::McpServer,
            ResourceKind::Skill,
            ResourceKind::AgentFramework,
        ] {
            let s = kind.to_string();
            let back = ResourceKind::from_str(&s);
            assert_eq!(back, Some(kind), "round-trip failed for {s}");
        }
    }

    #[test]
    fn resource_kind_from_str_other() {
        let kind = ResourceKind::from_str("UNKNOWN_TYPE");
        assert_eq!(kind, Some(ResourceKind::Other("UNKNOWN_TYPE".into())));
    }

    // ── CurationLevel ────────────────────────────────────

    #[test]
    fn curation_level_display() {
        assert_eq!(CurationLevel::UserCurated.to_string(), "USER_CURATED");
        assert_eq!(
            CurationLevel::SystemDiscovered.to_string(),
            "SYSTEM_DISCOVERED"
        );
    }

    #[test]
    fn curation_level_from_str_round_trip() {
        for level in [CurationLevel::UserCurated, CurationLevel::SystemDiscovered] {
            let s = level.to_string();
            let back = CurationLevel::from_str(&s);
            assert_eq!(back, Some(level), "round-trip failed for {s}");
        }
    }

    #[test]
    fn curation_level_from_str_invalid() {
        assert_eq!(CurationLevel::from_str("INVALID"), None);
        assert_eq!(CurationLevel::from_str(""), None);
    }

    // ── compute_resource_score ───────────────────────────

    #[test]
    fn score_zero() {
        let score = compute_resource_score(0.0, 0.0, 0.0);
        assert_eq!(score.total, 0.0);
    }

    #[test]
    fn score_max() {
        let score = compute_resource_score(1.0, 1.0, 1.0);
        assert!((score.total - 1.0).abs() < 1e-10);
    }

    #[test]
    fn score_partial() {
        // stack=1.0, star=0.5, recency=0.0
        // total = 0.4*1.0 + 0.35*0.5 + 0.25*0.0 = 0.4 + 0.175 = 0.575
        let score = compute_resource_score(1.0, 0.5, 0.0);
        assert!((score.total - 0.575).abs() < 1e-10);
    }

    #[test]
    fn score_clamps_over_one() {
        let score = compute_resource_score(2.0, -1.0, 5.0);
        assert_eq!(score.stack_relevance, 1.0);
        assert_eq!(score.star_delta_norm, 0.0);
        assert_eq!(score.recency_norm, 1.0);
    }

    // ── RecommendationReason ─────────────────────────────

    #[test]
    fn recommendation_language_match() {
        let reason = RecommendationReason::LanguageMatch("Rust".into());
        let text = reason.to_template("acme/mcp-rust-tools");
        assert!(text.contains("Rust"));
    }

    #[test]
    fn recommendation_framework_match() {
        let reason = RecommendationReason::FrameworkMatch("Axum".into());
        let text = reason.to_template("some-resource");
        assert!(text.contains("Axum"));
    }

    #[test]
    fn recommendation_growth_signal() {
        let reason = RecommendationReason::GrowthSignal(42);
        let text = reason.to_template("some-resource");
        assert!(text.contains("42"));
    }

    #[test]
    fn recommendation_user_curated() {
        let reason = RecommendationReason::UserCurated;
        let text = reason.to_template("my-fav");
        assert!(text.contains("my-fav"));
        assert!(text.contains("精选"));
    }

    // ── Resource serialization ───────────────────────────

    #[test]
    fn resource_serializes_round_trip() {
        let resource = Resource {
            resource_id: "res_01".into(),
            source_repo_id: Some(42),
            resource_kind: ResourceKind::McpServer,
            title: "acme/mcp-rust-tools".into(),
            summary: Some("MCP server for Rust".into()),
            source_url: "https://github.com/acme/mcp-rust-tools".into(),
            languages: vec!["Rust".into()],
            framework_tags: vec!["Axum".into()],
            agent_tags: vec!["MCP".into()],
            curation_level: CurationLevel::UserCurated,
            last_scored_at: None,
            is_active: true,
        };
        let json = serde_json::to_string(&resource).unwrap();
        let back: Resource = serde_json::from_str(&json).unwrap();
        assert_eq!(back.resource_id, "res_01");
        assert_eq!(back.resource_kind, ResourceKind::McpServer);
        assert_eq!(back.curation_level, CurationLevel::UserCurated);
        assert_eq!(back.languages, vec!["Rust"]);
    }

    #[test]
    fn resource_signal_key() {
        let resource = Resource {
            resource_id: "res_01".into(),
            source_repo_id: None,
            resource_kind: ResourceKind::Skill,
            title: "test".into(),
            summary: None,
            source_url: "https://example.com".into(),
            languages: vec![],
            framework_tags: vec![],
            agent_tags: vec![],
            curation_level: CurationLevel::SystemDiscovered,
            last_scored_at: None,
            is_active: true,
        };
        assert_eq!(resource.signal_key(), "res_01:RESOURCE_EMERGED");
    }
}
