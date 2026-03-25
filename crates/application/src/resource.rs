//! Resource 用例编排 — 评分、解释、精选、列表

use anyhow::{Context, Result};
use chrono::Utc;
use domain::resource::{
    compute_resource_score, CurationLevel, RecommendationReason, Resource, ResourceKind,
    ResourceScore,
};
use persistence_sqlite::resource_repository;
use rusqlite::Connection;
use shared_contracts::resource_dto::ResourceCardDto;

/// 计算 stack_relevance
/// 0.7 * settings_language_overlap + 0.3 * subscription_language_overlap
/// 使用 Jaccard 相似度: |intersection| / |union|
fn compute_stack_relevance(conn: &Connection, resource_languages: &[String]) -> Result<f64> {
    if resource_languages.is_empty() {
        return Ok(0.0);
    }

    // 从已订阅仓库中提取用户语言兴趣
    let subscriptions =
        persistence_sqlite::subscription_repository::list_active_subscriptions(conn)?;
    let mut user_langs: Vec<String> = Vec::new();
    for sub in &subscriptions {
        if let Some(repo) = persistence_sqlite::repo_repository::get_repository(conn, sub.repo_id)?
        {
            if let Some(lang) = repo.primary_language {
                if !user_langs.contains(&lang) {
                    user_langs.push(lang);
                }
            }
        }
    }

    if user_langs.is_empty() {
        return Ok(0.0);
    }

    // Jaccard 相似度
    let user_set: std::collections::HashSet<&str> = user_langs.iter().map(|s| s.as_str()).collect();
    let resource_set: std::collections::HashSet<&str> =
        resource_languages.iter().map(|s| s.as_str()).collect();

    let intersection_size = user_set.intersection(&resource_set).count() as f64;
    let union_size = user_set.union(&resource_set).count() as f64;

    if union_size == 0.0 {
        return Ok(0.0);
    }

    Ok(intersection_size / union_size)
}

/// 计算 recency_norm: 1.0 - days_since_push / 365, clamped [0,1]
fn compute_recency_norm(last_pushed_at: Option<&str>) -> f64 {
    match last_pushed_at {
        None => 0.0,
        Some(pushed_at) => {
            let pushed_dt = chrono::DateTime::parse_from_rfc3339(pushed_at);
            match pushed_dt {
                Err(_) => 0.0,
                Ok(dt) => {
                    let days = (Utc::now() - dt.with_timezone(&Utc)).num_seconds() as f64 / 86400.0;
                    (1.0 - days / 365.0).clamp(0.0, 1.0)
                }
            }
        }
    }
}

/// 计算 star_delta_norm: 同 MomentumScore 的 star_delta_norm 逻辑
fn compute_star_delta_norm(conn: &Connection, repo_id: i64) -> Result<f64> {
    let repo = persistence_sqlite::repo_repository::get_repository(conn, repo_id)?
        .context(format!("Repository not found: {repo_id}"))?;

    let prev_snap = persistence_sqlite::repo_repository::get_latest_repo_snapshot(conn, repo_id)?;
    let prev_stars = prev_snap.map(|s| s.stargazers_count).unwrap_or(0);

    let delta = (repo.stargazers_count - prev_stars) as f64;
    Ok((delta / 1000.0).clamp(0.0, 1.0))
}

/// 为单个资源生成评分 + 推荐解释
pub fn compute_resource_score_with_explanation(
    conn: &Connection,
    resource: &Resource,
) -> Result<(ResourceScore, Vec<String>)> {
    let stack_relevance = compute_stack_relevance(conn, &resource.languages)?;
    let star_delta_norm = resource
        .source_repo_id
        .map(|id| compute_star_delta_norm(conn, id).unwrap_or(0.0))
        .unwrap_or(0.0);

    // recency: 从 repositories 获取 pushed_at
    let recency_norm = if let Some(repo_id) = resource.source_repo_id {
        let repo = persistence_sqlite::repo_repository::get_repository(conn, repo_id)?;
        match repo {
            Some(r) => compute_recency_norm(r.pushed_at.as_deref()),
            None => 0.0,
        }
    } else {
        // 无关联仓库，基于 last_scored_at 近似
        compute_recency_norm(resource.last_scored_at.as_deref())
    };

    let score = compute_resource_score(stack_relevance, star_delta_norm, recency_norm);

    // 生成推荐解释
    let mut reasons = Vec::new();
    if resource.curation_level == CurationLevel::UserCurated {
        reasons.push(RecommendationReason::UserCurated.to_template(&resource.title));
    }
    for lang in &resource.languages {
        if stack_relevance > 0.3 {
            reasons.push(
                RecommendationReason::LanguageMatch(lang.clone()).to_template(&resource.title),
            );
            break;
        }
    }
    for tag in &resource.framework_tags {
        reasons
            .push(RecommendationReason::FrameworkMatch(tag.clone()).to_template(&resource.title));
    }
    if star_delta_norm > 0.5 {
        reasons.push(
            RecommendationReason::GrowthSignal((star_delta_norm * 1000.0) as i64)
                .to_template(&resource.title),
        );
    }
    if reasons.is_empty() {
        reasons.push(format!("发现新资源: {}", resource.title));
    }

    Ok((score, reasons))
}

/// 转换 Resource 列表为 ResourceCardDto 列表（含评分和解释）
fn resources_to_dtos(conn: &Connection, resources: Vec<Resource>) -> Result<Vec<ResourceCardDto>> {
    let mut dtos = Vec::new();
    for resource in resources {
        let (score, why_recommended) = compute_resource_score_with_explanation(conn, &resource)?;
        dtos.push(ResourceCardDto {
            resource_id: resource.resource_id.clone(),
            resource_kind: resource.resource_kind.to_string(),
            title: resource.title.clone(),
            source_repo_id: resource.source_repo_id,
            source_url: resource.source_url.clone(),
            languages: resource.languages.clone(),
            framework_tags: resource.framework_tags.clone(),
            agent_tags: resource.agent_tags.clone(),
            score: score.total,
            why_recommended,
            is_curated: resource.curation_level == CurationLevel::UserCurated,
            is_active: resource.is_active,
        });
    }
    // 按 score 降序排列
    dtos.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(dtos)
}

/// 列出资源并计算评分
pub fn list_resources(conn: &Connection, limit: i64, offset: i64) -> Result<Vec<ResourceCardDto>> {
    let resources = resource_repository::list_resources(conn, limit, offset)?;
    resources_to_dtos(conn, resources)
}

/// 搜索资源并计算评分
pub fn search_resources(
    conn: &Connection,
    tag_type: Option<&str>,
    tag_value: Option<&str>,
    resource_kind: Option<ResourceKind>,
    language: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ResourceCardDto>> {
    let resources = resource_repository::search_resources(
        conn,
        tag_type,
        tag_value,
        resource_kind,
        language,
        true,
        limit,
        offset,
    )?;
    resources_to_dtos(conn, resources)
}

/// 用户精选资源
pub fn curate_resource(
    conn: &Connection,
    resource_id: &str,
    action: &str,
) -> Result<ResourceCardDto> {
    let level = match action {
        "add" => CurationLevel::UserCurated,
        "remove" => CurationLevel::SystemDiscovered,
        _ => anyhow::bail!("Invalid curate action: {action}"),
    };

    resource_repository::update_curation_level(conn, resource_id, &level)?;

    let resource = resource_repository::get_resource(conn, resource_id)?
        .context(format!("Resource not found: {resource_id}"))?;

    let (score, why_recommended) = compute_resource_score_with_explanation(conn, &resource)?;

    Ok(ResourceCardDto {
        resource_id: resource.resource_id.clone(),
        resource_kind: resource.resource_kind.to_string(),
        title: resource.title.clone(),
        source_repo_id: resource.source_repo_id,
        source_url: resource.source_url.clone(),
        languages: resource.languages.clone(),
        framework_tags: resource.framework_tags.clone(),
        agent_tags: resource.agent_tags.clone(),
        score: score.total,
        why_recommended,
        is_curated: resource.curation_level == CurationLevel::UserCurated,
        is_active: resource.is_active,
    })
}

/// 停用资源（设为 is_active = false）
pub fn deactivate_resource(conn: &Connection, resource_id: &str) -> Result<()> {
    resource_repository::deactivate_resource(conn, resource_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use persistence_sqlite::init_db;
    use shared_contracts::resource_dto::ResourceCardDto;

    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();
        conn
    }

    fn insert_test_resource(conn: &Connection, id: &str, title: &str) {
        let resource = Resource {
            resource_id: id.into(),
            source_repo_id: None,
            resource_kind: ResourceKind::McpServer,
            title: title.into(),
            summary: Some("A test resource".into()),
            source_url: "https://github.com/test/test".into(),
            languages: vec!["Rust".into()],
            framework_tags: vec!["Axum".into()],
            agent_tags: vec!["MCP".into()],
            curation_level: CurationLevel::SystemDiscovered,
            last_scored_at: None,
            is_active: true,
        };
        resource_repository::insert_resource(conn, &resource).unwrap();
    }

    #[test]
    fn list_resources_returns_dtos() {
        let conn = setup_db();
        insert_test_resource(&conn, "res_01", "Test MCP");
        insert_test_resource(&conn, "res_02", "Another MCP");

        let dtos = list_resources(&conn, 50, 0).unwrap();
        assert_eq!(dtos.len(), 2);
        assert!(dtos[0].score >= 0.0);
        assert!(!dtos[0].why_recommended.is_empty());
    }

    #[test]
    fn curate_resource_add() {
        let conn = setup_db();
        insert_test_resource(&conn, "res_01", "Test MCP");

        let dto = curate_resource(&conn, "res_01", "add").unwrap();
        assert!(dto.is_curated);
        assert!(dto.why_recommended.iter().any(|r| r.contains("精选")));
    }

    #[test]
    fn curate_resource_remove() {
        let conn = setup_db();
        insert_test_resource(&conn, "res_01", "Test MCP");

        curate_resource(&conn, "res_01", "add").unwrap();
        let dto = curate_resource(&conn, "res_01", "remove").unwrap();
        assert!(!dto.is_curated);
    }

    #[test]
    fn curate_resource_invalid_action() {
        let conn = setup_db();
        insert_test_resource(&conn, "res_01", "Test MCP");

        let result = curate_resource(&conn, "res_01", "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn deactivate_resource_works() {
        let conn = setup_db();
        insert_test_resource(&conn, "res_01", "Test MCP");

        deactivate_resource(&conn, "res_01").unwrap();

        let dtos = list_resources(&conn, 50, 0).unwrap();
        assert!(dtos.is_empty());
    }

    #[test]
    fn compute_recency_norm_values() {
        // None → 0.0
        assert_eq!(compute_recency_norm(None), 0.0);

        // 刚推送 → 接近 1.0
        let now = Utc::now().to_rfc3339();
        let recent = compute_recency_norm(Some(&now));
        assert!(recent > 0.99);

        // 180 天前 → 约 0.5
        let half_year = (Utc::now() - chrono::Duration::days(180)).to_rfc3339();
        let mid = compute_recency_norm(Some(&half_year));
        assert!((mid - 0.5).abs() < 0.02);

        // 超过 365 天 → 0.0
        let old = (Utc::now() - chrono::Duration::days(400)).to_rfc3339();
        assert_eq!(compute_recency_norm(Some(&old)), 0.0);
    }

    #[test]
    fn search_resources_returns_dtos() {
        let conn = setup_db();
        insert_test_resource(&conn, "res_01", "Rust MCP");

        let mut resource2 = Resource {
            resource_id: "res_02".into(),
            source_repo_id: None,
            resource_kind: ResourceKind::Skill,
            title: "Python MCP".into(),
            summary: None,
            source_url: "https://github.com/test/python".into(),
            languages: vec!["Python".into()],
            framework_tags: vec![],
            agent_tags: vec!["MCP".into()],
            curation_level: CurationLevel::SystemDiscovered,
            last_scored_at: None,
            is_active: true,
        };
        resource_repository::insert_resource(&conn, &resource2).unwrap();

        let results = search_resources(
            &conn,
            None,
            None,
            Some(ResourceKind::McpServer),
            None,
            50,
            0,
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].resource_id, "res_01");
    }
}
