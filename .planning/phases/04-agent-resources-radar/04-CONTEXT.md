# Phase 04 Context: Agent 资源雷达

## 用户愿景

用户希望通过 geek taste 发现与自己技术栈相关的 **MCP / Skills / Agent 生产力资源**，而不是手动在 GitHub 上大海捞针。

核心体验：打开 Resources 页 → 看到按相关度排序的资源列表 → 每条有"为什么推荐给我"的解释 → 一键关注/订阅。

## 数据来源策略

**混合模式：用户精选列表 + GitHub Search 扩展**

1. **用户自定义精选列表** — 应用内提供 UI 让用户手动添加/管理感兴趣的资源仓库
2. **GitHub Search 扩展** — 基于用户技术栈画像，用 topic/关键词搜索发现新资源
3. **交叉验证** — 搜索结果与精选列表合并去重，精选列表中的资源获得 relevance 加分

数据流：
```
用户精选 repos ──→ ┐
                    ├─→ 合并去重 → 评分排序 → 展示
GitHub Search     ──→ ┘
```

## 分类体系

**扁平 + 标签** — 不预设分类树

- 每个资源通过 tags 标注（自由标签）
- 提供推荐标签列表引导用户（如 `mcp`, `skill`, `agent`, `database`, `web`, `test`, `code-review`, `deploy`）
- 用户按标签筛选，不强制分类

## 评分模型

`score = w1 * stack_relevance + w2 * star_delta + w3 * recency`

### stack_relevance 计算

用户技术栈画像来源（按优先级）：
1. **用户兴趣设置** — Settings 中选择的语言/框架兴趣（优先）
2. **订阅仓库推断** — 基于用户已订阅仓库的语言分布推断

两者的加权组合决定 stack_relevance 分数。

### star_delta
近期 star 增长趋势（复用 Phase 2 MomentumScore 逻辑）。

### recency
最近一次 commit/release 的时间衰减。

## 推荐解释

**模板 + LLM fallback**

- **模板规则**（覆盖 80%+ 场景）：
  - "你关注 {lang}，这个资源是 {lang} 生态的 MCP server"
  - "你订阅了 {N} 个 {framework} 相关仓库，这个 skill 适用于 {framework}"
  - "近期 star 增长 {delta}，社区关注度上升"
  - "你已将此资源加入精选列表"
- **LLM fallback**（边缘 case）：
  - 模板无法合理解释时（如跨栈推荐、冷门资源），调用 LLM 生成自然语言解释
  - v1 中可标记为 TODO，先以模板为主

## 关键决策

- **数据源**: 混合模式 — 用户精选 + GitHub Search，避免单一来源偏差
- **分类**: 扁平标签 — 灵活，不预设分类树，与产品"信号优先"理念一致
- **stack_relevance**: 双源画像（兴趣设置 + 订阅推断），设置优先
- **推荐解释**: 模板为主，v1 可先不接 LLM，用规则生成确定性解释
- **标签体系**: 自由标签 + 推荐列表引导，不强制

## 关键约束

- v1 数据源仅限 GitHub（同 PROJECT.md 约束）
- Search API 速率 30 req/min — 资源发现必须做端点级限流
- 精选列表存在本地 SQLite，不依赖云端
- 标签不强校验，允许用户自由输入

## 待研究

- [ ] MCP registry API 是否可用作额外数据源（ROADMAP 研究标记）
- [ ] GitHub topic 覆盖度：`topic:skill`, `topic:mcp` 等搜索质量
- [ ] 标签推荐列表的初始词库从哪来
- [ ] LLM fallback 的接入时机与成本预算
