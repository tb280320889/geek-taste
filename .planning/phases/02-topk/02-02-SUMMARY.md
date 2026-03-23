# Phase 02 Plan 02: SQLite 持久化层 Summary

## One-liner

SQLite 持久化层：V001 migration 创建 4 表 + 索引，Repository/RepoSnapshot/RankingView/RankingSnapshot 全部 CRUD 可用，23 个测试通过。

## What Was Built

- **V001 Migration** (`migrations.rs`)：使用 `rusqlite_migration` 创建 `repositories`、`repo_snapshots`、`ranking_views`、`ranking_snapshots` 4 张表 + 3 个索引
- **init_db** (`lib.rs`)：执行 migrations + 配置 WAL 模式 + busy_timeout=5000ms
- **Repository CRUD** (`repo_repository.rs`)：upsert、get、search（含 language/stars/topic 过滤 + 分页排序）、snapshot 插入与查询
- **Ranking CRUD** (`ranking_repository.rs`)：create/update/delete/get/list RankingView，save/get RankingSnapshot，toggle_pin

## Files Modified

| File | Change |
|------|--------|
| `crates/persistence_sqlite/Cargo.toml` | 添加 rusqlite_migration, serde, serde_json, chrono, anyhow, tokio 依赖 |
| `crates/persistence_sqlite/src/lib.rs` | init_db() + 模块声明 |
| `crates/persistence_sqlite/src/migrations.rs` | V001 migration (4 表 + 3 索引) — 新增 |
| `crates/persistence_sqlite/src/repo_repository.rs` | Repository/RepoSnapshot CRUD — 新增 |
| `crates/persistence_sqlite/src/ranking_repository.rs` | RankingView/RankingSnapshot CRUD — 新增 |

## Key Decisions

1. **SearchFilters::default() limit=30** — 避免 LIMIT 0 返回空结果，符合 TopK 前端默认页大小
2. **delete_ranking_view 级联删除** — 删除视图同时删除关联的 ranking_snapshots
3. **save_ranking_snapshot 自动更新 last_snapshot_at** — 保证视图层无需额外更新
4. **WAL 测试用文件数据库** — 内存数据库不支持 WAL 模式，测试使用临时文件验证

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] init_db 参数类型修正**
- **Found during:** Task 1
- **Issue:** `init_db` 接受 `&rusqlite::Connection`，但 `to_latest` 需要 `&mut Connection`
- **Fix:** 改为 `&mut rusqlite::Connection`
- **Files modified:** `crates/persistence_sqlite/src/lib.rs`

**2. [Rule 1 - Bug] WAL 测试在内存数据库中失败**
- **Found during:** Task 1
- **Issue:** WAL 模式在内存数据库中不生效（SQLite 限制），测试断言失败
- **Fix:** 拆分为 3 个独立测试：表创建、busy_timeout、WAL（使用文件数据库）
- **Files modified:** `crates/persistence_sqlite/src/lib.rs`

**3. [Rule 1 - Bug] SearchFilters default limit=0 导致空结果**
- **Found during:** Task 2
- **Issue:** `SearchFilters` 使用 `#[derive(Default)]`，`limit: i64` 默认值为 0，导致 `LIMIT 0` 查询返回空
- **Fix:** 手动实现 `Default`，limit=30, sort_by="stars"
- **Files modified:** `crates/persistence_sqlite/src/repo_repository.rs`

**4. [Rule 1 - Bug] 测试函数名与模块函数名冲突**
- **Found during:** Task 3
- **Issue:** `toggle_pin` 测试函数名与 `toggle_pin` 模块级函数同名，导致编译错误
- **Fix:** 重命名测试为 `test_toggle_pin`
- **Files modified:** `crates/persistence_sqlite/src/ranking_repository.rs`

## Verification

```bash
cargo check -p persistence_sqlite  # ✅ 编译通过
cargo test -p persistence_sqlite   # ✅ 23 passed (5 migrations + 9 repo + 9 ranking)
```

## Test Coverage

| 模块 | 测试数 | 覆盖内容 |
|------|--------|----------|
| migrations | 5 | 表创建、索引、busy_timeout、WAL |
| repo_repository | 9 | upsert、get、not_found、update、search by language/stars/topic、pagination、snapshot |
| ranking_repository | 9 | create、get、update、delete+cascade、list ordering、snapshot、toggle_pin |

## Next Steps

- Phase 2 Plan 03: GitHub Search API 客户端扩展（search_repositories 接口）
- 后续可将 persistence_sqlite 的 CRUD 接口连接到 application 层

---
*Executed: 2026-03-23 | Phase: 02-topk | Plan: 02*
