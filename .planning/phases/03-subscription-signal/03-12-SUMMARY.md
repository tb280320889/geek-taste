---
phase: 03-subscription-signal
plan: 12
subsystem: database
tags: [home-signals, since, ranking, affinity, svelte-store]

# Dependency graph
requires:
  - phase: 03-subscription-signal
    provides: signal repository + runtime signal commands + home signals store
provides:
  - Home 信号查询支持 since(last_visit) 参数
  - Home 排序扩展为 priority/time/source_kind/affinity 四维
  - 前端 localStorage 持久化 last_visit 并参与查询
affects: [home-page, signal-feed, phase-03-verification]

# Tech tracking
tech-stack:
  added: []
  patterns: ["runtime 读取 settings 透传 language_interests", "store 内游标持久化并按成功分支更新"]

key-files:
  created: []
  modified: [crates/persistence_sqlite/src/signal_repository.rs, crates/application/src/signal.rs, crates/runtime_tauri/src/commands/signal.rs, apps/desktop-ui/src/lib/ipc/tauri.ts, apps/desktop-ui/src/lib/stores/signals.ts]

key-decisions:
  - "affinity 采用 repository.primary_language 与 language_interests 的大小写不敏感匹配"
  - "last_visit 仅在 loadHomeSignals 成功后写回，失败不覆盖旧游标"

patterns-established:
  - "list_home_signals(since, language_interests) 作为 Home 查询统一 contract"
  - "IPC 函数 listHomeSignals(since?) 与 runtime 命令签名保持一一映射"

requirements-completed: [HOME-01, HOME-02]

# Metrics
duration: 5 min
completed: 2026-03-23
---

# Phase 03 Plan 12: Home 增量游标与多因子排序 Summary

**Home 信号流已实现“自上次访问以来”的增量语义，并按优先级/时间/来源类型/用户语言亲和度进行排序。**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-23T15:43:00Z
- **Completed:** 2026-03-23T15:48:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- `signal_repository::list_home_signals` 扩展签名：支持 `since` + `language_interests`，并加入 source_kind/affinity 排序表达式。
- `application::signal` 与 `runtime_tauri::commands::signal` 完成 `since` 和语言兴趣透传。
- `signals.ts` 新增 `home_signals_last_visit` 持久化，成功加载后更新游标，失败不覆盖。

## Task Commits

1. **Task 1: 扩展 Home 查询 contract（since + 排序因子）** - `a11ccea` (feat)
2. **Task 2: 前端落地 last_visit 游标并改造加载流程** - `39a9f31` (feat)

## Files Created/Modified
- `crates/persistence_sqlite/src/signal_repository.rs` - 增加 since 条件、source_kind 权重、affinity 权重排序和测试。
- `crates/application/src/signal.rs` - `list_home_signals` 支持 `since` + `language_interests`。
- `crates/runtime_tauri/src/commands/signal.rs` - 命令签名新增 `since`，从 settings 读取语言兴趣透传。
- `apps/desktop-ui/src/lib/ipc/tauri.ts` - `listHomeSignals(since?)` IPC 封装更新。
- `apps/desktop-ui/src/lib/stores/signals.ts` - 新增 `home_signals_last_visit` 读写与调用参数透传。

## Decisions Made
- affinity 在 SQL 中按 `lower(primary_language) IN (...)` 计算，避免前端/应用层二次排序。
- 排序主序保持 priority + occurred_at，source_kind/affinity 作为补充分层，保证新鲜度不回归。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] 修复 SQL `ORDER BY 0` 在 SQLite 中非法**
- **Found during:** Task 1 测试
- **Issue:** 当 `language_interests` 为空时，affinity 排序表达式退化为 `0 DESC`，SQLite 解释为列序号导致 SQL 错误。
- **Fix:** 空兴趣时改为常量 CASE 表达式（`CASE WHEN 1=1 THEN 0 ELSE 0 END`）保持合法排序项。
- **Files modified:** `crates/persistence_sqlite/src/signal_repository.rs`
- **Verification:** `rtk cargo test -p persistence_sqlite signal_repository -- --nocapture`
- **Committed in:** `a11ccea`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** 修复后保持计划功能边界，未扩大范围。

## Issues Encountered
- None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Home 页面信号数据 contract 已与后端保持一致，可继续做人测验证视觉与交互。

## Self-Check: PASSED
- Summary file exists: `.planning/phases/03-subscription-signal/03-12-SUMMARY.md`
- Commits verified: `a11ccea`, `39a9f31`
