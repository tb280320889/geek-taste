# Phase 1: 项目脚手架与认证 - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 1 delivers the authentication foundation and navigation shell for geek taste. Users can launch the app, authenticate with GitHub PAT, see basic repo info, configure settings, and navigate between 5 pages. This phase establishes the UI skeleton that all subsequent phases build upon.

</domain>

<decisions>
## Implementation Decisions

### 认证交互方式
- PAT 输入入口为首次启动时的 Onboarding 全屏流程 — 用户必须先认证才能进入主界面
- 输入后立即调用 `GET /user` 验证 token — 失败时显示具体错误信息（401/403/网络）+ 重试按钮，不存储无效 token
- Token 过期/撤销后：应用启动时后台验证 → 失败时弹出重新认证对话框，阻断 API 调用

### 导航架构
- 左侧固定 Sidebar — 图标+文字，折叠后仅图标；适合 5 个一级页面
- SvelteKit 文件路由：`/` (Home), `/topk`, `/subscriptions`, `/resources`, `/rules`
- layout 级别认证状态守卫 → 未认证时重定向到 `/onboarding`
- 未认证时完整导航壳可见，内容区域显示"请先连接 GitHub"提示卡

### 设置系统
- 设置存储为本地 JSON 文件，通过 `tauri-plugin-store`（已配置）
- 独立 Settings 页面（Sidebar 底部齿轮图标）包含所有配置
- 语言兴趣：从用户已 star 仓库自动推断 Top 5 + 手动调整复选框
- 安静时段：时间范围选择器（开始-结束时间）+ 开关，默认关闭

### 仓库信息展示
- 认证成功后跳转 Home 页面，显示用户 GitHub 头像 + 用户名欢迎
- TopK 页面顶部提供"探索"输入框 — 输入 repo URL/名称后弹出详情卡
- 信息卡展示 5 个字段：Stars ⭐, Forks 🔀, Description, Language（彩色标签）, Topics（标签组）
- 详情卡为弹出式 Modal，背景半透明遮罩

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `apps/desktop-ui/src/` — SvelteKit 基础结构已就绪（SSR 禁用，SPA 模式）
- `apps/desktop-ui/src-tauri/src/lib.rs` — Tauri 插件已配置：shell, dialog, store, notification
- `apps/desktop-ui/src-tauri/tauri.conf.json` — Tauri v2 配置完整（com.geek-taste.app, 1200x800）
- `crates/runtime_tauri/Cargo.toml` — 依赖配置完整（domain, application, github_adapter 等）
- `keyring 3.6` — 已在 workspace dependencies 配置（apple-native, windows-native）

### Established Patterns
- Hexagonal Architecture：domain → application → adapters → runtime
- SvelteKit static-adapter SPA 模式（ssr=false, prerender=false）
- Moon monorepo 构建：apps/ + crates/ 分层

### Integration Points
- `crates/runtime_tauri/` — 新增 Tauri commands（auth, settings, repo info）
- `crates/domain/` — 新增 AuthToken, User, Settings 领域模型
- `crates/shared_contracts/` — 新增认证/设置 DTO
- `apps/desktop-ui/src/routes/` — 新增 onboarding, topk, subscriptions, resources, rules 页面
- `apps/desktop-ui/src/lib/` — 新增 Sidebar 组件、Auth store、API 调用层

</code_context>

<specifics>
## Specific Ideas

- 首次 Onboarding 流程类似 Raycast 的初始设置体验 — 简洁、步骤清晰
- Sidebar 参考 Linear/Vercel Dashboard 风格 — 紧凑、图标优先、暗色适配
- 认证验证使用 octocrab（已在 github_adapter 依赖中）直接调用 `GET /user`

</specifics>

<deferred>
## Deferred Ideas

- Linux keyring 支持（需添加 `sync-secret-service` 特性）— v1 仅 macOS + Windows
- OAuth flow 替代 PAT — v1 使用 PAT 更简单直接
- 设置同步到云端 — v1 本地存储为主

</deferred>
