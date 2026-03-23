---
phase: 03-subscription-signal
verified: 2026-03-23T14:11:36Z
status: gaps_found
score: 1/5 must-haves verified
gaps:
  - truth: "用户能搜索仓库并创建订阅，能编辑/暂停/删除已有订阅"
    status: failed
    reason: "订阅页搜索组件仅调用 fetchRepoInfo 展示仓库信息，没有触发 subscribe/addSubscription。"
    artifacts:
      - path: "apps/desktop-ui/src/lib/components/SubscriptionSearch.svelte"
        issue: "缺少 onSubscribe 回调与 subscribe IPC/store 调用，无法从搜索结果直接创建订阅。"
      - path: "apps/desktop-ui/src/routes/subscriptions/+page.svelte"
        issue: "页面未把搜索结果与 addSubscription 连接。"
    missing:
      - "在 SubscriptionSearch 中提供可点击结果并回调 repo_id"
      - "在 subscriptions 页面接入 addSubscription(repoId) 流程"
  - truth: "系统按 U1-U4 优先级规则去重，HIGH 优先级信号触发桌面通知"
    status: failed
    reason: "当前仅有 release/tag 幂等插入与优先级排序；未实现 U1-U4 去重规则与 HIGH 即时通知发送链路。"
    artifacts:
      - path: "crates/application/src/subscription.rs"
        issue: "sync_subscriptions 仅生成 release/tag 信号，无 U1-U4 规则编排。"
      - path: "crates/notification_adapter/src/lib.rs"
        issue: "通知适配器仍是空文件，未封装发送逻辑。"
    missing:
      - "实现 U1-U4 冲突消解（RELEASE > TAG > DIGEST）"
      - "接入 HIGH 信号通知发送（含设置与安静时段判断）"
  - truth: "系统生成 12h/24h digest 聚合信号，用户能标记信号为已读/已处理"
    status: partial
    reason: "已实现 mark_seen/ack，但未在同步流程中生成 digest 聚合信号。"
    artifacts:
      - path: "crates/domain/src/signal.rs"
        issue: "存在 new_digest 构造函数但未被同步流程调用。"
      - path: "crates/application/src/subscription.rs"
        issue: "sync_subscriptions 未按 digest_window 产生 DEFAULT_BRANCH_ACTIVITY_DIGEST。"
    missing:
      - "在 sync_subscriptions 中基于 12h/24h 窗口聚合并写入 digest 信号"
  - truth: "Home 页面聚合自上次访问以来高优先级摘要，按优先级+时间+来源类型+用户亲和度排序"
    status: failed
    reason: "Home 仅按 priority/time 查询 NEW/SEEN；缺少 since-last-visit、source_type 与 affinity 维度排序。"
    artifacts:
      - path: "crates/persistence_sqlite/src/signal_repository.rs"
        issue: "list_home_signals 仅 CASE priority + occurred_at 排序，未纳入来源类型/亲和度。"
      - path: "apps/desktop-ui/src/lib/stores/signals.ts"
        issue: "未保存/传递 last_visit 游标。"
    missing:
      - "引入 last_visit 基线并只聚合其后信号"
      - "实现 source_kind 与用户亲和度排序因子"
---

# Phase 3: 订阅系统与信号模型 Verification Report

**Phase Goal:** 用户能管理订阅、接收高信噪比的技术信号、在 Home 页面一览全局（M2: repo search/subscribe/sync/signal/digest/notification）
**Verified:** 2026-03-23T14:11:36Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | 用户能搜索仓库并创建订阅，能编辑/暂停/删除已有订阅 | ✗ FAILED | `SubscriptionSearch.svelte` 仅 `fetchRepoInfo`（无 `addSubscription/subscribe`）；`subscriptions/+page.svelte` 无搜索订阅接线。暂停/删除在 `subscriptions.ts` + `SubscriptionCard.svelte` 已实现。 |
| 2 | 系统轮询订阅仓库 Releases/Tags 并幂等生成信号 | ✓ VERIFIED | `application::sync_subscriptions` 调用 `fetch_latest_releases/tags` + `signal_repository::insert_signal(INSERT OR IGNORE)`，见 `crates/application/src/subscription.rs`, `crates/persistence_sqlite/src/signal_repository.rs`。 |
| 3 | 系统按 U1-U4 去重，HIGH 信号触发桌面通知 | ✗ FAILED | 未见 U1-U4 冲突消解实现；`notification_adapter/src/lib.rs` 为空，未见 HIGH 发送链路。 |
| 4 | 系统生成 12h/24h digest 聚合信号，用户可标记已读/已处理 | ✗ FAILED | `mark_seen/ack_signal` 已实现（`application/src/signal.rs`）；但 `sync_subscriptions` 未生成 digest，`Signal::new_digest` 未被调用。 |
| 5 | Home 聚合自上次访问以来高优摘要，按优先级+时间+来源类型+亲和度排序 | ✗ FAILED | `list_home_signals` 仅 `state IN ('NEW','SEEN')` + `priority/time`，无 last_visit、source/affinity 排序。 |

**Score:** 1/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/application/src/subscription.rs` | 订阅同步主循环 | ✓ VERIFIED | 存在并实质实现 polling + 幂等写入；但缺 digest/notification 规则。 |
| `crates/persistence_sqlite/src/signal_repository.rs` | 信号幂等与状态管理 | ✓ VERIFIED | `INSERT OR IGNORE`、`mark_seen/acked`、未读统计齐全。 |
| `apps/desktop-ui/src/lib/components/SubscriptionSearch.svelte` | 搜索并可直接创建订阅 | ⚠️ HOLLOW — wired but data disconnected | 只查询 repo 信息，不连接订阅创建。 |
| `apps/desktop-ui/src/routes/+page.svelte` | Home 信号聚合展示 | ✓ VERIFIED | 已接 `stores/signals` 并渲染 SignalCard；聚合/排序维度不足。 |
| `crates/notification_adapter/src/lib.rs` | HIGH 通知适配器 | ✗ MISSING | 文件存在但仅 1 行注释，属于 stub。 |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `runtime_tauri::commands::subscription` | `application::subscription` | thin wrapper 调用 | ✓ WIRED | `subscribe/unsubscribe/pause/list/sync` 均已接线。 |
| `application::subscription::sync_subscriptions` | `github_adapter::releases` | `fetch_latest_releases/tags` | ✓ WIRED | 实际调用并处理结果。 |
| `application::subscription::sync_subscriptions` | `signal_repository::insert_signal` | `INSERT OR IGNORE` 幂等 | ✓ WIRED | 每条 release/tag 信号写入均走幂等接口。 |
| `SubscriptionSearch.svelte` | `stores/subscriptions` | 搜索结果订阅动作 | ✗ NOT_WIRED | 未调用 `addSubscription`。 |
| `Home +page.svelte` | `stores/signals` | `loadHomeSignals` on mount | ✓ WIRED | 页面装载即加载信号流。 |
| `signal_repository::list_home_signals` | Home 排序 contract | priority+time+source+affinity | ⚠️ PARTIAL | 仅 priority/time，缺 source/affinity。 |
| `application sync` | `notification_adapter` | HIGH 即时通知 | ✗ NOT_WIRED | 未见 notification adapter 调用。 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `subscriptions/+page.svelte` | `$subscriptions` | `stores/subscriptions.loadSubscriptions -> IPC list_subscriptions -> DB JOIN` | Yes | ✓ FLOWING |
| `routes/+page.svelte` | `$homeSignals` | `stores/signals.loadHomeSignals -> IPC list_home_signals -> signals table` | Yes | ✓ FLOWING |
| `SubscriptionSearch.svelte` | `result` | `fetchRepoInfo(owner/repo)` | Yes (repo info) | ✗ DISCONNECTED（未流向订阅创建） |
| `SignalCard.svelte` | `signal.state` 更新 | `markSeenAction/ackSignalAction -> IPC -> signal_repository` | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| 应用层订阅/信号主循环可编译 | `rtk cargo check -p application` | `cargo build (5 crates compiled)` | ✓ PASS |
| IPC 命令 wiring 可编译 | `rtk cargo check -p runtime_tauri` | `cargo build (7 crates compiled)` | ✓ PASS |
| 前端订阅/信号页面与 store 类型正确 | `rtk pnpm exec svelte-check --tsconfig ./tsconfig.json` | `0 errors, 7 warnings(既有 FilterPanel)` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| SUB-01 | 01/02/04/05/06/07/08 | 搜索仓库并创建订阅 | ✗ BLOCKED | 有订阅创建链路，但搜索页未接创建动作。 |
| SUB-02 | 01/02/04/05/06/07/08 | 编辑/暂停/删除订阅 | ✓ SATISFIED | `pause_subscription` + `unsubscribe` + UI 操作按钮。 |
| SUB-03 | 01/02/03/05 | 轮询 Releases/Tags 生成信号 | ✓ SATISFIED | `sync_subscriptions` + GitHub releases/tags adapter。 |
| SUB-04 | 01/05 | U1-U4 优先级去重 | ✗ BLOCKED | 未见 U1-U4 冲突消解编排。 |
| SUB-05 | 05/09 | 12h/24h digest 聚合信号 | ✗ BLOCKED | `new_digest` 未在同步流程使用。 |
| SUB-06 | 05/06/09 | HIGH 优先级桌面通知 | ✗ BLOCKED | notification adapter 空实现，应用层未触发。 |
| SUB-07 | 01/04/05/06/07/09 | 已读/已处理 | ✓ SATISFIED | `mark_signal_seen`/`ack_signal` 全链路已接。 |
| SUB-08 | 01/02/05 | 幂等同步（不重复信号） | ✓ SATISFIED | `signals.signal_key UNIQUE` + `INSERT OR IGNORE`。 |
| HOME-01 | 04/05/06/07/09 | Home 聚合自上次访问以来高优摘要 | ✗ BLOCKED | 未实现 last_visit 基线。 |
| HOME-02 | 04/05/09 | 按优先级+时间+来源类型+亲和度排序 | ✗ BLOCKED | 仅 priority/time 排序。 |

**Orphaned requirements check:** Phase 3 在 `REQUIREMENTS.md` 映射的 ID（SUB-01..08, HOME-01..02）均在 Phase 3 plans 的 `requirements` 字段中出现，**无 ORPHANED requirement**。

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/notification_adapter/src/lib.rs` | 1 | 空实现（仅注释） | 🛑 Blocker | HIGH 通知需求无法落地。 |
| `apps/desktop-ui/src/lib/components/SubscriptionSearch.svelte` | 88 | 功能错位（仅提示去 TopK 订阅） | ⚠️ Warning | 与“搜索并创建订阅”目标不一致。 |

### Human Verification Required

当前阶段存在阻塞性代码缺口（gaps_found），先补齐后再做人测更有效。建议补齐后重点做人测：

1. **信号优先级视觉与交互**
   - **Test:** 在 Home 页面实际点击 NEW 信号、ACK 信号，观察卡片样式与列表变化。
   - **Expected:** NEW->SEEN 样式变化，ACK 后从 Home 列表移除。
   - **Why human:** 视觉层级和交互流畅度无法通过静态检查完全确认。

### Gaps Summary

Phase 3 的“主循环骨架”已搭建（订阅 CRUD、轮询、幂等、Home 基本展示、IPC 全链路），但 **M2 目标的关键闭环仍未达成**：

1. 订阅搜索创建链路未完成（搜索只读，不可创建）；
2. digest 聚合未生成；
3. HIGH 通知未发送（适配器为空）；
4. Home 聚合排序维度不完整（缺 last_visit/source/affinity）。

因此当前判定为 **gaps_found**，尚不能认定 Phase 03 目标达成。

---

_Verified: 2026-03-23T14:11:36Z_
_Verifier: the agent (gsd-verifier)_
