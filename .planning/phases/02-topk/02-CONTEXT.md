# Phase 2: 数据层与 TopK 发现引擎 - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 2 delivers the SQLite persistence layer, GitHub REST client (search/releases/tags), and the TopK discovery engine — users can create/manage RankingViews, discover trending repos via sorting/filtering, view snapshot-based trend changes, compute Momentum scores, and one-click subscribe from the ranking list.

**Requirements covered:** INFRA-01, INFRA-02, INFRA-03, TOPK-01, TOPK-02, TOPK-03, TOPK-04, TOPK-05, TOPK-06, TOPK-07, TOPK-08

</domain>

<decisions>
## Implementation Decisions

### RankingView 创建
- **D-01:** 支持两种创建路径：(a) 预设模板（3-5 个内置，如「Rust 热门」「本周活跃高星」）选中即用；(b) 从零自定义筛选条件后保存
- **D-02:** 预设模板可在此基础上微调后另存为自定义视图

### RankingView 编辑
- **D-03:** 原地编辑模式 — 在当前 TopK 页面筛选面板中直接修改条件（language/stars 阈值/排序模式/k 值）
- **D-04:** 改动即时生效刷新结果，提供「保存」（覆盖原视图）和「另存为」（创建新视图）两个操作

### 视图切换
- **D-05:** TopK 页面顶部使用下拉选择器展示当前视图名称，展开列出所有已保存视图
- **D-06:** 下拉列表中支持 Pin/Unpin 排序、删除视图操作

### 快照触发
- **D-07:** 混合触发策略 — 后台定时器每 12h 为所有已保存视图创建快照（tokio-cron-scheduler）+ 用户打开视图时若距上次快照 >12h 则补拍一张
- **D-08:** 新 RankingView 创建时立即触发一次快照（暖机），之后进入 12h 定时周期

### 评分可视化
- **D-09:** 列表中每项仅显示综合 Momentum 分（如 0.87）+ 排序模式标签
- **D-10:** 鼠标悬停综合分时展开 Tooltip 显示三个细分维度（starDelta / forkDelta / updatedRecency）的贡献值

### 排名变化标识
- **D-11:** 与上次 snapshot 对比：排名上升标绿 `+↑N`，下降标红 `-↓N`，不变标灰 `—`
- **D-12:** 首次快照（无历史对比）时不显示变化标识

### 一键订阅确认
- **D-13:** 点击「订阅」按钮弹出小型预填面板（Popover），非全屏 Modal
- **D-14:** 默认设置为固定值：STANDARD 模式 / 12h digest / HIGH 立即通知 / event_types 排除 DEFAULT_BRANCH_ACTIVITY_DIGEST
- **D-15:** 用户可在面板中微调设置后确认订阅，订阅成功后 toast 通知 + 按钮变为「已订阅」状态

### SQLite 基础设施（agent 决定）
- **D-16:** 使用 `rusqlite_migration` 管理 schema 变更（需添加到 persistence_sqlite crate 依赖）
- **D-17:** 启用 WAL 模式 + busy_timeout（5000ms），满足并发读写需求
- **D-18:** 初始 migration 创建 Phase 2 所需的 4 张表：`repositories`, `repo_snapshots`, `ranking_views`, `ranking_snapshots`

### GitHub API 客户端扩展（agent 决定）
- **D-19:** 在 `github_adapter` 中扩展 `search_repositories()` 函数，支持 sort/filter/pagination 参数
- **D-20:** 速率预算管理器：core 池 5000 req/h，search 池 30 req/min，端点级隔离
- **D-21:** ETag 条件请求用于 repo 元数据增量刷新（`If-None-Match`）

### 分页模型（agent 决定）
- **D-22:** TopK 前端使用 offset-based 分页，默认页大小 30，最大 100
- **D-23:** GitHub Search API 硬上限 1000 条结果，RankingView 的 k_value ≤ 1000

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### 数据模型与契约
- `docs/05_data_model_and_contracts.md` — 全部 9 张表 DDL、Contract 对象定义（RepositoryCard, RankingViewSpec, RankingItem）、去重键规则、状态机、迁移策略

### 排名/订阅/通知规范
- `docs/04_ranking_subscription_notification_spec.md` — TopK 语义定义、Momentum 评分公式、信号生成规则、去重逻辑（U1-U4）

### 系统架构
- `docs/03_system_architecture_spec.md` — Rust 四层架构、crate 依赖方向、GitHub 集成架构（ETag/速率预算/轮询纪律）、SQLite 嵌入式策略

### 领域语言
- `docs/01_p0_domain_language_spec.md` — Repository/Subscription/Signal/Resource 统一术语定义

### Phase 1 上下文
- `.planning/phases/01-scaffold-auth/01-CONTEXT.md` — 已建立的认证、导航、IPC wrapper 模式

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/github_adapter/src/auth.rs` — octocrab 客户端创建模式（`Octocrab::builder().personal_token()`），可直接复用到 search/releases/tags
- `crates/domain/src/auth.rs`, `settings.rs` — 领域模型 struct/enum/derive 模式（thiserror, chrono, Default）
- `crates/shared_contracts/` — DTO + `From` 双向转换模式，新增 RankingViewSpec/RankingItem DTO 遵循此模式
- `crates/runtime_tauri/src/commands/` — Tauri command 薄封装模式（keyring 加载 token → adapter 调用 → 返回 DTO）
- `apps/desktop-ui/src/lib/ipc/tauri.ts` — 前端 IPC wrapper 层（typed invoke 封装）
- `apps/desktop-ui/src/lib/stores/` — Svelte writable + async load/update 模式
- `RepoInfoModal.svelte` — Modal 卡片模式，可复用到 TopK 详情
- `LanguagePicker.svelte` — 多选标签组件，直接复用到 TopK 语言筛选
- `SettingsGroup.svelte` — 卡片布局，可复用到筛选面板

### Established Patterns
- Hexagonal Architecture：domain → application → adapters → runtime（依赖单向）
- SvelteKit 5 Runes：`$state()`, `$derived()`, `$effect()`, `$props()`
- Tauri v2 IPC：`invoke()` 命令调用，通过 `$lib/ipc/tauri.ts` 封装
- Workspace dependencies：根 `Cargo.toml` 统一版本，crate 内 path dep

### Integration Points
- `crates/persistence_sqlite/src/lib.rs` — 空 stub，需创建 migration + repository 模块
- `crates/application/src/lib.rs` — 空 stub，需创建 TopK 用例编排
- `crates/domain/src/lib.rs` — 新增 `mod repository`, `mod ranking`
- `crates/shared_contracts/src/lib.rs` — 新增 `mod ranking_dto`
- `crates/runtime_tauri/src/commands/` — 新增 `mod topk`，注册到 `lib.rs` generate_handler
- `apps/desktop-ui/src/routes/topk/+page.svelte` — 当前仅搜索框，需重写为排名引擎 UI
- Workspace Cargo.toml — `rusqlite_migration 2.4` 已声明但需添加到 persistence_sqlite crate deps

</code_context>

<specifics>
## Specific Ideas

- RankingView 预设模板参考：(1) 「Rust 热门」— language=Rust, STARS_DESC, k=50；(2) 「本周活跃高星」— updatedSinceDays=7, MOMENTUM_7D, k=30；(3) 「新项目爆发」— minStars=10, CREATED_DESC, k=50
- 订阅弹出面板 UI 参考 Linear 的 issue 快捷操作 Popover — 紧凑、预填、可微调
- 排名变化箭头参考股票应用的涨跌标识风格

</specifics>

<deferred>
## Deferred Ideas

- 用户自定义 Momentum 权重（v2 考虑，v1 固定 0.5/0.2/0.3）
- RankingView 导出/分享功能
- 跨视图批量快照触发（目前逐视图独立触发）
- GitHub Search API 结果去重优化（1000 条上限外的候选发现）

</deferred>

---

*Phase: 02-topk*
*Context gathered: 2026-03-23*
