---
phase: 02-topk
plan: 01
subsystem: domain-layer
tags:
  - domain-model
  - ranking
  - dto
  - momentum
dependency_graph:
  requires: []
  provides:
    - domain::repository::Repository
    - domain::repository::RepoSnapshot
    - domain::ranking::RankingMode
    - domain::ranking::RankingView
    - domain::ranking::RankingFilters
    - domain::ranking::RankingSnapshot
    - domain::ranking::MomentumScore
    - shared_contracts::ranking_dto::RankingViewSpecDto
    - shared_contracts::ranking_dto::RankingItemDto
    - shared_contracts::ranking_dto::FiltersDto
  affects:
    - persistence_sqlite (future: table repositories, repo_snapshots)
    - persistence_sqlite (future: table ranking_views, ranking_snapshots)
    - application (future: TopK use cases)
    - runtime_tauri (future: topk commands)
tech_stack:
  added: []
  patterns:
    - serde Serialize/Deserialize domain objects
    - From trait for domain → DTO conversion
    - Display enum for string round-trip
key_files:
  created:
    - crates/domain/src/repository.rs
    - crates/domain/src/ranking.rs
    - crates/shared_contracts/src/ranking_dto.rs
  modified:
    - crates/domain/src/lib.rs
    - crates/shared_contracts/src/lib.rs
decisions:
  - RankingMode uses Rust enum naming (StarsDesc) with Display returning UPPER_SNAKE
  - RankingFilters::new() defaults exclude_archived=true, exclude_forks=true
  - MomentumScore::compute() uses max_delta=1000 as default (plan spec)
  - From conversions only domain→DTO direction (not reverse)
metrics:
  duration: ~10min
  tasks_completed: 3/3
  files_created: 3
  files_modified: 2
  tests_added: 26
  commits: 3
---

# Phase 02 Plan 01: 领域模型层 Summary

**One-liner:** 领域层定义 Repository/RepoSnapshot/RankingView/RankingSnapshot 纯领域对象与 Momentum 评分公式；shared_contracts 新增 RankingViewSpecDto/RankingItemDto/FiltersDto 契约对象与 From 双向转换。

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Repository + RepoSnapshot 领域对象 | `6b752e7` | repository.rs, lib.rs |
| 2 | Ranking 领域对象 + Momentum 评分 | `ba1beae` | ranking.rs, lib.rs |
| 3 | RankingViewSpec / RankingItem DTO | `fa1f5d4` | ranking_dto.rs, lib.rs |

## Task Details

### Task 1: Repository / RepoSnapshot 领域对象

- `Repository` struct: 16 fields 对应 repositories 表（repo_id, full_name, owner, name, html_url, description, default_branch, primary_language, topics: Vec<String>, archived, disabled, stargazers_count, forks_count, updated_at, pushed_at, last_synced_at）
- `RepoSnapshot` struct: 8 fields 对应 repo_snapshots 表（snapshot_id, repo_id, snapshot_at, stargazers_count, forks_count, updated_at, pushed_at, release_count）
- 模式: pub struct + pub fields + Debug, Clone, Serialize, Deserialize
- 3 tests: clone, JSON round-trip, optional fields

### Task 2: Ranking 领域对象 + Momentum 评分

- `RankingMode` enum: StarsDesc, UpdatedDesc, Momentum24h, Momentum7d
  - Display impl 返回字符串（如 "STARS_DESC"）
  - from_str() 反向解析
- `RankingFilters` struct: 6 fields，new() 默认 exclude_archived=true, exclude_forks=true
- `RankingView` struct: 11 fields 对应 ranking_views 表
- `RankingSnapshot` struct: items: Vec<RankingSnapshotItem>, stats: SnapshotStats
- `MomentumScore` struct: total, star_delta_norm, fork_delta_norm, updated_recency_norm
- `compute_momentum()` 公式: 0.5×star + 0.2×fork + 0.3×recency, max_delta 参数化
- `MomentumScore::compute()` 便捷方法使用 max_delta=1000.0
- 10 tests 覆盖: zero growth, max score, negative clamping, recency decay, custom max_delta, serialization

### Task 3: RankingViewSpecDto / RankingItemDto

- `RankingViewSpecDto` struct: 前端展示用，ranking_mode 为 String
- `FiltersDto` struct: 对应 RankingFilters 的 DTO
- `RankingItemDto` struct: 榜单项，含 score_breakdown + rank_change
- `ScoreBreakdownDto` struct: star_delta, fork_delta, updated_recency
- `CreateRankingViewRequest` struct: API 输入
- `From<RankingFilters> for FiltersDto`
- `From<RankingView> for RankingViewSpecDto`
- 5 tests: From 转换、JSON round-trip

## Verification Results

```
cargo check -p domain -p shared_contracts  → ✅ 编译通过
cargo test  -p domain -p shared_contracts  → ✅ 26 passed (4 suites, 0.00s)
```

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all types have proper derive, fields, and implementations.

## Self-Check: PASSED

- [x] crates/domain/src/repository.rs exists
- [x] crates/domain/src/ranking.rs exists
- [x] crates/shared_contracts/src/ranking_dto.rs exists
- [x] Commit 6b752e7 (Task 1) exists
- [x] Commit ba1beae (Task 2) exists
- [x] Commit fa1f5d4 (Task 3) exists
