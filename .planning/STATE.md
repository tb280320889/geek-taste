---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: Executing Phase 04
stopped_at: Completed 04-02-PLAN.md
last_updated: "2026-03-24T02:38:00.000Z"
progress:
  total_phases: 6
  completed_phases: 5
  total_plans: 33
  completed_plans: 32
---

# State: geek taste

## Project Reference

**Core Value:** 高信噪比、可行动、低打扰的技术信号
**What This Is:** 跨端技术雷达工作台 — TopK 趋势发现 + Repo 订阅跟踪 + Agent 资源雷达
**Stack:** Tauri v2 + SvelteKit 5 + SQLite (rusqlite) + octocrab

## Current Position

Phase: 04 (agent-resources-radar) — EXECUTING
Plan: 3 of 3

## Phase Summary

| Phase | Goal | Reqs | Status |
|-------|------|------|--------|
| 0. moonrepo 工程化基建 | monorepo 配置 + 目录结构 + Cargo workspace + CI/CD | — | Complete |
| 1. 项目脚手架与认证 | 启动应用、认证 GitHub、导航结构 | 4 | Complete |
| 2. 数据层与 TopK 发现引擎 | SQLite + GitHub 客户端 + TopK 排名 | 11 | Complete |
| 3. 订阅系统与信号模型 | 订阅 CRUD + Signal + Home | 10 | Complete |
| 4. Agent 资源雷达 | MCP/Skills/Agent 资源发现 | 3 | In Progress |
| 5. 打磨与发布准备 | 离线 + 性能 + 发布 | 1 | In Progress |

## Performance Metrics

| Metric | Value |
|--------|-------|
| Requirements total | 29 |
| Requirements validated | 0 |
| Phases complete | 3/6 |
| Current phase progress | 6/6 plans |
| Phase 01-scaffold-auth P01 | 20min | 2 tasks | 2 files |
| Phase 01-scaffold-auth P02 | 8min | 4 tasks | 16 files |
| Phase 01-scaffold-auth P03 | 15min | 5 tasks | 18 files |
| Phase 01-scaffold-auth P04 | 0min | 0 tasks | 0 files |
| Phase 01-scaffold-auth P05 | 0min | 0 tasks | 0 files |
| Phase 01-scaffold-auth P06 | 0min | 0 tasks | 0 files |
| Phase 02-topk P01 | 10min | 3 tasks | 5 files |
| Phase 02-topk P02 | 15min | 3 tasks | 5 files |
| Phase 02-topk P03 | 15min | 2 tasks | 4 files |
| Phase 02-topk P04 | 12min | 2 tasks | 7 files |
| Phase 02-topk P05 | 8min | 3 tasks | 3 files |
| Phase 02-topk P06 | 14min | 3 tasks | 5 files |
| Phase 03 P01 | 5 min | 2 tasks | 3 files |
| Phase 03 P02 | 2 min | 3 tasks | 4 files |
| Phase 03 P10 | 9min | 2 tasks | 6 files |
| Phase 03 P11 | 11min | 2 tasks | 4 files |
| Phase 03 P12 | 5min | 2 tasks | 5 files |
| Phase 04 P01 | 2min | 2 tasks | 5 files |
| Phase 04-agent-resources-radar P01 | 2min | 2 tasks | 5 files |
| Phase 04-agent-resources-radar P02 | 2min | 2 tasks | 4 files |

## Accumulated Context

### Key Decisions

- 桌面优先，本地优先 — SQLite 为主权威数据库
- REST-first — Search/Releases/Tags 端点清晰
- 轮询+差分+摘要 — 不依赖 webhook
- TopK = 产品定义排行榜 — 非 GitHub Trending 复制品
- 规则+模板摘要为主 — v1 不做全自动 LLM 摘要
- AuthToken::is_expired() 采用 24h 窗口 — v1 简化: 启动时验证
- serde_rusqlite 0.41 — 唯一兼容 rusqlite 0.38 的版本
- Fresh Octocrab client per call — 简化生命周期，无共享状态
- ValidateTokenResponse 包装 success/error — 前端友好
- Unicode 图标替代 icon 库 — v1 无额外依赖，桌面端可接受
- Auth guard 集中在 layout — Sidebar 始终可见，未认证显示提示卡
- IPC wrapper 层 (tauri.ts) — 组件不直接调 invoke()，关注点分离
- Settings auto-save — 每次变更即时保存 + toast 反馈
- Onboarding 2-step: validate → confirm — 防止误存错误 token
- Plan 05 + 06 功能已在 01-03 中实现 — 验证通过，无需代码改动
- RankingMode 使用 Rust enum (StarsDesc) + Display 返回 UPPER_SNAKE — 保持 Rust 类型安全，字符串仅在 DTO 边界
- MomentumScore max_delta 默认 1000.0 — 按 plan 规范，star/fork 增量超过 1000 即为满分
- From 转换仅 domain→DTO 方向 — DTO 不回退到领域对象
- RateBudget 使用 sync Mutex (非 tokio::Mutex) — Tauri 单线程应用
- Pool auto-reset on check() — 无需后台定时器
- octocrab models Repository.language 为 serde_json::Value — 需 match 提取 String
- application 层直接依赖 persistence_sqlite + github_adapter — v1 务实选择
- DB 连接每次 IPC 调用独立打开 (WAL 模式) — 避免共享状态复杂性
- 暖机快照失败仅 warn — 不阻塞视图创建
- SubscribePopover 使用 absolute 定位而非 fixed — 相对于 RankingList 容器
- 排名变化标识：rank_change > 0 → 绿 +↑N，< 0 → 红 -↓N，=== 0 → 灰 —，null → 不显示
- 订阅确认当前仅关闭 Popover — 实际订阅由 Phase 3 接入
- Subscription::new(repo_id) 统一默认值入口 — 避免上层重复拼装字段
- Signal 构造函数内置 signal_key 规则 — 领域层保证去重键唯一来源
- Subscription/Signal 状态迁移在 domain 层显式校验 — 防止非法状态写入持久化
- signals 表通过 signal_key UNIQUE + INSERT OR IGNORE 实现同步幂等
- subscription 查询保留 repo JOIN 结果结构，复用现有 application DTO 映射路径
- ResourceKind::Other(String) 兜底未知类型 — 保持前向兼容
- compute_resource_score 使用 0.4/0.35/0.25 权重 — stack_relevance 优先于 star 增长
- search_resources 使用 LIKE 匹配 languages_json — 小数据量简单有效，避免 JSON1 扩展依赖
- Stack relevance 使用 Jaccard 相似度 — 从订阅仓库推断用户语言兴趣，|intersection|/|union|
- 推荐解释使用 RecommendationReason::to_template() — 模板规则生成，非 LLM
- ResourcesToDtos 按 score 降序排列 — 高相关资源优先展示

### Known Risks

- Tauri v2 ACL 权限遗漏 → Phase 1 必须验证
- SQLite 并发写入 → Phase 2 启用 WAL + busy_timeout
- GitHub API 速率预算 → Phase 2 core/search 分池隔离
- SvelteKit static-adapter 误用 server 功能 → Phase 1 配置 ssr=false

### Blockers

- None currently

## Session Continuity

**Last session:** 2026-03-24T02:38:00Z
**Stopped at:** Completed 04-02-PLAN.md
**Next action:** 执行 04-03 资源前端页面组件计划

---
*Last updated: 2026-03-24 — Phase 04 Plan 02 complete*
