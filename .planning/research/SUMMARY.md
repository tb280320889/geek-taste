# Project Research Summary

**Project:** geek taste — 跨端技术雷达工作台
**Domain:** Desktop-first GitHub 技术雷达与信号发现应用
**Researched:** 2026-03-22
**Confidence:** HIGH

## Executive Summary

`geek taste` 是一个面向重度 GitHub 观察者的**桌面技术雷达工作台**——不是另一个 GitHub Trending 复制品或 star 管理器。它通过三线统一的 Signal 模型（TopK 趋势发现 + Repo 订阅跟踪 + Agent 资源雷达），为用户提供高信噪比、可行动的技术信号。竞品分析显示，没有任何现有工具同时覆盖这三条线，且桌面端技术雷达工作台是一个明确的空白领域。

技术栈研究一致推荐 **Tauri v2 + SvelteKit 5 + SQLite（rusqlite）+ octocrab（GitHub API）** 这一组合。Tauri v2 已完全稳定（2024-09 发布，当前 2.6.x），包体仅 ~4MB，远优于 Electron。SvelteKit 通过 `adapter-static` SPA 模式与 Tauri 原生集成。Rust 四层架构（Domain/Application/Infrastructure/Presentation）确保业务逻辑与 UI 解耦。所有核心依赖均经官方文档和社区验证，置信度 HIGH。

关键风险集中在三个方面：**Tauri v2 ACL 权限模型**（Phase 1 必须正确配置否则所有 IPC 静默失败）、**SQLite 并发写入**（需 WAL 模式 + 读写分离）、**GitHub API 速率预算管理**（Search 30/min 与 Core 5000/h 需分池隔离）。这些陷阱已在 PITFALLS.md 中有详细预防策略，roadmap 阶段划分时必须显式分配对应验证任务。

## Key Findings

### Recommended Stack

见 [STACK.md](./STACK.md) 获取完整依赖列表和版本号。核心原则：**SQLite 嵌入式场景用 rusqlite，不要过度设计**。

**核心技术：**
- **Tauri v2 (2.6.x)**: 桌面应用框架 — 官方稳定、插件生态成熟、~4MB 包体、原生 WebView 跨平台
- **SvelteKit 2 + Svelte 5 (5.54.x)**: 前端 UI — Runes 响应式系统、编译产物极小、无需外部状态管理库
- **rusqlite (0.38)**: SQLite 封装 — 桌面嵌入式场景最优解，同步 API 对桌面应用足够，比 diesel/sqlx/SeaORM 更薄更合适
- **octocrab (0.49.5)**: GitHub REST API 客户端 — Rust 生态最成熟，支持 ETag 条件请求、内置分页
- **keyring (3.6.3)**: OS 安全存储 — 跨平台 Keychain/Credential Manager 集成，取代已废弃的 Stronghold
- **reqwest (0.13)**: HTTP 客户端 — 标准库，rustls 默认跨平台一致
- **tokio (1.x)**: 异步运行时 — Tauri 内部已使用，cron 调度器依赖

**关键排除决策：**
- Electron → 包体太大（150MB+ vs 4MB）
- diesel/sqlx/SeaORM → 桌面 SQLite 场景过度设计
- Stronghold → 官方已声明 v3 废弃
- SurrealDB → 域模型不需要图数据库
- Turso（v1）→ 单机不需要远程数据库
- 外部状态管理库 → Svelte 5 Runes 内置跨组件状态

### Expected Features

见 [FEATURES.md](./FEATURES.md) 获取竞品分析和完整功能分类。

**Must have（Table Stakes）：**
- GitHub Token 认证 + OS 安全存储 — 所有功能的基础
- 按语言/时间窗筛选趋势仓库 — 用户期望的最低门槛
- 仓库基本信息展示（stars, forks, description, language）
- 订阅仓库 + 基本事件跟踪 — 核心差异化能力的入口
- Release/Tag 变更通知 — 用户订阅的首要原因
- Home/Today 页面 — 30 秒判断今天该看什么
- 桌面端 + 离线可读 — 项目定位的基本承诺

**Should have（Differentiators — geek taste 的独特价值）：**
- **TopK 产品定义排名**（查询模板 + 过滤器 + 快照序列 + 自定义评分公式）— 非 GitHub Trending 复制品
- **Momentum 评分公式**（T3/T4 模式）— 加权评分 + 版本化，避免"热门=最老"
- **连续快照序列**（每 12h）— 支持趋势对比，竞品无此粒度
- **三线统一 Signal 模型** — TopK + Subscription + Resource Radar 在同一信号流中排序去重
- **可用更新判定规则**（U1-U4）— release > tag > branch digest 优先级去重
- **Resource Radar** — 按语言/框架发现 MCP/Skills/Agent 资源
- **Digest 窗口 + 通知矩阵** — 12h/24h 窗口 + HIGH→即时/MEDIUM→digest/LOW→列表

**Defer to v2+：**
- 全自动 LLM 摘要 — v1 用规则+模板，v2 可选 LLM 增强
- 多源爬取（npm/crates/PyPI）— v1 聚焦 GitHub 单源
- Web-first 门户 — 桌面优先定位
- Org/Topic 级订阅 — v1 只绑定单个 Repository
- Issue/PR 级事件明细 — 偏离"可用更新"核心语义
- 跨设备同步（Turso）— v1 单机模式

### Architecture Approach

见 [ARCHITECTURE.md](./ARCHITECTURE.md) 获取完整架构图和数据流。

**四层 + Tauri 壳架构**：Rust 是权威来源（域模型、状态、业务逻辑在 Rust 侧），SvelteKit 只做呈现，IPC 是唯一桥梁。依赖单向：`SvelteKit UI → runtime_tauri → application → domain + adapters`。

**主要组件：**
1. **domain** — 纯实体、值对象、状态机、规则接口（零外部依赖）
2. **application** — 用例编排、策略、权限检查（依赖 domain + adapters as traits）
3. **github_adapter** — GitHub REST 客户端、响应映射、速率限制（octocrab + reqwest）
4. **persistence_sqlite** — SQLite repository 实现、schema migration（rusqlite + rusqlite_migration）
5. **notification_adapter** — 桌面通知（tauri-plugin-notification）
6. **shared_contracts** — IPC DTO、枚举、JSON Schema（serde + specta/ts-rs）
7. **scheduler** — 后台轮询/调度（tokio::time + tokio-cron-scheduler）
8. **runtime_tauri** — Tauri commands / bootstrap / 状态管理

### Critical Pitfalls

见 [PITFALLS.md](./PITFALLS.md) 获取完整 13 个陷阱及预防策略。

1. **Tauri v2 ACL 权限遗漏** — 必须在 `capabilities/default.json` 中显式声明所有命令和插件权限，否则 IPC 静默失败
2. **SQLite 并发写入死锁** — 启用 WAL 模式 + busy_timeout + 读写分离 + `tokio::sync::Mutex`
3. **GitHub API rate budget 踩踏** — Search（30/min）和 Core（5000/h）分池管理 + ETag 条件请求 + 优先级队列
4. **SvelteKit static-adapter 误用 server 功能** — 必须 `ssr = false`，绝不创建 `.server.js` 文件
5. **阻塞 Tokio 主线程** — 所有耗时 command 必须 `async fn` + CPU 密集型用 `spawn_blocking`

## Implications for Roadmap

基于四份研究文件的综合分析，建议 5 阶段结构：

### Phase 1: 项目脚手架与认证骨架
**Rationale:** 所有后续功能依赖项目骨架和认证。PITFALLS.md 指出 ACL 权限和 SvelteKit SSR 配置是 Phase 1 必须解决的"看起来完成但其实没完成"陷阱。ARCHITECTURE.md 确认 Domain crate 必须先行——所有层都依赖 domain 定义。
**Delivers:** 可运行的空壳应用；GitHub Token 认证流程；OS 安全存储集成
**Addresses:** Table Stakes — GitHub Token 认证 + 安全存储
**Avoids:** 陷阱 1（ACL 权限遗漏）、陷阱 3（SvelteKit SSR 误用）、陷阱 9（插件碎片化）
**Research flag:** ⚠️ 需要验证 specta/ts-rs 类型桥接方案的具体版本兼容性

### Phase 2: 数据层与 TopK 发现引擎
**Rationale:** ARCHITECTURE.md 建议 Persistence + GitHub adapter 并行开发（都实现 domain 的 trait，互不依赖），然后构建 Application 层。FEATURES.md 将 TopK 作为核心差异化功能。PITFALLS.md 标注 SQLite 并发、Search 1000 上限、ETag 不完整、IPC 大数据瓶颈均为本阶段陷阱。
**Delivers:** SQLite schema + migration；GitHub REST 客户端；TopK STARS_DESC/UPDATED_DESC 基础模式；仓库信息展示页面
**Addresses:** Table Stakes — 仓库信息展示、趋势筛选；Differentiators — TopK 排名基础
**Avoids:** 陷阱 2（IPC 大数据瓶颈）、陷阱 4（SQLite 并发）、陷阱 5（Migration 中断）、陷阱 7（Search 1000 上限）
**Research flag:** ⚠️ 需要实测 GitHub Search API 在特定查询模板下的实际结果分布

### Phase 3: 订阅系统与信号模型
**Rationale:** FEATURES.md 将订阅 + Release/Tag 跟踪列为 Table Stakes，三线统一 Signal 模型为核心差异化。ARCHITECTURE.md 建议 Application 层在 adapter 就绪后构建，后台调度在 application service 全部就绪后引入。
**Delivers:** 订阅 CRUD + Release/Tag 同步；Usable Update Rules（U1-U3）；Digest 窗口 + 通知矩阵；Home/Today 页面；Signal 去重与排序
**Addresses:** Table Stakes — 订阅管理、Release/Tag 通知、Home 页面；Differentiators — 三线统一 Signal、可用更新判定、Digest 窗口
**Avoids:** 陷阱 6（Rate budget 踩踏）、陷阱 12（ETag 不完整）、陷阱 13（阻塞主线程）、陷阱 8（async Mutex 死锁）
**Research flag:** ⚠️ 需要验证通知矩阵的默认阈值设定

### Phase 4: Agent 资源雷达
**Rationale:** FEATURES.md 将 Resource Radar 列为差异化功能（围绕用户技术栈发现 MCP/Skills/Agent 资源）。依赖 Phase 2 的 TopK 候选集查询机制和 Phase 3 的 Signal 模型。
**Delivers:** Resource Radar 查询模板（MCP_SERVER/SKILL_PACK/AGENT_FRAMEWORK）；分类规则；Resource Score；RESOURCE_EMERGED 信号
**Addresses:** Differentiators — Resource Radar、按语言/框架关联推荐
**Avoids:** AP-3（把资源雷达做成 AI 工具导航站 — 需限定 scope）
**Research flag:** ⚠️ 需要深入研究 MCP/Skills 分类的实际可操作性边界

### Phase 5: 打磨与发布准备
**Rationale:** ARCHITECTURE.md 将离线降级、错误处理、性能优化列为最后阶段。PITFALLS.md 强调 macOS 公证/Windows 签名需尽早配置（Phase 1-2 的 CI/CD），但完整验证在发布前。
**Delivers:** 离线降级模式；错误处理 + 重试策略；性能优化（WAL、虚拟滚动、分页）；TopK MOMENTUM 模式 + 连续快照；安静时段；代码签名 + 自动更新
**Addresses:** Differentiators — Momentum 评分、连续快照、算法变更纪律、冷启动引导
**Avoids:** 陷阱 10（macOS 公证）、陷阱 11（WebView 平台差异）
**Research flag:** ⚠️ macOS 公证流程需在 Phase 1-2 的 CI/CD 中提前配置验证

### Phase Ordering Rationale

- **Domain 先行，adapter 并行**：所有层依赖 domain 定义（ARCHITECTURE.md Build Order 建议）
- **TopK 在订阅之前**：TopK 是核心差异化，且不依赖订阅系统；订阅需要更多 GitHub API 端点
- **订阅在 Resource Radar 之前**：Resource Radar 复用订阅系统的 Signal 模型和调度基础设施
- **打磨最后**：离线/错误处理/性能优化需要完整功能集才能有效验证
- **CI/CD 签名从 Phase 1 开始**：PITFALLS.md 明确指出代码签名不能等到发布前才处理

### Research Flags

Phases 需要 deeper research：
- **Phase 1:** specta/ts-rs 类型桥接方案版本兼容性（ARCHITECTURE.md 标注 MEDIUM confidence）
- **Phase 2:** GitHub Search API 实际结果分布和排序稳定性
- **Phase 4:** MCP/Skills 分类的实际可操作性边界

Phases 使用标准模式（skip research-phase）：
- **Phase 1 脚手架:** Tauri v2 + SvelteKit 集成有官方文档，标准模式
- **Phase 3 订阅系统:** CRUD + 轮询 + 差分是成熟模式
- **Phase 5 打磨:** 性能优化和错误处理是通用模式

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | 全部核心依赖经官方文档、crates.io、Context7 验证；版本号已确认 |
| Features | HIGH | 8 个竞品公开可查，功能可观测；竞品空白领域分析可靠 |
| Architecture | HIGH | Tauri v2 官方文档明确；四层架构模式成熟；type-safe bridge 为 MEDIUM |
| Pitfalls | HIGH | 基于官方文档、社区 issue、真实项目经验；13 个陷阱均有具体预防策略 |

**Overall confidence:** HIGH

### Gaps to Address

- **specta/ts-rs 类型桥接**：社区使用中但具体版本兼容性需 Phase 1 实测 → 在 Phase 1 脚手架阶段用 2 天 spike 验证
- **GitHub Search API 排序稳定性**：研究指出结果不完全确定性 → Phase 2 设计时接受此限制，通过快照对比而非绝对排名
- **2000+ 订阅扩展性**：ARCHITECTURE.md 标注 LOW confidence → v1 假设 100-500 订阅规模，扩展性留待 v1.1 验证
- **tokio-cron-scheduler 在 Tauri 桌面应用中的实际表现**：无大规模案例验证 → Phase 3 优先用 `tokio::time::interval`，cron 作为增强引入

## Sources

### Primary (HIGH confidence)
- Tauri v2 官方文档 — 框架 API、State Management、IPC、SvelteKit 集成、插件系统
- crates.io — rusqlite 0.38、octocrab 0.49.5、keyring 3.6.3、reqwest 0.13.2 版本确认
- Context7 — Tauri v2 和 SvelteKit 文档查询验证
- GitHub API 官方文档 — rate limit、条件请求、Search API 限制

### Secondary (HIGH confidence)
- Rust ORMs 2026 对比文章 — rusqlite 在桌面嵌入式 SQLite 场景的一致推荐
- keyring-rs GitHub + Tauri v2 示例项目 — 跨平台安全存储验证
- octocrab CHANGELOG — v0.49.3+ ETag 支持确认
- 多个 Tauri v2 迁移陷阱回顾文章 — ACL、IPC 性能、插件碎片化

### Tertiary (MEDIUM confidence)
- 竞品分析基于公开可查的项目页面和 README（GitLight、StarGazer、Astral 等）
- MCP/Skills 生态分析基于 GitHub MCP Registry 公开信息

---

*Research completed: 2026-03-22*
*Ready for roadmap: yes*
