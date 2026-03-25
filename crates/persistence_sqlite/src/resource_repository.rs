//! Resource 的 SQLite CRUD 实现

use anyhow::Result;
use domain::resource::{CurationLevel, Resource, ResourceKind, ResourceTag};
use rusqlite::{params, Connection};

/// 插入单个资源
pub fn insert_resource(conn: &Connection, resource: &Resource) -> Result<()> {
    let languages_json = serde_json::to_string(&resource.languages)?;
    let framework_tags_json = serde_json::to_string(&resource.framework_tags)?;
    let agent_tags_json = serde_json::to_string(&resource.agent_tags)?;

    conn.execute(
        "INSERT INTO resources
         (resource_id, source_repo_id, resource_kind, title, summary, source_url,
          languages_json, framework_tags_json, agent_tags_json, curation_level,
          last_scored_at, is_active)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            resource.resource_id,
            resource.source_repo_id,
            resource.resource_kind.to_string(),
            resource.title,
            resource.summary,
            resource.source_url,
            languages_json,
            framework_tags_json,
            agent_tags_json,
            resource.curation_level.to_string(),
            resource.last_scored_at,
            resource.is_active as i32,
        ],
    )?;
    Ok(())
}

/// Upsert 资源（INSERT OR REPLACE）
pub fn upsert_resource(conn: &Connection, resource: &Resource) -> Result<()> {
    let languages_json = serde_json::to_string(&resource.languages)?;
    let framework_tags_json = serde_json::to_string(&resource.framework_tags)?;
    let agent_tags_json = serde_json::to_string(&resource.agent_tags)?;

    conn.execute(
        "INSERT OR REPLACE INTO resources
         (resource_id, source_repo_id, resource_kind, title, summary, source_url,
          languages_json, framework_tags_json, agent_tags_json, curation_level,
          last_scored_at, is_active)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            resource.resource_id,
            resource.source_repo_id,
            resource.resource_kind.to_string(),
            resource.title,
            resource.summary,
            resource.source_url,
            languages_json,
            framework_tags_json,
            agent_tags_json,
            resource.curation_level.to_string(),
            resource.last_scored_at,
            resource.is_active as i32,
        ],
    )?;
    Ok(())
}

/// 根据 ID 获取资源
pub fn get_resource(conn: &Connection, resource_id: &str) -> Result<Option<Resource>> {
    let mut stmt = conn.prepare(
        "SELECT resource_id, source_repo_id, resource_kind, title, summary, source_url,
                languages_json, framework_tags_json, agent_tags_json, curation_level,
                last_scored_at, is_active
         FROM resources WHERE resource_id = ?",
    )?;
    let mut rows = stmt.query(params![resource_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_resource(row)?))
    } else {
        Ok(None)
    }
}

/// 列出所有活跃资源，支持分页
pub fn list_resources(conn: &Connection, limit: i64, offset: i64) -> Result<Vec<Resource>> {
    let mut stmt = conn.prepare(
        "SELECT resource_id, source_repo_id, resource_kind, title, summary, source_url,
                languages_json, framework_tags_json, agent_tags_json, curation_level,
                last_scored_at, is_active
         FROM resources WHERE is_active = 1
         ORDER BY curation_level = 'USER_CURATED' DESC, resource_id
         LIMIT ? OFFSET ?",
    )?;
    let rows = stmt.query_map(params![limit, offset], |row| row_to_resource(row))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 搜索资源：按 tag_type+tag_value 过滤，按 curation_level 排序（USER_CURATED 优先）
pub fn search_resources(
    conn: &Connection,
    tag_type: Option<&str>,
    tag_value: Option<&str>,
    resource_kind: Option<ResourceKind>,
    language: Option<&str>,
    min_is_active: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Resource>> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if min_is_active {
        conditions.push("r.is_active = ?".into());
        params_vec.push(Box::new(1i32));
    }

    if let Some(kind) = resource_kind {
        conditions.push("r.resource_kind = ?".into());
        params_vec.push(Box::new(kind.to_string()));
    }

    if let Some(lang) = language {
        conditions.push("r.languages_json LIKE ?".into());
        params_vec.push(Box::new(format!("%\"{lang}\"%")));
    }

    // tag 过滤通过 JOIN resource_tags 实现
    let use_tag_join = tag_type.is_some() && tag_value.is_some();
    let tag_join = if use_tag_join {
        "INNER JOIN resource_tags rt ON r.resource_id = rt.resource_id"
    } else {
        ""
    };

    if let (Some(tt), Some(tv)) = (tag_type, tag_value) {
        conditions.push("rt.tag_type = ?".into());
        params_vec.push(Box::new(tt.to_string()));
        conditions.push("rt.tag_value = ?".into());
        params_vec.push(Box::new(tv.to_string()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "SELECT DISTINCT r.resource_id, r.source_repo_id, r.resource_kind, r.title,
                r.summary, r.source_url, r.languages_json, r.framework_tags_json,
                r.agent_tags_json, r.curation_level, r.last_scored_at, r.is_active
         FROM resources r {tag_join}
         {where_clause}
         ORDER BY r.curation_level = 'USER_CURATED' DESC, r.resource_id
         LIMIT ? OFFSET ?"
    );

    params_vec.push(Box::new(limit));
    params_vec.push(Box::new(offset));

    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| row_to_resource(row))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 插入资源标签（批量）
pub fn insert_tags(conn: &Connection, resource_id: &str, tags: &[ResourceTag]) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO resource_tags (resource_id, tag_type, tag_value) VALUES (?, ?, ?)",
    )?;
    for tag in tags {
        stmt.execute(params![resource_id, tag.tag_type, tag.tag_value])?;
    }
    Ok(())
}

/// 获取资源标签
pub fn get_tags(conn: &Connection, resource_id: &str) -> Result<Vec<ResourceTag>> {
    let mut stmt = conn.prepare(
        "SELECT resource_id, tag_type, tag_value FROM resource_tags WHERE resource_id = ?",
    )?;
    let rows = stmt.query_map(params![resource_id], |row| {
        Ok(ResourceTag {
            resource_id: row.get(0)?,
            tag_type: row.get(1)?,
            tag_value: row.get(2)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// 删除资源标签
pub fn delete_tags(conn: &Connection, resource_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM resource_tags WHERE resource_id = ?",
        params![resource_id],
    )?;
    Ok(())
}

/// 设置资源为非活跃
pub fn deactivate_resource(conn: &Connection, resource_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE resources SET is_active = 0 WHERE resource_id = ?",
        params![resource_id],
    )?;
    Ok(())
}

/// 更新评分时间
pub fn update_last_scored_at(conn: &Connection, resource_id: &str, timestamp: &str) -> Result<()> {
    conn.execute(
        "UPDATE resources SET last_scored_at = ? WHERE resource_id = ?",
        params![timestamp, resource_id],
    )?;
    Ok(())
}

/// 更新精选级别
pub fn update_curation_level(
    conn: &Connection,
    resource_id: &str,
    level: &CurationLevel,
) -> Result<()> {
    conn.execute(
        "UPDATE resources SET curation_level = ? WHERE resource_id = ?",
        params![level.to_string(), resource_id],
    )?;
    Ok(())
}

// ── 内部映射 ────────────────────────────────────────────────

fn row_to_resource(row: &rusqlite::Row) -> rusqlite::Result<Resource> {
    let resource_kind_str: String = row.get(2)?;
    let curation_level_str: String = row.get(9)?;
    let languages_json: String = row.get(6)?;
    let framework_tags_json: String = row.get(7)?;
    let agent_tags_json: String = row.get(8)?;
    let is_active: i32 = row.get(11)?;

    let resource_kind = ResourceKind::from_str(&resource_kind_str)
        .unwrap_or(ResourceKind::Other(resource_kind_str.clone()));
    let curation_level =
        CurationLevel::from_str(&curation_level_str).unwrap_or(CurationLevel::SystemDiscovered);
    let languages: Vec<String> = serde_json::from_str(&languages_json).unwrap_or_default();
    let framework_tags: Vec<String> =
        serde_json::from_str(&framework_tags_json).unwrap_or_default();
    let agent_tags: Vec<String> = serde_json::from_str(&agent_tags_json).unwrap_or_default();

    Ok(Resource {
        resource_id: row.get(0)?,
        source_repo_id: row.get(1)?,
        resource_kind,
        title: row.get(3)?,
        summary: row.get(4)?,
        source_url: row.get(5)?,
        languages,
        framework_tags,
        agent_tags,
        curation_level,
        last_scored_at: row.get(10)?,
        is_active: is_active != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_db;

    fn setup_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        init_db(&mut conn).unwrap();
        conn
    }

    fn sample_resource(id: &str, title: &str) -> Resource {
        Resource {
            resource_id: id.into(),
            source_repo_id: None,
            resource_kind: ResourceKind::McpServer,
            title: title.into(),
            summary: Some("A test MCP server".into()),
            source_url: "https://github.com/acme/test-mcp".into(),
            languages: vec!["Rust".into()],
            framework_tags: vec!["Axum".into()],
            agent_tags: vec!["MCP".into()],
            curation_level: CurationLevel::SystemDiscovered,
            last_scored_at: None,
            is_active: true,
        }
    }

    #[test]
    fn insert_and_get_round_trip() {
        let conn = setup_db();
        let resource = sample_resource("res_01", "acme/test-mcp");
        insert_resource(&conn, &resource).unwrap();

        let got = get_resource(&conn, "res_01").unwrap().unwrap();
        assert_eq!(got.resource_id, "res_01");
        assert_eq!(got.title, "acme/test-mcp");
        assert_eq!(got.resource_kind, ResourceKind::McpServer);
        assert_eq!(got.source_repo_id, None);
        assert_eq!(got.languages, vec!["Rust"]);
        assert_eq!(got.framework_tags, vec!["Axum"]);
        assert_eq!(got.agent_tags, vec!["MCP"]);
        assert!(got.is_active);
    }

    #[test]
    fn get_not_found() {
        let conn = setup_db();
        let got = get_resource(&conn, "nonexistent").unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn upsert_updates_existing() {
        let conn = setup_db();
        let resource = sample_resource("res_01", "acme/test-mcp");
        insert_resource(&conn, &resource).unwrap();

        let mut updated = resource.clone();
        updated.title = "acme/test-mcp-v2".into();
        updated.curation_level = CurationLevel::UserCurated;
        upsert_resource(&conn, &updated).unwrap();

        let got = get_resource(&conn, "res_01").unwrap().unwrap();
        assert_eq!(got.title, "acme/test-mcp-v2");
        assert_eq!(got.curation_level, CurationLevel::UserCurated);
    }

    #[test]
    fn list_resources_pagination() {
        let conn = setup_db();
        for i in 0..5 {
            insert_resource(
                &conn,
                &sample_resource(&format!("res_{i:02}"), &format!("Resource {i}")),
            )
            .unwrap();
        }

        let page1 = list_resources(&conn, 2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = list_resources(&conn, 2, 2).unwrap();
        assert_eq!(page2.len(), 2);

        let page3 = list_resources(&conn, 2, 4).unwrap();
        assert_eq!(page3.len(), 1);
    }

    #[test]
    fn search_resources_by_kind() {
        let conn = setup_db();
        let mut r1 = sample_resource("res_01", "MCP Server");
        r1.resource_kind = ResourceKind::McpServer;
        insert_resource(&conn, &r1).unwrap();

        let mut r2 = sample_resource("res_02", "Skill");
        r2.resource_kind = ResourceKind::Skill;
        insert_resource(&conn, &r2).unwrap();

        let results = search_resources(
            &conn,
            None,
            None,
            Some(ResourceKind::McpServer),
            None,
            true,
            50,
            0,
        )
        .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].resource_id, "res_01");
    }

    #[test]
    fn search_resources_by_language() {
        let conn = setup_db();
        let mut r1 = sample_resource("res_01", "Rust MCP");
        r1.languages = vec!["Rust".into()];
        insert_resource(&conn, &r1).unwrap();

        let mut r2 = sample_resource("res_02", "Python MCP");
        r2.languages = vec!["Python".into()];
        insert_resource(&conn, &r2).unwrap();

        let results = search_resources(&conn, None, None, None, Some("Rust"), true, 50, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].resource_id, "res_01");
    }

    #[test]
    fn insert_and_get_tags() {
        let conn = setup_db();
        insert_resource(&conn, &sample_resource("res_01", "Test")).unwrap();

        let tags = vec![
            ResourceTag {
                resource_id: "res_01".into(),
                tag_type: "language".into(),
                tag_value: "Rust".into(),
            },
            ResourceTag {
                resource_id: "res_01".into(),
                tag_type: "framework".into(),
                tag_value: "Axum".into(),
            },
        ];
        insert_tags(&conn, "res_01", &tags).unwrap();

        let got = get_tags(&conn, "res_01").unwrap();
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn test_delete_tags() {
        let conn = setup_db();
        insert_resource(&conn, &sample_resource("res_01", "Test")).unwrap();

        let tags = vec![ResourceTag {
            resource_id: "res_01".into(),
            tag_type: "language".into(),
            tag_value: "Rust".into(),
        }];
        insert_tags(&conn, "res_01", &tags).unwrap();
        assert_eq!(get_tags(&conn, "res_01").unwrap().len(), 1);

        delete_tags(&conn, "res_01").unwrap();
        assert_eq!(get_tags(&conn, "res_01").unwrap().len(), 0);
    }

    #[test]
    fn test_deactivate_resource() {
        let conn = setup_db();
        insert_resource(&conn, &sample_resource("res_01", "Test")).unwrap();

        let got = get_resource(&conn, "res_01").unwrap().unwrap();
        assert!(got.is_active);

        deactivate_resource(&conn, "res_01").unwrap();

        let got = get_resource(&conn, "res_01").unwrap().unwrap();
        assert!(!got.is_active);

        // 不应出现在 list_resources 中
        let active = list_resources(&conn, 50, 0).unwrap();
        assert!(active.is_empty());
    }

    #[test]
    fn test_update_last_scored_at() {
        let conn = setup_db();
        insert_resource(&conn, &sample_resource("res_01", "Test")).unwrap();

        update_last_scored_at(&conn, "res_01", "2026-03-24T00:00:00Z").unwrap();

        let got = get_resource(&conn, "res_01").unwrap().unwrap();
        assert_eq!(got.last_scored_at, Some("2026-03-24T00:00:00Z".into()));
    }

    #[test]
    fn test_update_curation_level() {
        let conn = setup_db();
        insert_resource(&conn, &sample_resource("res_01", "Test")).unwrap();

        update_curation_level(&conn, "res_01", &CurationLevel::UserCurated).unwrap();

        let got = get_resource(&conn, "res_01").unwrap().unwrap();
        assert_eq!(got.curation_level, CurationLevel::UserCurated);
    }

    #[test]
    fn search_resources_by_tag_join() {
        let conn = setup_db();
        insert_resource(&conn, &sample_resource("res_01", "Rust MCP")).unwrap();
        insert_resource(&conn, &sample_resource("res_02", "Python MCP")).unwrap();

        insert_tags(
            &conn,
            "res_01",
            &[ResourceTag {
                resource_id: "res_01".into(),
                tag_type: "agent".into(),
                tag_value: "MCP".into(),
            }],
        )
        .unwrap();

        insert_tags(
            &conn,
            "res_02",
            &[ResourceTag {
                resource_id: "res_02".into(),
                tag_type: "agent".into(),
                tag_value: "coding-agent".into(),
            }],
        )
        .unwrap();

        let results =
            search_resources(&conn, Some("agent"), Some("MCP"), None, None, true, 50, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].resource_id, "res_01");
    }
}
