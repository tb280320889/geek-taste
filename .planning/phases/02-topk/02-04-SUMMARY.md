---
phase: 02-topk
plan: 04
subsystem: application
tags: [tauri, ipc, use-case, ranking, momentum, topk]

requires:
  - phase: 02-topk
    provides: domain ranking models, persistence CRUD, github search client, shared DTOs

provides:
  - "TopK 应用层用例编排 (application::topk)"
  - "Tauri IPC 命令 (list/create/delete/toggle_pin/execute)"
  - "execute_ranking 支持 STARS_DESC / UPDATED_DESC / MOMENTUM_24H / MOMENTUM_7D"
  - "Momentum 无历史快照时降级为 UPDATED_DESC"
  - "create_ranking_view 自动触发暖机快照"
  - "排名变化计算 (rank_change)"
  - "快照自动保存 (execute 后)"

affects: [topk-frontend, subscription-system]

tech-stack:
  added: []
  patterns:
    - "应用层编排: application 调用 persistence + github_adapter + domain"
    - "Tauri command 薄封装: keyring token → application 层 → DTO"
    - "DB 连接每次调用独立打开 (WAL 模式支持并发)"
    - "暖机快照: 创建视图后自动触发首次排名"
    - "Momentum 降级: 无历史快照 → UPDATED_DESC"

key-files:
  created:
    - crates/application/src/topk.rs
    - crates/runtime_tauri/src/commands/topk.rs
  modified:
    - crates/application/src/lib.rs
    - crates/application/Cargo.toml
    - crates/runtime_tauri/src/commands/mod.rs
    - crates/runtime_tauri/Cargo.toml
    - apps/desktop-ui/src-tauri/src/lib.rs

key-decisions:
  - "application 层直接依赖 persistence_sqlite + github_adapter — 中小型项目务实选择，不引入 trait 抽象"
  - "execute_ranking 接收 Connection + token，在内部创建 RateBudget — v1 简化，无需外部注入"
  - "DB 连接每次 IPC 调用独立打开 — WAL 模式支持并发，避免 Mutex 共享状态复杂性"
  - "Momentum 模式用 repo_snapshots 获取上一次 stars/forks 数据，非 ranking_snapshot"
  - "create_snapshot 使用简化统计 (new_count/changed_count) — 精确统计需跨视图比较，v1 不做"
  - "desktop-ui-tauri 编译因缺少 icons/icon.ico 失败 — 预存问题，非本次改动引起"

patterns-established:
  - "应用层编排模式: 从 RankingFilters → SearchQuery 的转换 + 排序 + DTO 转换"
  - "Tauri command 薄封装: get_db_connection() + load_token() 辅助函数模式"
  - "暖机快照: create_view 后 try execute_ranking + create_snapshot，失败仅 warn"

requirements-completed: [TOPK-01, TOPK-02, TOPK-03, TOPK-04, TOPK-05, TOPK-06, TOPK-07, TOPK-08]

# Metrics
duration: 12min
completed: 2026-03-23
---

# Phase 02 Plan 04: TopK 应用层编排 + Tauri IPC 命令 Summary

**TopK 排名引擎的用例编排层和 IPC 命令薄封装 — 前端可通过 invoke() 调用 list/create/delete/toggle_pin/execute 5 个命令**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-23T07:23:35Z
- **Completed:** 2026-03-23T07:35:40Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- application::topk 模块实现 7 个用例函数: execute_ranking, list_views, create_view, delete_view, toggle_pin_view, create_snapshot, get_rank_change
- execute_ranking 支持 4 种排序模式，Momentum 无历史快照时降级为 UPDATED_DESC
- Tauri IPC 命令 5 个: list_ranking_views, create_ranking_view, delete_ranking_view, toggle_pin_ranking_view, execute_ranking
- create_ranking_view 自动触发暖机快照（失败仅 warn）
- execute_ranking 执行后自动保存快照 + 计算排名变化
- 8 个单元测试覆盖 CRUD + 排序 + 快照 + 排名变化

## Task Commits

1. **Task 1: 实现 TopK 应用层用例** - `3e2547f` (feat)
2. **Task 2: 实现 Tauri IPC 命令** - `a8095f1` (feat)

## Files Created/Modified
- `crates/application/src/topk.rs` - TopK 用例编排 (566 行, 含 8 个测试)
- `crates/application/src/lib.rs` - 添加 `pub mod topk;`
- `crates/application/Cargo.toml` - 添加 persistence_sqlite/github_adapter/rusqlite/chrono/anyhow 依赖
- `crates/runtime_tauri/src/commands/topk.rs` - 5 个 Tauri IPC 命令 (117 行)
- `crates/runtime_tauri/src/commands/mod.rs` - 添加 topk 模块导出
- `crates/runtime_tauri/Cargo.toml` - 添加 rusqlite 依赖
- `apps/desktop-ui/src-tauri/src/lib.rs` - 注册 5 个新命令到 invoke_handler

## Decisions Made
- application 层直接依赖 persistence_sqlite + github_adapter — v1 务实选择，避免 trait 抽闻
- DB 连接每次 IPC 调用独立打开 — WAL 模式支持并发
- Momentum 排序使用 repo_snapshots 获取上一次 stars/forks（非 ranking_snapshot）
- 暖机快照失败仅 warn — 不阻塞视图创建

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- desktop-ui-tauri 编译因缺少 `icons/icon.ico` 失败 — 预存问题，不影响 runtime_tauri 和 application 编译
- 测试中 `delete_view` 函数名与模块函数冲突 — 重命名为 `delete_view_works` 解决

## Next Phase Readiness
- TopK 发现引擎完整链路已打通: 前端 IPC → Tauri command → application 层 → persistence + GitHub API
- 下一步 (Plan 05/06): 前端 TopK 页面 UI 实现

---
*Phase: 02-topk*
*Completed: 2026-03-23*
