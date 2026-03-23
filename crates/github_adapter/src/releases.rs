//! GitHub Releases + Tags 获取 — 支持 cursor-based 增量同步

/// Release 信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReleaseInfo {
    pub release_id: i64,
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub prerelease: bool,
    pub published_at: Option<String>,
    pub html_url: String,
}

/// Tag 信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TagInfo {
    pub name: String,
    pub sha: String,
    pub commit_url: Option<String>,
}

/// 获取仓库最新 releases（支持 cursor 增量过滤）
///
/// - `since_cursor`: 如果提供，只返回 published_at > since_cursor 的 releases
pub async fn fetch_latest_releases(
    token: &str,
    owner: &str,
    repo: &str,
    since_cursor: Option<&str>,
) -> Result<Vec<ReleaseInfo>, String> {
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|e| e.to_string())?;

    let releases = octocrab
        .repos(owner, repo)
        .releases()
        .list()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for release in releases.items {
        let published_at = release.published_at.map(|t| t.to_rfc3339());

        // cursor 过滤: 跳过已处理的
        if let (Some(cursor), Some(ref pub_at)) = (since_cursor, &published_at) {
            if pub_at.as_str() <= cursor {
                continue;
            }
        }

        results.push(ReleaseInfo {
            release_id: release.id.0 as i64,
            tag_name: release.tag_name,
            name: release.name,
            body: release.body,
            prerelease: release.prerelease,
            published_at,
            html_url: release.html_url.to_string(),
        });
    }

    Ok(results)
}

/// 获取仓库最新 tags（支持 cursor 增量过滤）
///
/// - `since_tag`: 如果提供，只返回名称排在 since_tag 之后的 tags
pub async fn fetch_latest_tags(
    token: &str,
    owner: &str,
    repo: &str,
    since_tag: Option<&str>,
) -> Result<Vec<TagInfo>, String> {
    let octocrab = octocrab::Octocrab::builder()
        .personal_token(token.to_string())
        .build()
        .map_err(|e| e.to_string())?;

    let tags = octocrab
        .repos(owner, repo)
        .list_tags()
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for tag in tags.items {
        // cursor 过滤: 跳过已处理的 tag
        if let Some(cursor) = since_tag {
            if tag.name == cursor {
                break; // tags 是按时间倒序，遇到 cursor 就停
            }
        }

        results.push(TagInfo {
            name: tag.name,
            sha: tag.commit.sha,
            commit_url: Some(tag.commit.url.to_string()),
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_info_struct_fields() {
        let info = ReleaseInfo {
            release_id: 123,
            tag_name: "v1.0.0".into(),
            name: Some("First release".into()),
            body: Some("Release notes".into()),
            prerelease: false,
            published_at: Some("2026-03-23T00:00:00+00:00".into()),
            html_url: "https://github.com/o/r/releases/tag/v1.0.0".into(),
        };
        assert_eq!(info.tag_name, "v1.0.0");
        assert!(!info.prerelease);
    }

    #[test]
    fn tag_info_struct_fields() {
        let info = TagInfo {
            name: "v0.9.0".into(),
            sha: "abc123".into(),
            commit_url: Some("https://api.github.com/repos/o/r/commits/abc123".into()),
        };
        assert_eq!(info.name, "v0.9.0");
    }

    #[test]
    fn release_info_serializes_round_trip() {
        let info = ReleaseInfo {
            release_id: 1,
            tag_name: "v1".into(),
            name: None,
            body: None,
            prerelease: true,
            published_at: None,
            html_url: "https://example.com".into(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: ReleaseInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.release_id, 1);
        assert!(back.prerelease);
    }
}
