//! Resource DTO — 前端展示用契约对象

use serde::{Deserialize, Serialize};

/// 资源卡片 DTO — 前端契约对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCardDto {
    pub resource_id: String,
    pub resource_kind: String,
    pub title: String,
    pub source_repo_id: Option<i64>,
    pub source_url: String,
    pub languages: Vec<String>,
    pub framework_tags: Vec<String>,
    pub agent_tags: Vec<String>,
    pub score: f64,
    pub why_recommended: Vec<String>,
    pub is_curated: bool,
    pub is_active: bool,
}

/// 资源列表请求 DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceListRequest {
    pub tag_type: Option<String>,
    pub tag_value: Option<String>,
    pub resource_kind: Option<String>,
    pub language: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 精选操作请求 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurateResourceRequest {
    pub resource_id: String,
    /// "add" | "remove"
    pub action: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_card_dto_serializes() {
        let dto = ResourceCardDto {
            resource_id: "res_01".into(),
            resource_kind: "MCP_SERVER".into(),
            title: "acme/mcp-rust-tools".into(),
            source_repo_id: Some(42),
            source_url: "https://github.com/acme/mcp-rust-tools".into(),
            languages: vec!["Rust".into()],
            framework_tags: vec!["Axum".into()],
            agent_tags: vec!["MCP".into()],
            score: 0.91,
            why_recommended: vec!["你关注 Rust，这是 Rust 生态的资源".into()],
            is_curated: true,
            is_active: true,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: ResourceCardDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back.resource_id, "res_01");
        assert_eq!(back.score, 0.91);
        assert!(back.is_curated);
    }

    #[test]
    fn resource_list_request_default() {
        let req = ResourceListRequest::default();
        assert!(req.tag_type.is_none());
        assert!(req.language.is_none());
        assert!(req.limit.is_none());
    }

    #[test]
    fn curate_resource_request_serializes() {
        let req = CurateResourceRequest {
            resource_id: "res_01".into(),
            action: "add".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: CurateResourceRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.resource_id, "res_01");
        assert_eq!(back.action, "add");
    }
}
