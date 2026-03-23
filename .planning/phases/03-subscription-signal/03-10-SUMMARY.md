---
phase: 03-subscription-signal
plan: 10
subsystem: ui
tags: [subscriptions, svelte, store, tauri-ipc]

# Dependency graph
requires:
  - phase: 03-subscription-signal
    provides: subscriptions 页面与 SubscriptionSearch 基础组件
provides:
  - /subscriptions 页面搜索结果可直接创建订阅
  - 搜索组件 onSubscribe 回调与按钮状态反馈
  - addSubscription 的 repo_id 去重与失败错误透传
affects: [home-signals, subscription-sync]

# Tech tracking
tech-stack:
  added: []
  patterns: ["页面回调驱动 store action", "store 内统一错误状态", "repo_id 级别去重更新"]

key-files:
  created: []
  modified: [apps/desktop-ui/src/lib/components/SubscriptionSearch.svelte, apps/desktop-ui/src/routes/subscriptions/+page.svelte, apps/desktop-ui/src/lib/stores/subscriptions.ts, apps/desktop-ui/src/lib/types.ts, crates/shared_contracts/src/repo_dto.rs, crates/github_adapter/src/auth.rs]

key-decisions:
  - "直接在 SubscriptionSearch 触发 onSubscribe(repo_id)，避免再引入中间组件层"
  - "addSubscription 在 store 内做 repo_id 去重，页面只负责触发动作"

patterns-established:
  - "搜索组件业务动作通过必需回调上抛，由页面决定具体副作用"
  - "创建动作失败统一写入 subscriptionsError，页面复用现有错误块展示"

requirements-completed: [SUB-01]

# Metrics
duration: 9 min
completed: 2026-03-23
---

# Phase 03 Plan 10: Subscription 搜索创建断链修复 Summary

**订阅页已实现“搜索即订阅”：搜索结果卡直接触发 addSubscription，并在失败时显示明确错误。**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-23T15:27:00Z
- **Completed:** 2026-03-23T15:36:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- SubscriptionSearch 新增 `onSubscribe(repoId)` 必需回调与“订阅此仓库”按钮（含 loading/disabled 状态）。
- `/subscriptions` 页面完成 `onSubscribe -> addSubscription(repoId)` 接线。
- `addSubscription` 增加 repo_id 去重替换逻辑，避免重复卡片，并在失败时写入 `subscriptionsError`。

## Task Commits

1. **Task 1: 为 SubscriptionSearch 增加可订阅回调** - `dc53aba` (feat)
2. **Task 2: 在订阅页接线 addSubscription 并处理去重/错误** - `da467c7` (feat)

## Files Created/Modified
- `apps/desktop-ui/src/lib/components/SubscriptionSearch.svelte` - 新增 `onSubscribe`、订阅按钮、错误展示与交互状态。
- `apps/desktop-ui/src/routes/subscriptions/+page.svelte` - 注入 `addSubscription` 并传入 `onSubscribe`。
- `apps/desktop-ui/src/lib/stores/subscriptions.ts` - `addSubscription` 增加去重更新和错误透传。
- `apps/desktop-ui/src/lib/types.ts` - `RepoBasicInfo` 增加 `repo_id`。
- `crates/shared_contracts/src/repo_dto.rs` - `RepoBasicInfo` DTO 增加 `repo_id`。
- `crates/github_adapter/src/auth.rs` - `fetch_repo_info` 返回 `repo_id`。

## Decisions Made
- 为满足 `onSubscribe(repo_id)` contract，在现有 `fetchRepoInfo` DTO 上补充 `repo_id`，避免新增额外查询 API。
- 保持页面层最小职责：页面仅触发 `addSubscription`，去重/错误由 store 统一处理。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] 补齐 repo_id 数据链路以支撑 onSubscribe(repo_id)**
- **Found during:** Task 1
- **Issue:** 现有 `fetchRepoInfo` 返回结构缺少 `repo_id`，无法按计划回调 `onSubscribe(repo_id)`。
- **Fix:** 扩展 shared_contracts + github_adapter + 前端类型，补齐 `repo_id` 字段。
- **Files modified:** `crates/shared_contracts/src/repo_dto.rs`, `crates/github_adapter/src/auth.rs`, `apps/desktop-ui/src/lib/types.ts`
- **Verification:** `rtk pnpm exec svelte-check --tsconfig ./tsconfig.json`（在 `apps/desktop-ui` 目录执行）
- **Committed in:** `dc53aba`

**2. [Rule 3 - Blocking] 修正计划命令在 monorepo 根目录不可执行**
- **Found during:** Task 1 验证
- **Issue:** 计划命令 `rtk pnpm exec svelte-check --tsconfig ./apps/desktop-ui/tsconfig.json` 在仓库根执行时报 `No package found in this workspace`。
- **Fix:** 按现有项目结构切换到 `apps/desktop-ui` 执行同一验证工具。
- **Files modified:** None（仅执行路径调整）
- **Verification:** `rtk pnpm exec svelte-check --tsconfig ./tsconfig.json` 通过（0 error, 7 existing warnings）
- **Committed in:** 无代码变更

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 blocking)
**Impact on plan:** 均为闭合 SUB-01 所必需，未引入范围外功能。

## Issues Encountered
- None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `/subscriptions` 已可独立完成搜索->订阅闭环。
- 可继续在同步链路中消费 `notify_high_immediately`、`digest_window` 等订阅配置。

## Self-Check: PASSED
- Summary file exists: `.planning/phases/03-subscription-signal/03-10-SUMMARY.md`
- Commits verified: `dc53aba`, `da467c7`
