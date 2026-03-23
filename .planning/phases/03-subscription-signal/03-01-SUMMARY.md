---
phase: 03-subscription-signal
plan: 01
subsystem: domain
tags: [subscription, signal, rust, serde, state-machine]

# Dependency graph
requires:
  - phase: 02-topk
    provides: domain/repository/ranking 基础领域模型与模块组织模式
provides:
  - Subscription 领域对象（状态、跟踪模式、游标、默认配置）
  - Signal 领域对象（类型、优先级、状态、去重键构造）
  - SubscriptionState / SignalState 显式状态机约束
affects: [application, persistence_sqlite, runtime_tauri, desktop-ui]

# Tech tracking
tech-stack:
  added: []
  patterns: [domain-struct+serde, explicit-state-machine, deterministic-signal-key]

key-files:
  created: [crates/domain/src/subscription.rs, crates/domain/src/signal.rs]
  modified: [crates/domain/src/lib.rs]

key-decisions:
  - "Subscription::new(repo_id) 统一默认值入口，避免上层重复拼装字段"
  - "Signal 构造函数内置 signal_key 规则，保证去重键在领域层唯一来源"
  - "在 domain 层实现 Subscription/Signal 状态迁移校验，防止非法状态写入"

patterns-established:
  - "Pattern 1: enum 使用 serde rename 与数据库/契约枚举字符串对齐"
  - "Pattern 2: 构造函数集中生成 ULID-like id 与时间字段，减少调用方错误"

requirements-completed: [SUB-01, SUB-02, SUB-03, SUB-04, SUB-07, SUB-08]

# Metrics
duration: 5 min
completed: 2026-03-23
---

# Phase 3 Plan 01: 领域模型层 — Subscription + Signal Summary

**新增 Subscription/Signal 统一领域模型，并把 signal_key 生成与状态机约束下沉到 domain 层。**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-23T13:47:00.550Z
- **Completed:** 2026-03-23T13:52:00.509Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- 完成 `SubscriptionState`、`TrackingMode`、`Subscription` 以及 `Subscription::new(repo_id)` 默认构造逻辑。
- 完成 `SignalType`、`SignalPriority`、`SignalState`、`SourceKind`、`Signal` 及 release/tag/digest 三类 signal_key 构造函数。
- 在领域层加入 Subscription/Signal 状态机迁移校验，覆盖 `ACTIVE/PAUSED/ARCHIVED` 与 `NEW/SEEN/ACKED/ARCHIVED` 规范。

## Task Commits

Each task was committed atomically:

1. **Task 1: 创建 Subscription 领域对象** - `3c2f448` (feat)
2. **Task 2: 创建 Signal 领域对象** - `e74717a` (feat)

## Files Created/Modified

- `crates/domain/src/subscription.rs` - Subscription 领域对象、默认值、状态迁移校验、相关单测
- `crates/domain/src/signal.rs` - Signal 领域对象、signal_key 构造函数、状态迁移校验、相关单测
- `crates/domain/src/lib.rs` - 导出 `subscription` 与 `signal` 模块

## Decisions Made

- 使用领域构造函数承载默认字段和值（event types、digest window、priority），避免应用层重复硬编码。
- 使用领域层方法封装状态机转换规则，防止无效状态变更穿透到持久化层。
- 采用 `repo_id:TYPE:unique_id` 规则直接生成 signal_key，确保幂等去重键与规范一致。

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] 补齐 Subscription/Signal 状态机约束实现**
- **Found during:** Task 2 (创建 Signal 领域对象)
- **Issue:** 计划动作已建模枚举，但未显式提供状态迁移校验，无法保证非法转换被拦截。
- **Fix:** 在 `Subscription` 与 `Signal` 中新增 `transition_state` / `can_transition_to` 规则及单元测试，严格限制允许路径。
- **Files modified:** `crates/domain/src/subscription.rs`, `crates/domain/src/signal.rs`
- **Verification:** `cargo check` + `cargo test` 全通过
- **Committed in:** `e74717a` (part of Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 2: 1)
**Impact on plan:** 该修正属于领域正确性必需项，无额外范围膨胀。

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- 已具备 Phase 03-02 持久化层实现所需领域对象与约束。
- 后续可直接在 persistence/application 层复用 `Subscription::new` 与 signal 构造函数，减少重复业务拼装。

---

*Phase: 03-subscription-signal*
*Completed: 2026-03-23*

## Self-Check: PASSED

- 文件存在性检查通过：`crates/domain/src/subscription.rs`, `crates/domain/src/signal.rs`, `.planning/phases/03-subscription-signal/03-01-SUMMARY.md`
- 提交存在性检查通过：`3c2f448`, `e74717a`
