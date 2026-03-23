---
phase: 03-subscription-signal
plan: 11
subsystem: api
tags: [subscription, signal, digest, notification, tauri]

# Dependency graph
requires:
  - phase: 03-subscription-signal
    provides: subscription/signal 仓库层与 releases/tags adapter
provides:
  - sync_subscriptions 的 U1-U4 冲突消解（RELEASE > TAG > DIGEST）
  - digest_window(12h/24h) 生成 DEFAULT_BRANCH_ACTIVITY_DIGEST
  - HIGH 信号即时通知链路 + quiet-hours 抑制
affects: [home-signals, desktop-notification, phase-03-gap-close]

# Tech tracking
tech-stack:
  added: []
  patterns: ["同步候选事件先聚合后决策", "notification_adapter 仅做发送封装", "quiet-hours 业务判断在 application"]

key-files:
  created: []
  modified: [crates/application/src/subscription.rs, crates/notification_adapter/src/lib.rs, crates/notification_adapter/Cargo.toml, crates/runtime_tauri/src/commands/subscription.rs]

key-decisions:
  - "U1-U4 冲突消解在 application 层统一执行，持久化层保持幂等插入职责"
  - "runtime_tauri 在同步后发送通知，application 只返回待通知 HIGH 信号"

patterns-established:
  - "sync_subscriptions 返回 (synced_count, notifications) 以隔离业务决策与基础设施发送"
  - "quiet-hours 判定函数独立，可复用于后续其他通知策略"

requirements-completed: [SUB-04, SUB-05, SUB-06]

# Metrics
duration: 11 min
completed: 2026-03-23
---

# Phase 03 Plan 11: 同步规则 + digest + 通知链路 Summary

**订阅同步主循环已升级为可冲突消解、可 digest 聚合、可按 quiet-hours 约束发送 HIGH 通知的完整后端闭环。**

## Performance

- **Duration:** 11 min
- **Started:** 2026-03-23T15:36:00Z
- **Completed:** 2026-03-23T15:43:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- `sync_subscriptions` 实现 U1-U4：同批同步中 RELEASE 覆盖 TAG/DIGEST，TAG 覆盖 DIGEST。
- 按订阅 `digest_window` 计算 12h/24h bucket，且仅在无 release/tag 时生成 digest 信号。
- 实现 `notification_adapter` 发送函数；`runtime_tauri::sync_subscriptions` 在 HIGH + notify_high_immediately + 非 quiet-hours 时发通知。

## Task Commits

1. **Task 1: 在 sync_subscriptions 实现 U1-U4 与 digest 聚合** - `fcc3c34` (feat)
2. **Task 2: 建立 HIGH 通知链路并落实 quiet-hours** - `758cc17` (feat)

## Files Created/Modified
- `crates/application/src/subscription.rs` - 新增候选事件聚合、U1-U4 决策、digest 生成、quiet-hours 判定与通知输出 DTO。
- `crates/notification_adapter/src/lib.rs` - 新增 `send_high_signal_notification`，封装 tauri-plugin-notification。
- `crates/notification_adapter/Cargo.toml` - 补充 `tauri` 依赖以支持 `AppHandle`。
- `crates/runtime_tauri/src/commands/subscription.rs` - 同步后拉取 settings、触发通知发送。

## Decisions Made
- 通知文案采用统一模板：`{repo_name}: {signal_type_text} — {title}`，由 notification_adapter 生成 body。
- quiet-hours 作为 application 层纯函数判定，runtime 层只消费判定结果，不复制业务逻辑。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] 修复 notification_adapter 缺少 tauri 依赖导致编译失败**
- **Found during:** Task 2 验证
- **Issue:** `notification_adapter` 使用 `tauri::AppHandle` 时报 unresolved import `tauri`。
- **Fix:** 在 `crates/notification_adapter/Cargo.toml` 增加 `tauri = { workspace = true }`。
- **Files modified:** `crates/notification_adapter/Cargo.toml`
- **Verification:** `rtk cargo check -p runtime_tauri && rtk cargo check -p notification_adapter`
- **Committed in:** `758cc17`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** 仅补齐编译前置条件，无额外功能扩张。

## Issues Encountered
- None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- 已具备 Home 增量查询和多因子排序改造的完整上游信号数据输入。

## Self-Check: PASSED
- Summary file exists: `.planning/phases/03-subscription-signal/03-11-SUMMARY.md`
- Commits verified: `fcc3c34`, `758cc17`
