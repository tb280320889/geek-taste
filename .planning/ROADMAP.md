# Roadmap: geek taste

**Created:** 2026-03-22
**Granularity:** standard
**Total v1 Requirements:** 29
**Phases:** 6

## Phases

- [x] **Phase 0: moonrepo 工程化基建** — moon monorepo 配置、目录结构、Cargo workspace、SvelteKit+Tauri 骨架、CI/CD pipeline
- [ ] **Phase 1: 项目脚手架与认证** — GitHub Token 认证、OS 安全存储、导航壳、基础仓库信息展示
- [x] **Phase 2: 数据层与 TopK 发现引擎** — SQLite + GitHub REST 客户端、TopK 排名/筛选/快照/评分、一键订阅
- [x] **Phase 3: 订阅系统与信号模型** — 订阅 CRUD、Release/Tag 轮询、Signal 去重/排序/Digest/通知、Home 页面 (completed 2026-03-23)
- [ ] **Phase 4: Agent 资源雷达** — MCP/Skills/Agent 资源发现、分类评分、推荐解释
- [ ] **Phase 5: 打磨与发布准备** — 离线降级、错误处理、性能优化、Momentum 暖机、代码签名

## Phase Details

### Phase 0: moonrepo 工程化基建
**Goal**: 搭建符合 moon/rust 最佳实践的 monorepo 工程骨架，所有后续阶段在此骨架上开发
**Depends on**: Nothing (first phase)
**Requirements**: (无用户需求 — 基础设施阶段)
**Scope**:
  1. 安装并配置 moonrepo/moon，初始化 `.moon/workspace.yml`
  2. 设计目录结构（参考 docs/03 架构规范 + moon/rust 最佳实践）：
     - `apps/desktop-ui` — SvelteKit + Tauri frontend shell
     - `crates/domain` — 纯领域对象与规则
     - `crates/application` — 用例编排
     - `crates/github_adapter` — GitHub REST client + mapping
     - `crates/persistence_sqlite` — SQLite repository impl
     - `crates/notification_adapter` — 桌面通知
     - `crates/runtime_tauri` — Tauri commands / bootstrap
     - `crates/runtime_server` — Axum (future)
     - `crates/shared_contracts` — JSON schema / DTO / enum export
  3. 配置 Cargo workspace（`Cargo.toml` + workspace members）
  4. 初始化 Tauri v2 + SvelteKit 5 项目骨架（static-adapter SPA 模式）
  5. 配置 moon tasks：build、test、lint、format、check
  6. 配置 CI/CD pipeline（moon ci / moon run）
  7. 配置开发工具链：rustfmt、clippy、eslint/prettier（SvelteKit）
  8. 验证 moon build pipeline 全绿
**Success Criteria** (what must be TRUE):
  1. `moon ci` 全部通过（build + test + lint）
  2. 目录结构与 docs/03 架构规范一致
  3. Tauri v2 应用可冷启动（空白页面不崩溃）
  4. SvelteKit SPA 在 Tauri WebView 中正确渲染
  5. Cargo workspace 各 crate 可独立编译
  6. 开发者可 `moon run desktop-ui:dev` 启动开发环境
**Plans**: 5 plans
Plans:
- [x] 00-01-PLAN.md — 目录结构与 moon 配置
- [x] 00-02-PLAN.md — Cargo workspace 配置
- [x] 00-03-PLAN.md — Tauri v2 + SvelteKit 5 SPA 骨架
- [x] 00-04-PLAN.md — moon 任务与 CI/CD
- [x] 00-05-PLAN.md — 开发工具链配置

### Phase 1: 项目脚手架与认证
**Goal**: 用户能启动应用、认证 GitHub、看到可用的导航结构
**Depends on**: Phase 0
**Requirements**: FOUND-01, FOUND-02, FOUND-03, FOUND-04
**Success Criteria** (what must be TRUE):
  1. 用户能输入 GitHub PAT 并保存到 OS 安全存储（keyring），应用能用该 token 调用 GitHub API
  2. 用户能查看仓库基本信息（stars, forks, description, language, topics）
  3. 用户能配置通知频率、语言兴趣、安静时段等设置
  4. 应用提供 Home/TopK/Subscriptions/Resources/Rules 五个导航页面，路由可切换
**Plans**: 6 plans
Plans:
- [x] 01-01-PLAN.md — Cargo workspace + runtime_tauri + domain/application/infrastructure crates + GitHub auth keyring
- [x] 01-02-PLAN.md — GitHub auth Tauri commands (validate/store/remove/get_user) + settings commands
- [x] 01-03-PLAN.md — 导航壳与路由 (Sidebar + 7 routes + auth guard + stores + IPC wrapper)
- [x] 01-04-PLAN.md — Onboarding 流程 (已在 01-03 中实现)
- [x] 01-05-PLAN.md — Settings 页面 (已在 01-03 中实现)
- [x] 01-06-PLAN.md — Home 欢迎页与仓库探索 (已在 01-03 中实现)

### Phase 2: 数据层与 TopK 发现引擎
**Goal**: 用户能通过 TopK 榜单发现趋势项目并一键订阅
**Depends on**: Phase 1
**Requirements**: INFRA-01, INFRA-02, INFRA-03, TOPK-01, TOPK-02, TOPK-03, TOPK-04, TOPK-05, TOPK-06, TOPK-07, TOPK-08
**Success Criteria** (what must be TRUE):
  1. 用户能按 STARS_DESC 和 UPDATED_DESC 排序查看趋势仓库，按 language/topic/stars 阈值筛选
  2. 用户能保存多个过滤+排序组合为可复用的 RankingView，下次直接使用
  3. 系统每 12h 自动创建排名快照，支持时间对比查看趋势变化
  4. 系统支持 Momentum 评分公式（0.5×star_delta + 0.2×fork_delta + 0.3×recency），榜单变化时生成 VIEW_CHANGED 信号
  5. 用户能从 TopK 榜单项一键订阅仓库，直接进入订阅管理
  6. GitHub API 速率预算按 core/search 端点隔离管理，同步支持 ETag 条件请求和增量刷新
  7. SQLite 使用 WAL 模式，支持并发读写
**Plans**: 6 plans
Plans:
- [x] 02-01-PLAN.md — 领域模型层（Repository / RankingView / Momentum 评分）
- [x] 02-02-PLAN.md — SQLite 持久化层（Migration + CRUD）
- [x] 02-03-PLAN.md — GitHub Search 客户端 + 速率预算
- [x] 02-04-PLAN.md — 应用层编排 + Tauri IPC 命令
- [x] 02-05-PLAN.md — 前端 IPC + Store + Types
- [x] 02-06-PLAN.md — 前端 TopK UI（视图选择/筛选/排名/订阅）

### Phase 3: 订阅系统与信号模型
**Goal**: 用户能管理订阅、接收高信噪比的技术信号、在 Home 页面一览全局
**Depends on**: Phase 2
**Requirements**: SUB-01, SUB-02, SUB-03, SUB-04, SUB-05, SUB-06, SUB-07, SUB-08, HOME-01, HOME-02
**Success Criteria** (what must be TRUE):
  1. 用户能搜索仓库并创建订阅，能编辑/暂停/删除已有订阅
  2. 系统轮询已订阅仓库的 Releases 和 Tags 生成信号，同一信号不会重复生成（幂等同步）
  3. 系统按 U1-U4 优先级规则去重（RELEASE_PUBLISHED > TAG_PUBLISHED > DEFAULT_BRANCH_ACTIVITY_DIGEST），HIGH 优先级信号触发桌面通知
  4. 系统生成 12h/24h digest 窗口内的聚合信号，用户能标记信号为已读/已处理
  5. Home 页面聚合自上次访问以来的高优先级信号摘要，按优先级 + 时间 + 来源类型 + 用户亲和度排序
**Plans**: TBD

### Phase 4: Agent 资源雷达
**Goal**: 用户能按技术栈发现 MCP/Skills/Agent 生产力资源
**Depends on**: Phase 2, Phase 3
**Requirements**: RES-01, RES-02, RES-03
**Success Criteria** (what must be TRUE):
  1. 用户能按语言/框架发现 MCP/Skills/Agent 资源列表
  2. 系统按 stack_relevance + star_delta + recency 为资源评分排序
  3. 每条资源推荐展示"为什么推荐给我"的解释说明
**Plans**: 3 plans
Plans:
- [x] 04-01-PLAN.md — 领域模型与 SQLite 持久化（Resource 类型 + V003 migration + repository）
- [x] 04-02-PLAN.md — 应用层编排与 Tauri IPC（评分 + 推荐解释 + 精选 + 命令）
- [x] 04-03-PLAN.md — 前端界面（ResourceCard + ResourceFilters + Resources 页面）

### Phase 5: 打磨与发布准备
**Goal**: 应用可离线使用、性能达标、可签名发布
**Depends on**: Phase 3, Phase 4
**Requirements**: HOME-03
**Success Criteria** (what must be TRUE):
  1. 应用可离线打开并展示上次同步缓存，标记 STALE 状态
**Plans**: 1 plan
Plans:
- [ ] 05-01-PLAN.md — 离线降级、错误处理与 Momentum 暖机

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 0. moonrepo 工程化基建 | 5/5 | Complete | 2026-03-22 |
| 1. 项目脚手架与认证 | 6/6 | Complete | 2026-03-23 |
| 2. 数据层与 TopK 发现引擎 | 6/6 | Complete | 2026-03-23 |
| 3. 订阅系统与信号模型 | 9/9 | Complete   | 2026-03-23 |
| 4. Agent 资源雷达 | 0/3 | Planning | - |
| 5. 打磨与发布准备 | 0/1 | Not started | - |

## Coverage

| Requirement | Phase | Status |
|-------------|-------|--------|
| FOUND-01 | Phase 1 | ✅ Complete |
| FOUND-02 | Phase 1 | ✅ Complete |
| FOUND-03 | Phase 1 | ✅ Complete |
| FOUND-04 | Phase 1 | ✅ Complete |
| INFRA-01 | Phase 2 | ✅ Complete (02-02) |
| INFRA-02 | Phase 2 | ✅ Complete (02-03) |
| INFRA-03 | Phase 2 | ✅ Complete (02-02) |
| TOPK-01 | Phase 2 | ✅ Complete (02-03) |
| TOPK-02 | Phase 2 | ✅ Complete (02-04) |
| TOPK-03 | Phase 2 | ✅ Complete (02-03 + 02-06) |
| TOPK-04 | Phase 2 | ✅ Complete (02-04) |
| TOPK-05 | Phase 2 | ✅ Complete (02-04) |
| TOPK-06 | Phase 2 | ✅ Complete (02-04) |
| TOPK-07 | Phase 2 | ✅ Complete (02-04) |
| TOPK-08 | Phase 2 | ✅ Complete (02-06) — subscribe popover UI ready, actual IPC in Phase 3 |
| SUB-01 | Phase 3 | Pending |
| SUB-02 | Phase 3 | Pending |
| SUB-03 | Phase 3 | Pending |
| SUB-04 | Phase 3 | Pending |
| SUB-05 | Phase 3 | Pending |
| SUB-06 | Phase 3 | Pending |
| SUB-07 | Phase 3 | Pending |
| SUB-08 | Phase 3 | Pending |
| HOME-01 | Phase 3 | Pending |
| HOME-02 | Phase 3 | Pending |
| HOME-03 | Phase 5 | Pending |
| RES-01 | Phase 4 | Pending |
| RES-02 | Phase 4 | Pending |
| RES-03 | Phase 4 | Pending |

**Coverage: 29/29 v1 requirements mapped ✓**

## Research Flags

| Phase | Flag | Action |
|-------|------|--------|
| Phase 0 | moonrepo/moon + Tauri v2 + SvelteKit 集成验证 | Spike: moon init + Tauri scaffold in moon workspace |
| Phase 1 | specta/ts-rs 类型桥接版本兼容性 | 2-day spike to validate |
| Phase 2 | GitHub Search API 结果分布和排序稳定性 | Design accepts non-determinism, use snapshot comparison |
| Phase 4 | MCP/Skills 分类可操作性边界 | Deeper research before implementation |

## Milestone Alignment

| Milestone | Phase | Notes |
|-----------|-------|-------|
| M0: Spec freeze | ✅ Done | Specs exist in docs/ |
| M1: Basic shell & local state | Phase 0 + Phase 1 | moon scaffold + Tauri + token + UI shell |
| M2: Subscription main loop | Phase 3 | repo search/subscribe/sync/signal/digest/notification |
| M3: TopK ranking engine | Phase 2 (core) | saved views/snapshots/ranking/change signals |
| M4: Resource Radar | Phase 4 | resource classification/relevance scoring |
| M5: Hardening & release | Phase 5 | performance/security/testing/packaging |

---
*Roadmap created: 2026-03-22*
*Updated: 2026-03-22 — added Phase 0 (moonrepo monorepo setup)*
*Ready for: `/gsd-execute-phase 0`*
