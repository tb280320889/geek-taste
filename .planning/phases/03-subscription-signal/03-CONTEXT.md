# Phase 3: 订阅系统与信号模型 - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 3 delivers the subscription management system and signal model. Users can search repos and create subscriptions, manage subscription lifecycle (edit/pause/delete), receive deduplicated signals from release/tag polling, view digest-aggregated signals on Home page, and receive desktop notifications for HIGH priority events.

**Requirements covered:** SUB-01, SUB-02, SUB-03, SUB-04, SUB-05, SUB-06, SUB-07, SUB-08, HOME-01, HOME-02

</domain>

<decisions>
## Implementation Decisions

### 订阅搜索与创建
- D-01: 订阅搜索入口放在 `/subscriptions` 页面顶部搜索框 — 搜索 repo 后直接创建订阅，步骤集中
- D-02: 保持 Phase 2 的 SubscribePopover 预填逻辑（STANDARD/12h/HIGH 通知），`onConfirm` 直接调用 subscribe IPC
- D-03: 订阅列表用卡片列表展示 — 每张卡显示 repo 信息 + 状态标签 + 暂停/删除操作
- D-04: 创建订阅时自动建立 baseline（当前 release/tag/branch SHA），不回溯历史信号

### 信号轮询与同步引擎
- D-05: 应用启动时触发一次全量同步 + 手动刷新按钮 — v1 不做后台定时器
- D-06: 每次同步对每个活跃订阅调用 Releases List + Tags List API，使用 cursor 做增量 — 超过 cursor 的才处理
- D-07: 单 repo 失败仅记录 error log + 更新 last_synced_at，不阻塞其他订阅
- D-08: SQLite INSERT OR IGNORE on signal_key UNIQUE 约束实现幂等去重

### Digest 与 Home 页面信号流
- D-09: 用户打开 Home 时实时查询 signals 表（按 priority + occurred_at 排序），v1 不做前端缓存
- D-10: 同步完成后按 digest_window 对同一 repo 的多条信号做合并，生成一条 digest 类型 signal
- D-11: 信号卡片点击后自动标记 SEEN，提供「标记已处理」按钮设为 ACKED
- D-12: 无信号时显示「暂无新信号」+ 引导用户去 TopK 发现或创建订阅

### 桌面通知集成
- D-13: 同步过程中发现 HIGH 优先级信号且 notify_high_immediately=1 时立即调用 `tauri-plugin-notification`
- D-14: application 层检查 Settings 的 quiet_hours 配置，安静时段跳过通知发送，信号照常生成
- D-15: 通知格式：`{repo_name}: {signal_type_text} — {title}`
- D-16: `notification_adapter` crate 仅封装 `tauri-plugin-notification` 调用，不做业务判断

### agent's Discretion
- crate 内部模块组织、函数签名、SQL 查询优化、前端组件粒度由 agent 自行决定

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/persistence_sqlite/src/migrations.rs` — V001 migration 模式（rusqlite_migration），V002 直接扩展
- `crates/persistence_sqlite/src/repo_repository.rs` — Repository CRUD 可直接复用到订阅关联查询
- `crates/application/src/topk.rs:196` — `is_subscribed: false, // TODO: Phase 3` — 需要接入订阅表交叉检查
- `crates/runtime_tauri/src/commands/topk.rs` — IPC 命令薄封装模式（get_db_connection → application 调用 → 返回 DTO）
- `crates/github_adapter/src/rate_limit.rs` — RateBudget 可复用到 release/tag API 调用
- `apps/desktop-ui/src/lib/ipc/tauri.ts` — IPC wrapper 层模式
- `apps/desktop-ui/src/lib/stores/` — writable + async action 模式
- `apps/desktop-ui/src/lib/components/SubscribePopover.svelte` — 预填订阅 UI 已就绪，需接入 IPC

### Established Patterns
- Hexagonal Architecture：domain → application → adapters → runtime（依赖单向）
- SvelteKit 5 Runes：`$state()`, `$derived()`, `$effect()`, `$props()`
- Tauri v2 IPC：`invoke()` 命令调用，通过 `$lib/ipc/tauri.ts` 封装
- DB 连接每次 IPC 调用独立打开（WAL 模式）
- Fresh Octocrab client per call（简化生命周期）

### Integration Points
- `crates/domain/src/lib.rs` — 新增 `mod subscription`, `mod signal`
- `crates/application/src/lib.rs` — 新增 `mod subscription`, `mod signal`
- `crates/persistence_sqlite/src/lib.rs` — 新增 `mod subscription_repository`, `mod signal_repository` + V002 migration
- `crates/github_adapter/src/lib.rs` — 新增 `mod releases`（fetch releases + tags）
- `crates/notification_adapter/src/lib.rs` — 从 stub 实现为实际通知封装
- `crates/shared_contracts/src/lib.rs` — 新增 `mod subscription_dto`, `mod signal_dto`
- `crates/runtime_tauri/src/commands/mod.rs` — 新增 `mod subscription`, `mod signal`
- `apps/desktop-ui/src-tauri/src/lib.rs` — 注册新命令到 generate_handler![]
- `apps/desktop-ui/src/routes/subscriptions/+page.svelte` — 从 placeholder 重写为订阅管理页
- `apps/desktop-ui/src/routes/+page.svelte` — 接入信号摘要
- `apps/desktop-ui/src/lib/types.ts` — 新增 Subscription/Signal 类型
- `apps/desktop-ui/src/lib/ipc/tauri.ts` — 新增 subscription/signal IPC 函数
- `apps/desktop-ui/src/lib/stores/` — 新增 subscriptions.ts, signals.ts

</code_context>

<specifics>
## Specific Ideas

- 订阅搜索交互参考 GitHub 的 repo 搜索体验 — 输入关键词实时显示匹配结果，点击即订阅
- SubscribePopover 的 `onConfirm` 直接调用 `subscribe(repoId, settings)` IPC，成功后 toast + 按钮变「已订阅」
- 信号卡片参考 Linear 的通知收件箱 — 左侧优先级色条 + 中间标题摘要 + 右侧时间
- Home 页面信号流按 priority 分组：HIGH 在顶部突出显示，MEDIUM/LOW 在下方列表

</specifics>

<deferred>
## Deferred Ideas

- 后台定时轮询（tokio-cron-scheduler）— v2 考虑
- 每条订阅可配置独立的通知级别 — v1 使用全局设置
- 安静时段 UI 配置增强（当前已有基础配置）
- PR_MERGED_DIGEST 高级模式 — v2 范围
- 订阅导入/导出功能

</deferred>

---

*Phase: 03-subscription-signal*
*Context gathered: 2026-03-23*
*Decisions captured: 16 across 4 areas*
