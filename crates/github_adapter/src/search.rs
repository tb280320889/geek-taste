//! GitHub Search API client — search_repositories + 速率预算集成

use chrono::Utc;
use thiserror::Error;

use crate::rate_limit::{RateBudget, RateError, RatePool};

/// 搜索排序字段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchSort {
    Stars,
    Updated,
    Created,
}

impl SearchSort {
    fn as_str(&self) -> &'static str {
        match self {
            SearchSort::Stars => "stars",
            SearchSort::Updated => "updated",
            SearchSort::Created => "created",
        }
    }
}

/// 排序方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Asc => "asc",
            SortOrder::Desc => "desc",
        }
    }
}

/// 搜索查询参数
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub language: Option<String>,
    pub topic: Option<String>,
    pub min_stars: Option<u32>,
    pub sort: SearchSort,
    pub order: SortOrder,
    pub per_page: u8,
    pub page: u32,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            language: None,
            topic: None,
            min_stars: None,
            sort: SearchSort::Stars,
            order: SortOrder::Desc,
            per_page: 30,
            page: 1,
        }
    }
}

impl SearchQuery {
    /// 构建 GitHub Search API 查询字符串
    fn build_query_str(&self) -> String {
        let mut parts = Vec::new();

        if let Some(lang) = &self.language {
            parts.push(format!("language:{}", lang));
        }
        if let Some(topic) = &self.topic {
            parts.push(format!("topic:{}", topic));
        }
        if let Some(min) = self.min_stars {
            parts.push(format!("stars:>={}", min));
        }
        parts.push("archived:false".to_string());

        parts.join(" ")
    }
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub items: Vec<domain::repository::Repository>,
    pub total_count: u64,
    pub incomplete_results: bool,
}

/// 搜索错误
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Search API 速率限制: {0}")]
    RateLimited(String),
    #[error("网络错误: {0}")]
    NetworkError(String),
    #[error("API 错误: {0}")]
    ApiError(String),
}

/// 将 SearchError 从 RateError 转换
impl From<RateError> for SearchError {
    fn from(e: RateError) -> Self {
        SearchError::RateLimited(e.to_string())
    }
}

/// 将 octocrab 搜索结果映射为领域 Repository 对象
fn map_octocrab_repo(r: octocrab::models::Repository) -> domain::repository::Repository {
    domain::repository::Repository {
        repo_id: r.id.0 as i64,
        full_name: r.full_name.unwrap_or_default(),
        owner: r.owner.map(|o| o.login.to_string()).unwrap_or_default(),
        name: r.name,
        html_url: r.html_url.map(|u| u.to_string()).unwrap_or_default(),
        description: r.description,
        default_branch: r.default_branch.unwrap_or_else(|| "main".to_string()),
        primary_language: r.language.and_then(|v| match v {
            serde_json::Value::String(s) => Some(s),
            _ => v.as_str().map(|s| s.to_string()),
        }),
        topics: r.topics.unwrap_or_default(),
        archived: r.archived.unwrap_or(false),
        disabled: r.disabled.unwrap_or(false),
        stargazers_count: r.stargazers_count.unwrap_or(0) as i64,
        forks_count: r.forks_count.unwrap_or(0) as i64,
        updated_at: r.updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
        pushed_at: r.pushed_at.map(|d| d.to_rfc3339()),
        last_synced_at: Utc::now().to_rfc3339(),
    }
}

/// 搜索 GitHub 仓库
///
/// 调用前检查速率预算，调用后记录消耗，返回领域 Repository 列表。
pub async fn search_repositories(
    token: &str,
    query: &SearchQuery,
    budget: &RateBudget,
) -> Result<SearchResult, SearchError> {
    // 1. 速率预算检查
    budget.check(RatePool::Search)?;

    // 2. 构建 octocrab 客户端
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|e| SearchError::NetworkError(e.to_string()))?;

    // 3. 构建查询
    let query_str = query.build_query_str();

    // 4. 调用搜索 API
    let page = octocrab
        .search()
        .repositories(&query_str)
        .sort(query.sort.as_str())
        .order(query.order.as_str())
        .per_page(query.per_page)
        .page(query.page)
        .send()
        .await
        .map_err(|e| match e {
            octocrab::Error::GitHub { source, .. } => {
                if source.status_code.as_u16() == 403 || source.status_code.as_u16() == 429 {
                    SearchError::RateLimited(source.message.clone())
                } else {
                    SearchError::ApiError(source.message)
                }
            }
            _ => SearchError::NetworkError(e.to_string()),
        })?;

    // 5. 记录速率消耗
    budget.record(RatePool::Search);

    // 6. 映射为领域对象
    let items: Vec<domain::repository::Repository> =
        page.items.into_iter().map(map_octocrab_repo).collect();

    Ok(SearchResult {
        items,
        total_count: page.total_count.unwrap_or(0),
        incomplete_results: page.incomplete_results.unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_query_default_values() {
        let q = SearchQuery::default();
        assert_eq!(q.sort, SearchSort::Stars);
        assert_eq!(q.order, SortOrder::Desc);
        assert_eq!(q.per_page, 30);
        assert_eq!(q.page, 1);
        assert!(q.language.is_none());
        assert!(q.topic.is_none());
        assert!(q.min_stars.is_none());
    }

    #[test]
    fn query_str_archived_false_always() {
        let q = SearchQuery::default();
        let s = q.build_query_str();
        assert_eq!(s, "archived:false");
    }

    #[test]
    fn query_str_with_language() {
        let q = SearchQuery {
            language: Some("Rust".into()),
            ..Default::default()
        };
        let s = q.build_query_str();
        assert!(s.contains("language:Rust"));
        assert!(s.contains("archived:false"));
    }

    #[test]
    fn query_str_with_all_filters() {
        let q = SearchQuery {
            language: Some("Rust".into()),
            topic: Some("cli".into()),
            min_stars: Some(100),
            ..Default::default()
        };
        let s = q.build_query_str();
        assert!(s.contains("language:Rust"));
        assert!(s.contains("topic:cli"));
        assert!(s.contains("stars:>=100"));
        assert!(s.contains("archived:false"));
    }

    #[test]
    fn search_sort_as_str() {
        assert_eq!(SearchSort::Stars.as_str(), "stars");
        assert_eq!(SearchSort::Updated.as_str(), "updated");
        assert_eq!(SearchSort::Created.as_str(), "created");
    }

    #[test]
    fn sort_order_as_str() {
        assert_eq!(SortOrder::Asc.as_str(), "asc");
        assert_eq!(SortOrder::Desc.as_str(), "desc");
    }

    #[test]
    fn search_error_from_rate_limit() {
        let rate_err = RateError::SearchExceeded(Utc::now());
        let search_err: SearchError = rate_err.into();
        match search_err {
            SearchError::RateLimited(msg) => assert!(msg.contains("Search API")),
            _ => panic!("expected RateLimited"),
        }
    }
}
