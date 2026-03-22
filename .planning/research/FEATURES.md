# Feature Landscape — geek taste

**Domain:** Desktop GitHub 技术雷达工作台
**Researched:** 2026-03-22
**Overall confidence:** HIGH（多数竞品公开可查，功能可观测）

---

## 1. 竞品生态总览

### 1.1 竞品分类矩阵

| 类别 | 代表工具 | 核心定位 | 活跃度 |
|------|----------|----------|--------|
| **GitHub Trending** | github.com/trending | GitHub 官方，按日/周/语言展示 star 增速最快的仓库 | 官方维护，但功能极简 |
| **Star 追踪/可视化** | star-history.com, daily-stars-explorer, StarGazer | 单仓库/多仓库 star 历史曲线、增长率 | 活跃，但功能单一 |
| **Star 管理/分类** | Astral, StarGazer (星眸) | 给已 star 仓库打标签、分类管理 | Astral 低维护；StarGazer 新但功能全 |
| **通知收件箱** | Gitify, GitLight, Octobox, DevHub | 管理 GitHub notifications（issue/PR/review） | 大部分活跃 |
| **趋势发现** | LibHunt, RepoHunt, BestGitHub, octotrends, RepositoryStats | 语义搜索、趋势排序、增长率分析 | 活跃 |
| **Agent/MCP 资源** | GitHub MCP Registry, MCP Atlas, MCPfinder, MCP Discovery | MCP Server 发现与注册 | 新兴领域，快速发展 |
| **Dev Tools 发现** | DevHunt, DevToolHunt, Product Hunt | 社区驱动的工具发布与投票 | 活跃 |
| **邮件/SaaS 摘要** | Gitmore, GitDailies, Radar, PullNotifier | AI 摘要 + Slack/邮件推送 | 商业化产品 |

### 1.2 关键竞品深度分析

#### GitHub Trending（github.com/trending）
- **做什么好：** 官方权威数据源，无需认证，按日/语言筛选，覆盖全量仓库
- **做什么差：** 无订阅管理、无自定义排序公式、无通知、无历史快照、无跨时间对比、无过滤器保存、不支持 topic/框架级筛选、无 API 供外部消费、无桌面端
- **geek taste 差异化：** TopK 是产品定义的排名系统（非 GitHub Trending 复制品），支持保存视图、连续快照、momentum 计算、自定义过滤器

#### star-history.com
- **做什么好：** star 历史可视化、多仓库对比、embed 图表、Chrome 插件、log scale
- **做什么差：** 纯可视化工具，无订阅、无通知、无趋势排名、无分类管理、40k star 上限（REST API 限制）、不跟踪 fork/release/tag
- **geek taste 差异化：** 从"看图"到"做决策"——TopK 榜单 + 订阅信号 + Agent 资源雷达，三线统一为 Signal

#### Astral（astralapp.com）
- **做什么好：** tag 分类、Smart Filter（规则过滤器）、README 预览、notes 备注
- **做什么差：** 只管理已 star 的仓库、无趋势发现、无更新通知、PHP 后端老旧、维护缓慢（最后 commit 2025-05）、不跟踪 release/tag、无桌面端、无离线
- **geek taste 差异化：** Astral 是"星标整理柜"，geek taste 是"技术雷达工作台"——从分类管理升级到主动信号发现

#### GitLight（gitlight.app）
- **做什么好：** Tauri + SvelteKit 桌面端（与 geek taste 相同技术栈）、GitHub/GitLab 双源、Kanban 式通知界面、按 repo/org 过滤
- **做什么差：** 纯通知查看器（不聚合 digest）、无趋势发现、无订阅管理、无 Agent 资源、无自定义排名、project 停更（最后 commit 2024-12）、不支持 release/tag digest 级通知
- **geek taste 差异化：** GitLight 管理"已发生的通知"，geek taste 生成"可行动的信号"——有优先级、有去重、有 digest 窗口

#### StarGazer（星眸）
- **做什么好：** star 同步与管理、标签/别名/备注、多渠道推送（Bark/Gotify/Webhook）、AI 摘要 README、i18n、响应式
- **做什么差：** 单用户设计、只管理已有 star、无趋势发现、无 TopK 排名、无 Agent 资源雷达、Python 后端（非桌面端）
- **geek taste 差异化：** StarGazer 是"收藏夹增强版"，geek taste 是"信号驱动的发现引擎"——从被动管理到主动推荐

#### LibHunt / RepositoryStats / octotrends
- **做什么好：** 跨仓库增长趋势（百分比/绝对值）、多时间窗口（30d/180d/365d）、按语言/topic 聚合
- **做什么差：** Web-only、无个性化、无订阅、无通知、无自定义排名公式、无本地缓存、数据更新频率不透明
- **geek taste 差异化：** TopK 将趋势发现从"全球热榜"变成"我的雷达"——按用户兴趣栈、保存视图、连续快照、momentum 公式版本化

#### GitHub MCP Registry / MCP Atlas / MCPfinder
- **做什么好：** MCP Server 发现与注册、分类目录、质量评分、语义搜索
- **做什么差：** 独立于 GitHub 趋势系统、无语言/框架相关性排序、无与用户已订阅仓库的关联、无桌面端、无定时刷新
- **geek taste 差异化：** Resource Radar 不是另一个 MCP 目录，而是"围绕我的技术栈发现相关 Agent 资源"——与 TopK/订阅统一在一个工作台中

---

## 2. 功能分类

### 2.1 Table Stakes（必须有，否则用户流失）

| 功能 | 为什么是必需 | 复杂度 | 竞品参考 |
|------|-------------|--------|----------|
| **GitHub Token 认证** | 所有功能依赖 API 调用，无 token 无数据 | 低 | 全部竞品 |
| **按语言/时间窗筛选趋势仓库** | GitHub Trending 的核心价值，用户期望最低门槛 | 中 | GitHub Trending, LibHunt |
| **仓库基本信息展示**（stars, forks, description, language） | 决策判断的基础数据 | 低 | 全部竞品 |
| **订阅仓库 + 基本事件跟踪** | 核心差异化能力的入口，无订阅则 Home 无内容 | 中 | StarGazer, Gitify |
| **Release/Tag 变更通知** | 用户订阅仓库的首要原因——"它发新版本了吗" | 中 | GitHub Notifications, Gitify |
| **Home/Today 页面** | 冷启动后第一个到达的页面，30 秒判断今天该看什么 | 中 | 无直接竞品 |
| **桌面端 + 离线可读** | 项目定位为桌面优先应用，离线是用户预期 | 高 | GitLight（Tauri） |
| **安全存储 Token** | 桌面应用处理 PAT 的基本安全要求 | 低 | 系统 keychain |

### 2.2 Differentiators（竞争优势——geek taste 的独特价值）

| 功能 | 竞争优势 | 复杂度 | 竞品对比 |
|------|----------|--------|----------|
| **TopK 产品定义排名（非 GitHub Trending）** | 由查询模板 + 过滤器 + 快照序列 + 自定义评分公式构成，用户可以保存多个排名视图。竞品要么是 GitHub Trending 的简单复制，要么是单纯的 star 可视化 | 高 | GitHub Trending 只有固定榜单；LibHunt 不支持自定义 |
| **Momentum 评分公式（T3/T4 模式）** | 0.5*star_delta + 0.2*fork_delta + 0.3*recency 的加权评分，版本化，避免竞品中"热门=最老"的问题 | 中 | octotrends 用百分比增长，无加权；无版本化概念 |
| **连续快照序列** | 每 12h 自动快照，支持 momentum 计算和趋势对比。竞品无此粒度 | 高 | star-history.com 按需查询，无自动化快照 |
| **三线统一 Signal 模型** | TopK + Subscription + Resource Radar 统一输出为可解释、可去重、可排序的 Signal。竞品只覆盖其中一条线 | 高 | 无竞品做到三线统一 |
| **可用更新判定规则（U1-U4）** | 不是全量通知，而是按 release > tag > branch digest 的优先级去重，一个 digest 窗口内同一 repo 最多 1 条主信号 | 中 | GitHub Notifications 全量推送；Gitify 无去重逻辑 |
| **Resource Radar（按语言/框架发现 Agent 资源）** | 围绕用户技术栈发现 MCP/Skills/Agent 资源，与 TopK/订阅在同一工作台中 | 高 | MCP Registry 是独立目录；无与趋势/订阅的关联 |
| **Digest 窗口 + 去噪矩阵** | 12h/24h digest 窗口 + 默认通知矩阵（HIGH→桌面即时，MEDIUM→digest，LOW→仅列表），安静时段支持 | 中 | Gitmore/GitDailies 做 digest，但只覆盖 PR 活动；无优先级矩阵 |
| **算法变更纪律** | 所有评分公式、优先级映射、显著变化阈值版本化，禁止 silent change。用户可以预期结果 | 低 | 无竞品公开版本化策略 |
| **冷启动引导** | TopK 在无快照时降级为 UPDATED_DESC 并提示"暖机中"；订阅首次同步只建立 baseline 而非回溯全部历史 | 中 | 无竞品关注冷启动体验 |

### 2.3 Anti-Features（明确不做的功能）

| 不做 | 原因 | 替代方案 |
|------|------|----------|
| **GitHub 全站搜索替代品** | 与 GitHub 本体竞争，范围失控 | 使用 GitHub Search API 作为 TopK 候选集来源 |
| **Issues/PR/Discussions 全功能收件箱** | 范围过大，偏离信号聚焦。Octobox/Gitify/DevHub 已经做得很好 | 只跟踪 PR_MERGED_DIGEST（可选高级模式），不做逐条 PR 管理 |
| **秒级实时告警** | GitHub Events API 延迟 30s-6h，承诺实时会制造虚假期望 | 轮询 + 差分 + digest 窗口，承诺"下一个 digest 周期内"可见 |
| **用户间社交互动/协作** | v1 做个人工具，不做社区 | 社区层面留给 GitHub 本体 |
| **全自动 LLM 摘要作为核心路径** | 成本/稳定性/延迟风险高，v1 应该先用确定性方案验证 | v1 用规则 + 模板摘要；v2 可选 LLM 增强 |
| **多源爬取（npm/crates/PyPI 等）** | 数据源复杂度爆炸，v1 聚焦 GitHub 单源 | v1 仅 GitHub API；后续根据需求扩展 |
| **Web-first 门户** | 桌面优先定位；Web 需要额外的 auth/安全考量 | SvelteKit static-adapter SPA 在桌面内渲染 |
| **Org 级/Topic 级订阅** | v1 订阅对象只绑定单个 Repository | 减少订阅模型复杂度 |
| **Issue/PR 级事件明细** | 偏离"可用更新"（release/tag/branch）的核心语义 | 只在高级模式下做 PR_MERGED_DIGEST |
| **嵌入式 README/AI 总结作为核心功能** | StarGazer 做了 AI 总结 README，但这不是 geek taste 的核心差异 | v1 可以做 README 预览但不做 AI 总结 |

---

## 3. 功能依赖图

```
GitHub Token Auth
  ├── TopK Ranking
  │     ├── Candidate Query (GitHub Search API)
  │     ├── Snapshot Engine (每 12h 快照)
  │     ├── RankingMode (STARS_DESC / UPDATED_DESC / MOMENTUM_24H / MOMENTUM_7D)
  │     └── Saved RankingViews → Home 信号
  ├── Subscription
  │     ├── Repo Sync Scheduler (轮询 Releases/Tags/Events)
  │     ├── Usable Update Rules (U1-U4)
  │     ├── Digest Window (12h/24h)
  │     └── Signals → Home + Desktop Notification
  ├── Resource Radar
  │     ├── Query Templates (MCP_SERVER / SKILL_PACK / AGENT_FRAMEWORK / ...)
  │     ├── Classification Rules (curated > topics > template context > fallback)
  │     ├── Resource Score (stack_relevance + star_delta + recency + curation)
  │     └── Signals → Home
  └── Home/Today
        ├── Unified Priority Sort (priority + recency + source_type + user_affinity)
        ├── Read/Processed State
        └── Quiet Hours 配置
```

---

## 4. MVP 排序建议

### Phase 1 — 基础骨架
1. GitHub Token Auth + 安全存储
2. 基础仓库信息展示
3. TopK STARS_DESC / UPDATED_DESC（无快照依赖的简单模式）

### Phase 2 — 发现引擎
4. TopK Snapshot Engine + MOMENTUM 模式
5. 候选集过滤器（language/topic/stars 阈值）
6. Saved RankingViews

### Phase 3 — 订阅与信号
7. 订阅仓库 + Release/Tag 同步
8. Usable Update Rules (U1-U3)
9. Digest 窗口 + 通知矩阵

### Phase 4 — Agent 资源雷达
10. Resource Radar 查询模板 + 分类规则
11. Resource Score + RESOURCE_EMERGED 信号

### Phase 5 — 打磨
12. Home/Today 统一排序 + 已读/已处理状态
13. 安静时段
14. TopK VIEW_CHANGED 信号
15. PR_MERGED_DIGEST 高级模式

---

## 5. 竞品未覆盖的空白领域（geek taste 的机会）

| 空白 | 说明 |
|------|------|
| **"我的技术栈"趋势发现** | 现有工具要么是全局热榜（GitHub Trending），要么是个人收藏管理（Astral）。没有人做"基于我的兴趣栈 + 自定义排名公式"的趋势发现 |
| **三线统一** | TopK 发现 + 订阅跟踪 + Agent 资源雷达，三者在同一个 Signal 模型中排序和去重。任何竞品只覆盖其中一条线 |
| **桌面端技术雷达** | 所有趋势发现工具都是 Web-only。没有一个桌面应用做"我的技术雷达工作台"。GitLight 是桌面端但只做通知 |
| **MCP/Skills 资源与语言/框架关联** | MCP Registry 是"全局目录"，没有人做"我用 Rust + Tauri，所以这些 MCP/Skills 与我相关"的关联推荐 |
| **版本化排名公式** | 竞品的排名逻辑是黑盒。geek taste 公开版本化评分公式，用户可以预期和信任结果 |
| **可用更新判定（而非全量通知）** | 从"推给你一切"到"告诉你什么值得处理"——release > tag > branch digest 的优先级去重是竞品缺失的能力 |

---

## 6. Sources

- GitHub Trending: github.com/trending（官方页面）
- star-history.com: newsletter.star-history.com/p/star-history-com-in-2025（2026-01-07 年度总结）
- Astral: github.com/astralapp/astral, astralapp.com（最后 push 2025-05）
- GitLight: github.com/colinlienard/gitlight, gitlight.app（最后 push 2024-12-26，停更标记）
- StarGazer: github.com/xy2yp/StarGazer（2025-08 创建，功能全面的星标管理工具）
- LibHunt: libhunt.com（趋势项目发现）
- RepositoryStats: repositorystats.com（仓库趋势分析）
- octotrends: github.com/doda/octotrends（增长率分析）
- BestGitHub: bestgithub.com（仓库健康评分 + 排名）
- RepoHunt: repohunt.dev（语义搜索仓库）
- GitHub MCP Registry: github.blog/changelog/2025-09-16-github-mcp-registry（2025-09 发布）
- MCP Atlas: github.com/mcevoyinit/mcp-atlas（2100+ MCP Server 目录）
- MCPfinder: github.com/mcpfinder/server（MCP 动态发现）
- DevHunt: devhunt.org（开发者工具社区发布平台）
- Gitmore: gitmore.io/blog/github-activity-digest-notification-tools（2026-03 对比评测）
- Gitify: 桌面菜单栏 GitHub 通知（开源）
- Octobox: 通知收件箱（开源，GitHub 通知管理先驱）
- Radar: radar.town（GitHub → Slack 通知，含 digest 模式）
