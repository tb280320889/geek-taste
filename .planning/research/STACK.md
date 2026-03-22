# Technology Stack — geek taste

**项目:** geek taste — 跨端技术雷达工作台  
**研究日期:** 2026-03-22  
**整体置信度:** HIGH

---

## 核心框架

### Tauri v2（桌面壳）

| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `tauri` | 2.6.x | 桌面应用框架 | v2 已于 2024-09 稳定发布，当前 2.6.1；插件生态成熟；跨 macOS + Windows 原生 WebView，包体 ~4MB 远小于 Electron |
| `tauri-build` | 2.x | 构建工具链 | 与 tauri 同版本对齐 |

**置信度:** HIGH  
**来源:** [Tauri v2 官方文档](https://v2.tauri.app/)，Context7 文档，crates.io

#### Tauri v2 迁移状态（v1→v2）

- v2 已完全稳定，不再接受 v1 新功能
- v2.6.x 为最新稳定版
- 官方提供 `tauri migrate` 自动迁移工具
- **本项目为 greenfield，无需迁移，直接使用 v2**
- Stronghold 插件将在 v3 被废弃 → **不推荐依赖 Stronghold**

#### Tauri v2 必需插件

| 插件 | 版本 | 用途 |
|------|------|------|
| `tauri-plugin-notification` | 2.x | 桌面通知（digest 推送） |
| `tauri-plugin-shell` | 2.x | 打开外部 URL（仓库链接等） |
| `tauri-plugin-dialog` | 2.x | 文件选择/确认对话框 |
| `tauri-plugin-store` | 2.x | 轻量 KV 持久化（非敏感配置） |
| `tauri-plugin-updater` | 2.x | 自动更新 |
| `tauri-plugin-window-state` | 2.x | 窗口位置/大小记忆 |

**置信度:** HIGH  
**来源:** [Tauri 插件仓库](https://github.com/tauri-apps/plugins-workspace)，v2.tauri.app

---

### SvelteKit + Svelte 5（前端 UI）

| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `svelte` | 5.54.x | UI 编译器 | Runes 响应式系统；编译产物极小（~1.6KB gzipped）；明确优于隐式响应式 |
| `sveltekit` | 2.50.x | 应用框架 | 路由、load 函数、TypeScript 集成 |
| `@sveltejs/adapter-static` | latest | SPA 模式适配器 | Tauri 官方推荐；`fallback: 'index.html'` 实现 SPA 路由 |
| `@sveltejs/vite-plugin-svelte` | latest | Vite 集成 | `vitePreprocess()` 用于 TypeScript/SCSS |

**置信度:** HIGH  
**来源:** [Tauri + SvelteKit 集成指南](https://v2.tauri.app/start/frontend/sveltekit/)，明确标注适用于 SvelteKit 2.20.4 / Svelte 5.25.8

#### SPA 模式配置

```typescript
// svelte.config.js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({ fallback: 'index.html' })
  }
};

export default config;
```

```typescript
// src/routes/+layout.ts
export const ssr = false;
```

#### Svelte 5 Runes 说明

- `$state()` 替代 `let` 声明式响应式
- `$derived()` 替代 `$:` 响应式声明
- `$effect()` 替代 `onMount`/`afterUpdate`
- `$props()` 替代 `export let`
- 响应式可以在 `.svelte.ts` 文件中使用（跨组件状态共享）
- **不需要外部状态管理库**

---

## 数据层

### SQLite — 主数据库

| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `rusqlite` | 0.38.0 | SQLite 封装 | 桌面/嵌入式场景的行业标准；薄封装、零抽象泄漏；完整 SQLite 功能集可直接使用 |
| `rusqlite_migration` | 2.4.1 | Schema 迁移 | 使用 `user_version` 而非额外表，性能最优；内联 SQL 定义简洁；与 rusqlite 版本严格对齐 |
| `serde_rusqlite` | latest | 行→结构体映射 | 减少手动 `.get()` 映射样板代码 |

**置信度:** HIGH  
**来源:** [Rust ORMs 2026 对比文章](https://aarambhdevhub.medium.com/rust-orms-in-2026-diesel-vs-sqlx-vs-seaorm-vs-rusqlite-which-one-should-you-actually-use-706d0fe912f3)，crates.io，Reddit r/rust 讨论

#### 为什么是 rusqlite 而不是 diesel / sqlx

| 方案 | 推荐？ | 原因 |
|------|--------|------|
| **rusqlite** | ✅ 推荐 | SQLite 专用；同步 API 对桌面应用足够；编译快、抽象薄、调试友好 |
| diesel | ❌ | 编译时 SQL DSL 增加学习成本和编译时间；异步支持是外挂的；对于桌面嵌入式 SQLite 场景过度设计 |
| sqlx | ❌ | 异步开销在桌面应用中无必要；编译时 SQL 检查需要连接数据库；query builder 偏向 Web 后端场景 |
| SeaORM | ❌ | ActiveRecord 模式不适合领域驱动的四层架构；异步 ORM 对桌面应用无收益 |

**结论:** "Is it SQLite? → Rusqlite. Done. Don't overthink it."（原文引用自 2026 年对比文章）

#### 如果需要异步 SQLite 连接池

Tauri 的 IPC 本身运行在异步上下文中。如果某些操作需要避免阻塞，可以使用：

| 技术 | 用途 |
|------|------|
| `deadpool-sqlite` | 将 rusqlite 包装为异步连接池，通过 `spawn_blocking` 实现 |

**但 v1 建议直接在 Tauri command 中同步调用 rusqlite**，Tauri 的 command 系统在独立线程运行，不会阻塞 UI。

---

### 为什么不用 Turso / libSQL

- v1 单机模式不需要远程数据库
- Turso 的嵌入式副本（embedded replicas）需要 write-to-cloud 路径，增加架构复杂度
- 当出现跨设备同步需求时（v1.1+）再引入
- **不提前引入，不做过度准备**

---

## HTTP 与 GitHub API

### HTTP 客户端

| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `reqwest` | 0.13.x | HTTP 客户端 | Rust 生态标准；默认 rustls（跨平台一致）；JSON 序列化/反序列化内置；tokio 异步原生 |
| `tokio` | 1.x | 异步运行时 | Rust 异步事实标准；Tauri 内部已使用 tokio；cron 调度器依赖 |

**置信度:** HIGH  
**来源:** [reqwest crates.io](https://crates.io/crates/reqwest)（v0.13.2，2026-02-06 发布）

### GitHub API 客户端

| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `octocrab` | 0.49.5 | GitHub REST API 客户端 | Rust 生态最成熟的 GitHub 客户端；强类型语义 API + 底层 HTTP 扩展；支持 ETag 条件请求（v0.49.3+）；分页处理内置 |

**置信度:** HIGH  
**来源:** [octocrab CHANGELOG](https://github.com/XAMPPRocky/octocrab/blob/master/CHANGELOG.md)，[docs.rs](https://docs.rs/octocrab)

#### octocrab 关键能力

- **搜索 API**: `octocrab.search().repositories(q).send().await?`
- **Releases**: `octocrab.repos(owner, repo).releases().list().send().await?`
- **Tags**: `octocrab.repos(owner, repo).list_tags().send().await?`
- **Events**: `octocrab.repos(owner, repo).list_events().send().await?`
- **条件请求**: v0.49.3+ 支持 HTTP 缓存与 ETag
- **分页**: `Page<T>` 结构体带 `next()` 链接
- **速率限制**: 内置 `ratelimit` API

#### REST-first 策略确认

octocrab 同时支持 REST 和 GraphQL（`octocrab.graphql(query, variables).await?`），但 v1 仅使用 REST 端点。

---

## 安全存储（Token）

| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `keyring` | 3.6.3（稳定） | OS 安全存储 | 跨平台 Keychain/Credential Manager/Secret Service 集成；Tauri 社区首选方案 |

**置信度:** HIGH  
**来源:** [keyring crates.io](https://crates.io/crates/keyring)，[keyring-rs GitHub](https://github.com/open-source-cooperative/keyring-rs)，Reddit 社区讨论

#### 特性配置

```toml
[dependencies]
keyring = { version = "3", features = ["apple-native", "windows-native", "sync-secret-service"] }
```

- `apple-native`: macOS Keychain / iOS Keychain
- `windows-native`: Windows Credential Manager
- `sync-secret-service`: Linux D-Bus Secret Service（同步版本，兼容性更好）

#### 为什么不是 Stronghold

| 方案 | 推荐？ | 原因 |
|------|--------|------|
| **keyring** | ✅ 推荐 | 直接使用 OS 原生安全存储；跨平台已验证；Tauri 官方维护者已明确推荐 |
| tauri-plugin-stronghold | ❌ | 官方已声明 v3 将废弃；需要用户输入密码或额外存储加密密钥；社区反馈调试困难 |

**关键引用:** "stronghold is no longer recommended and will be deprecated and therefore removed in v3" — Tauri 维护者 FabianLars

---

## 调度与定时

| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `tokio::time` | 1.x（内置） | 简单轮询定时器 | 对固定间隔轮询（如每 12h 同步）足够简单可靠 |
| `tokio-cron-scheduler` | 0.15.1 | Cron 表达式调度 | 300 万下载量；成熟稳定；支持 tokio 异步；适合需要灵活 cron 调度的场景 |
| `chrono` | 0.4.x | 时间处理 | Rust 时间标准库；tokio-cron-scheduler 和 octocrab 均依赖 |

**置信度:** HIGH  

#### 调度策略建议

- **简单轮询**（每 N 小时同步订阅）：用 `tokio::time::interval`
- **复杂调度**（按 cron 表达式触发 TopK 快照/摘要生成）：用 `tokio-cron-scheduler`
- v1 大多数场景用 `tokio::time::interval` 就够了，`tokio-cron-scheduler` 作为增强引入

---

## 错误处理与序列化

| 技术 | 版本 | 用途 | 选择理由 |
|------|------|------|---------|
| `serde` | 1.x | 序列化/反序列化 | Rust 生态标准；JSON/TOML/结构体转换；octocrab/reqwest/rusqlite 均依赖 |
| `serde_json` | 1.x | JSON 处理 | GitHub API 响应解析；IPC 数据传输 |
| `anyhow` | 1.x | 应用层错误 | Application Layer 的错误包装；简化错误传播 |
| `thiserror` | 2.x | 领域层错误 | Domain Layer 的类型化错误定义；derive 宏简化 |
| `tracing` | 0.1.x | 结构化日志 | Rust 异步日志标准；替代 log crate；支持 span 和 structured fields |
| `tracing-subscriber` | 0.3.x | 日志输出配置 | 控制台/文件输出 |

**置信度:** HIGH

#### 错误分层策略

```
Domain Layer  → thiserror 定义领域错误类型
Application Layer  → anyhow::Result + context()
Infrastructure Layer  → 各 crate 原生错误 + impl From 转换
IPC 边界  → 错误序列化为 JSON 传给前端
```

---

## 完整依赖安装

### Cargo.toml 核心依赖

```toml
[dependencies]
# Tauri
tauri = { version = "2", features = [] }
tauri-build = "2"

# Tauri 插件
tauri-plugin-notification = "2"
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-store = "2"
tauri-plugin-window-state = "2"

# 异步运行时
tokio = { version = "1", features = ["full"] }

# HTTP
reqwest = { version = "0.13", features = ["json", "rustls-tls"] }

# GitHub API
octocrab = "0.49"

# SQLite
rusqlite = { version = "0.38", features = ["bundled"] }
rusqlite_migration = "2.4"
serde_rusqlite = "0.36"

# 安全存储
keyring = { version = "3", features = ["apple-native", "windows-native", "sync-secret-service"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 错误处理
anyhow = "1"
thiserror = "2"

# 时间
chrono = { version = "0.4", features = ["serde"] }

# 调度
tokio-cron-scheduler = "0.15"

# 日志
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### package.json 前端依赖

```json
{
  "devDependencies": {
    "@sveltejs/adapter-static": "latest",
    "@sveltejs/kit": "^2.50",
    "@sveltejs/vite-plugin-svelte": "latest",
    "svelte": "^5.54",
    "svelte-check": "latest",
    "typescript": "^5.x",
    "vite": "^6.x"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-notification": "^2",
    "@tauri-apps/plugin-shell": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-store": "^2",
    "@tauri-apps/plugin-window-state": "^2"
  }
}
```

---

## 不使用的方案及原因

| 被排除方案 | 原因 |
|-----------|------|
| **Electron** | 包体巨大（150MB+ vs Tauri ~4MB）；Node.js 运行时冗余 |
| **Deno/Bun** | Tauri 绑定 Node.js 生态；切换运行时无收益且增加不确定性 |
| **GraphQL（v1）** | REST-first 策略已确定；批量优化留待 v1.1+ |
| **SurrealDB** | 域模型不需图数据库；SQLite 确定性和成熟度更重要 |
| **Turso（v1）** | 单机不需要远程数据库；过度准备 |
| **diesel / sqlx / SeaORM** | 桌面 SQLite 嵌入式场景中 rusqlite 是最优解 |
| **Stronghold** | 官方已宣布 v3 废弃；keyring 直接使用 OS 安全存储更可靠 |
| **GraphQL client（async-graphql 等）** | v1 不使用 GraphQL |
| **Zustand/Pinia/Svelte Stores** | Svelte 5 Runes 提供内置跨组件状态，无需外部库 |
| **Axum（v1）** | v1 不需要 HTTP 服务器；预留至 v1.1 云同步 |

---

## Cargo Workspace 结构（对应架构规范）

```
/geek-taste
  /apps
    /desktop-ui              # SvelteKit + Tauri 前端壳
  /crates
    /domain                  # 纯领域对象与规则（thiserror, serde）
    /application             # 用例编排（anyhow, tokio, chrono）
    /github_adapter          # GitHub REST client（octocrab, reqwest）
    /persistence_sqlite      # SQLite repository（rusqlite, rusqlite_migration, serde_rusqlite）
    /notification_adapter    # 桌面通知（tauri-plugin-notification）
    /scheduler               # 轮询/cron（tokio, tokio-cron-scheduler）
    /secure_storage          # Token 安全存储（keyring）
    /runtime_tauri           # Tauri commands / bootstrap
    /runtime_server          # Axum（future，v1.1+）
    /shared_contracts        # JSON schema / DTO / enum 导出
```

---

## 兼容性注意事项

| 关注点 | 状态 | 说明 |
|--------|------|------|
| Tauri v2 + SvelteKit 2 | ✅ 官方支持 | Tauri 文档有专门集成指南 |
| rusqlite `bundled` 特性 | ✅ 推荐启用 | 内置 SQLite 编译，避免系统库版本不一致 |
| reqwest 默认使用 rustls | ✅ 跨平台一致 | 无需系统 OpenSSL 依赖 |
| keyring 跨平台 | ✅ macOS/Windows/Linux | 需分别启用 `apple-native`/`windows-native`/`sync-secret-service` |
| Svelte 5 Runes + Tauri IPC | ✅ 无冲突 | Runes 是编译时特性，不影响运行时 IPC |
| tokio 版本对齐 | ⚠️ 需注意 | 确保所有依赖使用 tokio 1.x；octocrab/reqwest/tokio-cron-scheduler 均使用 tokio 1.x |

---

## 置信度评估

| 领域 | 置信度 | 原因 |
|------|--------|------|
| Tauri v2 | HIGH | 官方文档 + Context7 + crates.io 验证；v2.6.x 为当前稳定版 |
| SvelteKit + Svelte 5 | HIGH | Tauri 官方集成指南验证；Svelte 5.54.0 GA 已发布超过 1 年 |
| rusqlite | HIGH | 多篇 2026 年对比文章一致推荐；crates.io 版本确认 |
| octocrab | HIGH | GitHub 仓库活跃；v0.49.5 已支持 HTTP 缓存；1.3k stars |
| keyring | HIGH | crates.io 活跃更新；keyring-rs GitHub 有 Tauri v2 示例项目 |
| reqwest | HIGH | 4 亿下载量；v0.13.2 为最新稳定版 |
| 调度方案 | MEDIUM | tokio-cron-scheduler 成熟但无大规模 Tauri 桌面应用案例验证；`tokio::time::interval` 更简单稳妥 |
| Stronghold 不推荐 | HIGH | Tauri 官方维护者公开声明将废弃；社区普遍反馈不佳 |

---

## 来源

- [Tauri v2 官方文档](https://v2.tauri.app/) — HIGH
- [Tauri + SvelteKit 集成指南](https://v2.tauri.app/start/frontend/sveltekit/) — HIGH
- [rusqlite crates.io](https://crates.io/crates/rusqlite) — HIGH
- [octocrab CHANGELOG](https://github.com/XAMPPRocky/octocrab/blob/master/CHANGELOG.md) — HIGH
- [reqwest crates.io](https://crates.io/crates/reqwest) — HIGH
- [keyring-rs GitHub](https://github.com/open-source-cooperative/keyring-rs) — HIGH
- [Rust ORMs 2026 对比](https://aarambhdevhub.medium.com/rust-orms-in-2026-diesel-vs-sqlx-vs-seaorm-vs-rusqlite-which-one-should-you-actually-use-706d0fe912f3) — HIGH
- [rusqlite_migration](https://crates.io/crates/rusqlite_migration) — HIGH
- [Reddit: Stronghold 不推荐](https://www.reddit.com/r/rust/comments/1qgb3rq/) — HIGH
- [keyring-demo Tauri v2 项目](https://github.com/open-source-cooperative/keyring-demo) — HIGH

---

*Last updated: 2026-03-22*
