# geek taste

## What This Is

`geek taste` 是一个面向重度 GitHub 观察者与 AI coding 采用者的**跨端技术雷达工作台**。用 TopK 榜单发现值得关注的新趋势，用 Repo 订阅跟踪值得处理的可用更新，用 Agent 资源雷达发现与当前语言/框架相关的 MCP / Skills / Agent 生产力资源。核心输出不是"更多信息"，而是**高信噪比、可行动、低打扰的技术信号**。

## Core Value

**信号质量高于一切。** 如果 everything else fails，用户仍然能在 30 秒内完成判断：今天有哪些仓库/资源值得看。

## Requirements

### Validated

- [x] 用户能输入 GitHub PAT 并保存到 OS 安全存储 — *Validated in Phase 01: scaffold-auth*
- [x] 用户能查看仓库基本信息（stars, forks, description, language, topics） — *Validated in Phase 01: scaffold-auth*
- [x] 用户能配置通知频率、语言兴趣、安静时段等设置 — *Validated in Phase 01: scaffold-auth*
- [x] 应用提供 Home/TopK/Subscriptions/Resources/Rules 导航壳与路由 — *Validated in Phase 01: scaffold-auth*
- [x] 用户能通过 TopK 榜单按语言/框架/主题/时间窗发现趋势项目 — *Validated in Phase 02: topk*
- [x] 用户能从 TopK 一键订阅感兴趣的 repo — *Validated in Phase 02: topk*
- [x] 用户能按语言/框架发现 Agent 生产力资源（MCP/Skills/Agent）— *Validated in Phase 04: agent-resources-radar*

### Active
- [ ] 用户能管理订阅列表（创建/编辑/暂停/删除）
- [ ] 系统能按默认事件类型同步订阅 repo 的可用更新（release、tag、branch digest）
- [ ] 用户能在 Home/Today 页面看到自上次访问以来的高优先级信号摘要
- [ ] 系统能生成 digest（12h/24h 窗口）并支持桌面通知
- [ ] 用户能标记 signal 为已读/已处理
- [ ] 应用可离线打开并展示上次同步缓存
- [ ] GitHub token 安全存储于 OS 安全存储

### Out of Scope

- GitHub 全站搜索替代品 — 不与 GitHub 本体竞争
- Issues/PR/Discussions 全功能收件箱 — 范围过大，偏离信号聚焦
- 秒级实时告警系统 — GitHub Events API 延迟 30s-6h，不承诺实时
- 用户间社交互动/协作 — v1 不做内容社区
- 全自动 LLM 摘要作为核心路径 — 成本/稳定性风险，v1 用规则+模板
- SurrealDB — 域模型不需要图数据库灵活性
- Web-first 门户 — 桌面优先
- 覆盖 npm/crates/PyPI 等多源爬取 — v1 数据源仅限 GitHub

## Context

- **目标用户**：重度 GitHub 跟踪者 + AI coding 工具采用者（每周多次查看 GitHub、按趋势调整选型、使用 Cursor/Claude Code/Windsurf 等 agent 工具、关心 MCP/Skills 生态、有一批长期关注的仓库）
- **技术环境**：Tauri + SvelteKit(static-adapter) + SQLite，Rust 四层架构（Presentation/Application/Domain/Infrastructure）
- **核心交互流**：冷启动（选兴趣 → TopK → 订阅 → Home）→ 回访（缓存 → 信号摘要 → 看变更/找新项目/找 Agent 资源）
- **数据策略**：轮询 + 差分 + 摘要 + 本地缓存；REST-first；ETag 条件请求；速率预算隔离（core vs search）

## Constraints

- **平台**: v1 正式支持 macOS + Windows 桌面端（Tauri）；Web/Linux/移动仅保留技术准备
- **数据源**: v1 MUST 使用 GitHub 官方 API（Repos/Search/Releases/Tags/Events）；禁止多源爬取
- **前端架构**: SvelteKit 必须使用 static-adapter SPA 模式；Tauri 不支持 server-based frontend
- **数据库**: SQLite 为 v1 主权威数据库；Turso 仅在跨设备同步/云调度时引入
- **API 速率**: 认证 5000 req/hour，Search 30 req/min；scheduler MUST 做端点级限流与退避
- **安全**: GitHub token MUST NOT 明文存入 SQLite；MUST 使用 OS 安全存储
- **同步**: 不得依赖 webhook（需要 owner/admin 权限）；Events API 不是实时接口

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 桌面优先，本地优先 | 本地缓存/离线/通知/token管理在桌面端更稳 | — Pending |
| SQLite 为主数据库 | 强结构化、本地优先适配、事务/索引成熟、AI生成代码约束清晰 | — Pending |
| REST-first | Search/Releases/Tags 端点清晰；更适合 agent loop 稳定生成调试 | — Pending |
| 轮询+差分+摘要 | Webhook 需要 owner 权限；Events API 延迟高；轮询可控 | — Pending |
| TopK = 产品定义排行榜 | 不绑定 GitHub Trending；由查询模板+过滤器+快照+评分公式定义 | — Pending |
| 规则+模板摘要为主 | LLM 摘要成本/稳定性/时延风险高；v1 用确定性方案 | — Pending |
| SurrealDB 不进入 v1 | 域模型不需图数据库灵活性；SQLite 确定性和成熟度更重要 | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-24 after Phase 04 (agent-resources-radar) completion*
