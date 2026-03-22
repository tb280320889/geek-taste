# Architecture Patterns: Tauri v2 + SvelteKit Desktop Tech Radar

**Domain:** Desktop-first GitHub tech radar workbench
**Researched:** 2026-03-22
**Confidence:** HIGH (official Tauri v2 docs + multiple verified sources)

---

## Recommended Architecture

### 概览：四层 + Tauri 壳

```
┌─────────────────────────────────────────────────────────────┐
│                     SvelteKit SPA (WebView)                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ Components│  │  Stores  │  │  Routes  │  │ Services │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│         ▲ invoke() / listen() │ emit()        ▲              │
├─────────┼─────────────────────┼───────────────┼──────────────┤
│         │    Tauri IPC Bridge │               │              │
├─────────┼─────────────────────┼───────────────┼──────────────┤
│         ▼        ▼            ▼               ▼              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │         runtime_tauri (Tauri Commands + Events)      │    │
│  │    #[tauri::command] fn sync_subscriptions(...)      │    │
│  └────────────────────────┬────────────────────────────┘    │
│                           │ depends on                       │
│  ┌────────────────────────▼────────────────────────────┐    │
│  │         application (Use Cases / Orchestration)      │    │
│  │    sync_subscriptions(), refresh_ranking_view()      │    │
│  └────┬──────────────┬──────────────┬──────────────────┘    │
│       │              │              │                         │
│  ┌────▼────┐   ┌─────▼─────┐  ┌────▼──────────────┐        │
│  │ domain  │   │github_    │  │persistence_       │        │
│  │(pure)   │   │adapter    │  │sqlite              │        │
│  │         │   │(REST)     │  │(repositories)      │        │
│  └─────────┘   └───────────┘  └───────────────────┘        │
│                                                               │
│  ┌──────────────────┐  ┌─────────────────────┐              │
│  │notification_     │  │shared_contracts     │              │
│  │adapter           │  │(DTO/enum export)    │              │
│  └──────────────────┘  └─────────────────────┘              │
└─────────────────────────────────────────────────────────────┘
```

### 关键架构约束

1. **Rust 是权威来源**：域模型、状态、业务逻辑在 Rust 侧；SvelteKit 只做呈现
2. **IPC 是唯一桥梁**：前端通过 `invoke()` 调用 Rust 命令，Rust 通过 `emit()` 推送事件
3. **Tauri v2 不支持 SSR**：SvelteKit 必须使用 `adapter-static` SPA 模式
4. **共享类型由 Rust 生成**：`shared_contracts` crate 导出 JSON Schema / TypeScript 类型定义

---

## Component Boundaries

### 层级职责矩阵

| 组件 | 职责 | 依赖 | 禁止 |
|------|------|------|------|
| **SvelteKit UI** | 呈现视图、路由、用户交互、view model | `@tauri-apps/api`, shared TS contracts | 定义域语义、直接调 GitHub API、实现评分公式 |
| **runtime_tauri** | Tauri 命令注册、事件桥接、app 启动、状态管理 | `application`, `tauri` | 包含业务逻辑、直接访问 SQLite |
| **application** | 用例编排、策略、权限检查 | `domain`, all adapters (as traits) | 依赖 Tauri、依赖具体 HTTP 库 |
| **domain** | 纯实体、值对象、状态机、规则接口 | 无外部依赖 (纯 Rust) | HTTP/SQLite/Tauri/serde 以外的任何 IO |
| **github_adapter** | GitHub REST 客户端、响应映射、速率限制 | `domain`, `reqwest` | 持久化、业务逻辑 |
| **persistence_sqlite** | SQLite repository 实现、migration | `domain`, `rusqlite`/`sqlx` | GitHub 语义、通知逻辑 |
| **notification_adapter** | 桌面通知 (tauri-plugin-notification) | `domain`, `tauri` | 持久化、GitHub 调用 |
| **shared_contracts** | IPC DTO、枚举、JSON Schema | `domain` (re-export) | 业务逻辑、实现细节 |
| **runtime_server** | Axum API (future v1.1+) | `application`, `axum` | v1 不实现 |

### 依赖方向 (必须单向)

```
SvelteKit UI ──→ shared_contracts (TS types)
     │
     ▼
runtime_tauri ──→ application ──→ domain
                    │    │    │
                    ▼    ▼    ▼
              github_adapter  persistence_sqlite  notification_adapter
                    │              │
                    └──────────────┘
                         shared_contracts
```

**规则**：依赖只能从外层指向内层，绝不反向。

---

## Data Flow

### 1. 命令调用流 (Frontend → Rust)

```typescript
// SvelteKit: 用户点击"同步订阅"
import { invoke } from '@tauri-apps/api/core';
import type { SignalCard } from '$lib/contracts';

const signals = await invoke<SignalCard[]>('sync_subscriptions', {
  subscriptionIds: ['sub_01', 'sub_02']
});
```

```rust
// runtime_tauri: 命令入口
#[tauri::command]
async fn sync_subscriptions(
    state: State<'_, AppState>,
    subscription_ids: Vec<String>,
) -> Result<Vec<SignalCard>, CommandError> {
    let app_service = state.application_service.lock().await;
    let signals = app_service.sync_subscriptions(&subscription_ids).await?;
    Ok(signals.into_iter().map(SignalCard::from).collect())
}
```

**流路径**：
```
SvelteKit invoke() → Tauri IPC → runtime_tauri command handler
    → application service → github_adapter (HTTP) + persistence_sqlite (write)
    → 返回 Vec<SignalCard> → Tauri IPC → SvelteKit Promise resolve
```

### 2. 事件推送流 (Rust → Frontend)

**用于**：后台同步进度、新信号通知、digest 生成完成

```rust
// application service 完成同步后
app.emit("sync:completed", SyncCompletedEvent {
    subscription_id: "sub_01",
    new_signal_count: 3,
}).unwrap();
```

```typescript
// SvelteKit: 监听后台事件
import { listen } from '@tauri-apps/api/event';

listen<SyncCompletedEvent>('sync:completed', (event) => {
  // 更新 store，触发 UI 刷新
  signalStore.addNewSignals(event.payload.newSignalCount);
});
```

**流路径**：
```
tokio::spawn (后台任务) → application service 完成工作
    → app.emit("event_name", payload) → Tauri 事件系统
    → SvelteKit listen() 回调 → 更新 store → UI 响应式刷新
```

### 3. 后台轮询架构

```
┌────────────────────────────────────────────────────────┐
│                  Scheduler (Tokio Runtime)              │
│                                                         │
│  ┌───────────────────┐    ┌───────────────────────┐   │
│  │ subscription_poll │    │ ranking_view_refresh  │   │
│  │ interval: 15min   │    │ interval: 1h          │   │
│  │ per-repo throttle │    │ batch query + diff    │   │
│  └────────┬──────────┘    └──────────┬────────────┘   │
│           │                          │                  │
│           ▼                          ▼                  │
│  ┌────────────────────────────────────────────────┐   │
│  │    application: sync_subscriptions()            │   │
│  │    1. 从 SQLite 读取 cursor / ETag              │   │
│  │    2. 调用 github_adapter (条件请求)            │   │
│  │    3. Diff 新旧数据，生成 Signal                │   │
│  │    4. 写入 SQLite + 更新 cursor                 │   │
│  │    5. emit("sync:completed") → 通知 UI          │   │
│  └────────────────────────────────────────────────┘   │
│                                                         │
│  ┌───────────────────┐    ┌───────────────────────┐   │
│  │ digest_generator  │    │ notification_sender   │   │
│  │ window: 12h/24h   │    │ on: HIGH priority     │   │
│  └────────┬──────────┘    └──────────┬────────────┘   │
│           │                          │                  │
│           ▼                          ▼                  │
│  emit("digest:ready")    emit("notification:sent")    │
└────────────────────────────────────────────────────────┘
```

**关键实现细节**：

```rust
// runtime_tauri/lib.rs setup 中启动后台调度
tauri::Builder::default()
    .manage(AppState::new(/* ... */))
    .setup(|app| {
        let app_handle = app.handle().clone();
        // 启动后台轮询调度器
        tauri::async_runtime::spawn(async move {
            let scheduler = SubscriptionScheduler::new(app_handle);
            scheduler.run().await;
        });
        Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        sync_subscriptions,
        refresh_ranking_view,
        create_subscription,
        // ...
    ])
```

### 4. 离线降级流

```
UI 请求 → invoke() → command handler
    → 检查网络状态
    → 无网络: 从 SQLite 读取缓存，标记 STALE
    → 有网络: 正常请求 + 缓存
    → 返回数据 (带 freshness 标记)
```

---

## IPC 结构设计

### 命令命名约定

```
verb_noun                    // 简单操作
sync_subscriptions           // 同步
refresh_ranking_view         // 刷新
generate_digest              // 生成
ack_signal                   // 确认
create_subscription          // 创建
delete_subscription          // 删除
get_subscription_list        // 查询列表
get_signal_feed              // 获取信号流
```

### 错误处理模式

```rust
// shared_contracts: 统一错误类型
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,        // "RATE_LIMITED", "NETWORK_OFFLINE", "NOT_FOUND"
    pub message: String,     // 人类可读描述
    pub retry_after: Option<u64>,  // 秒数，如果可重试
}

// 所有 command 返回 Result<T, CommandError>
impl From<GitHubApiError> for CommandError { /* ... */ }
impl From<SqliteError> for CommandError { /* ... */ }
```

### 类型安全桥梁

**推荐方案**：`shared_contracts` crate 使用 `serde` + `ts-rs` 或 `specta` 自动生成 TypeScript 类型。

```rust
// shared_contracts/src/signal.rs
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]  // 自动生成 .ts 文件
pub struct SignalCard {
    pub signal_id: String,
    pub signal_type: SignalType,
    pub priority: Priority,
    pub state: SignalState,
    pub title: String,
    pub summary: Option<String>,
    pub occurred_at: String,  // ISO8601
    pub evidence: serde_json::Value,
}
```

**MEDIUM confidence**：`ts-rs` 和 `specta` 都支持 Tauri v2，但具体集成方式需验证。`tauri-specta` 是目前 Tauri 生态中较成熟的方案。

---

## SvelteKit 集成配置

### 必要配置 (来自 Tauri v2 官方文档)

```javascript
// svelte.config.js
import adapter from '@sveltejs/adapter-static';

const config = {
  kit: {
    adapter: adapter({
      fallback: 'index.html'  // SPA 模式
    })
  }
};
```

```typescript
// src/routes/+layout.ts (根布局)
export const prerender = false;  // SPA 模式不需要预渲染
export const ssr = false;        // Tauri 不支持 SSR
export const trailingSlash = 'always';
```

```json
// src-tauri/tauri.conf.json
{
  "build": {
    "frontendDist": "../build",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  }
}
```

### 前端状态管理推荐

```
三层状态模型：

1. 组件本地状态     → Svelte $state (runes)
2. 全局 UI 状态     → Svelte store 或 Zustand
3. 持久化数据状态   → invoke() + cache layer
```

**理由**：
- Tauri 的状态在 Rust 侧，前端不需要全局状态管理库来管理"权威数据"
- 前端 store 只缓存最近一次 invoke 的结果 + UI 临时状态
- 当 Rust 通过 emit 推送事件时，前端更新对应 store

---

## Cargo Workspace 组织

### 推荐结构

```
/geek-taste
├── Cargo.toml                    # workspace root
├── Cargo.lock                    # 共享锁文件
├── package.json                  # SvelteKit 前端
├── svelte.config.js
├── vite.config.ts
├── src/                          # SvelteKit 前端
│   ├── routes/
│   ├── lib/
│   │   ├── contracts/            # 从 shared_contracts 生成的 TS 类型
│   │   ├── stores/
│   │   └── services/
│   └── app.html
├── src-tauri/                    # Tauri 应用入口 (workspace member)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs                # 应用构建 + 命令注册
│   │   └── commands/             # Tauri command 实现
│   └── icons/
├── crates/
│   ├── domain/                   # 纯领域对象 (无外部依赖)
│   ├── application/              # 用例编排
│   ├── github_adapter/           # GitHub REST
│   ├── persistence_sqlite/       # SQLite 持久化
│   ├── notification_adapter/     # 桌面通知
│   ├── shared_contracts/         # IPC DTO + 类型导出
│   └── runtime_server/           # Axum (future)
└── docs/
```

### Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "src-tauri",
    "crates/*",
]
exclude = [
    "crates/runtime_server",  # v1.1+ 才启用
]

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
```

### Tauri 与 Workspace 的集成注意点

**已知问题 (Tauri v1 历史)**：
- Tauri v1 曾有 workspace inheritance 问题 ([#6252](https://github.com/tauri-apps/tauri/issues/6252))，已在 v2 修复
- `tauri.conf.json` 必须在 `src-tauri/` 目录下，Tauri CLI 通过此文件定位 Rust 项目

**v2 正确做法**：
- `src-tauri/` 作为 workspace 成员
- `src-tauri/Cargo.toml` 可以使用 `{ workspace = true }` 继承 workspace 配置
- Tauri CLI 从项目根目录运行 `npm run tauri dev`

---

## Tauri v2 特定模式

### State Management 模式

```rust
// 推荐: 使用 tokio::sync::Mutex (异步安全)
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct AppState {
    pub application_service: Arc<Mutex<ApplicationService>>,
    pub scheduler_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

// 注册
tauri::Builder::default()
    .manage(AppState { /* ... */ })

// 在 command 中访问
#[tauri::command]
async fn get_signals(
    state: State<'_, AppState>,
) -> Result<Vec<SignalCard>, CommandError> {
    let service = state.application_service.lock().await;
    service.get_recent_signals().await
        .map(|s| s.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}
```

**选择 `tokio::sync::Mutex` 而非 `std::sync::Mutex` 的原因**：
- Application Service 需要 await 异步操作 (数据库、HTTP)
- std::sync::Mutex 在持有锁时不能 `.await`
- tokio::sync::Mutex 允许跨越 await 点持有锁

### Background Task 模式

```rust
// 长期运行的后台轮询任务
async fn start_subscription_poller(app: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(900)); // 15min

    loop {
        interval.tick().await;

        // 检查网络 + 速率限制
        if !is_network_available().await {
            continue;
        }

        // 执行同步
        match sync_all_active_subscriptions(&app).await {
            Ok(result) => {
                // 推送事件给 UI
                app.emit("sync:completed", result).ok();
            }
            Err(e) => {
                app.emit("sync:failed", e.to_string()).ok();
                // 指数退避
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        }
    }
}
```

### Event 系统模式

```rust
// Rust → Frontend 事件命名约定
// 格式: "{domain}:{action}"
"sync:started"
"sync:progress"
"sync:completed"
"sync:failed"
"digest:ready"
"notification:triggered"
"ranking:updated"
"app:offline"
"app:online"
```

```typescript
// SvelteKit 中的事件监听组织
// lib/events.ts - 集中管理事件订阅
export function setupEventListeners() {
    const unlisteners: Array<() => void> = [];

    listen<SyncCompleted>('sync:completed', (e) => {
        signalStore.handleSyncCompleted(e.payload);
    }).then(unlisten => unlisteners.push(unlisten));

    // ...

    return () => unlisteners.forEach(u => u());
}
```

---

## Build Order 建议

### 阶段依赖图

```
Phase 1: 基础骨架
├── domain crate (实体 + 值对象 + 状态机)
├── shared_contracts (DTO 定义)
├── SvelteKit 脚手架 + Tauri 壳
└── runtime_tauri 基础命令注册

Phase 2: 数据层
├── persistence_sqlite (schema + migration + repository)
├── github_adapter (REST client + 认证 + 速率限制)
└── 集成测试: CRUD → SQLite → GitHub mock

Phase 3: 核心用例
├── application service (sync, subscribe, ranking)
├── runtime_tauri commands (桥接 application → IPC)
└── SvelteKit 页面 (TopK, Subscriptions, Home)

Phase 4: 后台调度
├── SubscriptionScheduler (tokio interval)
├── RankingViewRefresher
├── DigestGenerator
└── notification_adapter

Phase 5: 打磨 + 云预备
├── 离线降级
├── 错误处理 + 重试策略
├── 性能优化
└── runtime_server 脚手架 (Axum)
```

### 为什么这个顺序

1. **Domain 先行**：所有层都依赖 domain 定义，不先锁定实体，后续全部返工
2. **Persistence + GitHub adapter 并行**：它们都实现 domain 的 trait，互不依赖
3. **Application 层在 adapter 就绪后**：需要真实 adapter 来编排用例
4. **UI 在 command 就绪后**：前端需要 invoke 有真实后端可调
5. **后台调度最后**：依赖 application service 全部就绪

---

## Tauri v2 vs v1 考量

### 已确认 v2 变更

| 特性 | v1 | v2 | 本项目影响 |
|------|----|----|-----------|
| **安全模型** | allowlist in tauri.conf.json | Capabilities (deny-by-default) | 必须为每个插件/命令声明 permission |
| **命令签名** | `#[tauri::command] fn foo(window: Window)` | `#[tauri::command] fn foo(window: Window)` (类似，但 trait 变化) | `Manager` trait 更统一 |
| **事件系统** | `window.emit()` / `app.emit()` | `app.emit()` + `Emitter` trait | 更一致，去掉 window 特定事件 |
| **插件系统** | 简单 | 插件更成熟 (plugin-store, plugin-notification 等) | 可直接使用官方插件 |
| **移动端** | beta | 正式支持 | v1 不需要，但架构预留 |
| **异步** | 特定于 Tauri 的异步运行时 | 直接使用 Tokio runtime | 后台任务更自然 |

### 必须使用 v2 的理由

1. **安全模型**：Capabilities 系统对 GitHub token 保护更严格
2. **插件生态**：`tauri-plugin-notification`、`tauri-plugin-store` 直接满足需求
3. **Tokio 原生支持**：后台调度无需额外配置
4. **SvelteKit 官方支持文档**：v2 有明确的 SvelteKit 集成指南

---

## Anti-Patterns（必须避免）

### Anti-Pattern 1: 在前端定义域语义

**错误**：
```typescript
// ❌ 前端自行定义信号类型枚举
enum SignalType {
  Release = 'release',
  Tag = 'tag',
  Branch = 'branch',
}
```

**正确**：
```typescript
// ✅ 从 shared_contracts 生成的类型导入
import type { SignalType } from '$lib/contracts/generated';
```

**后果**：前后端类型漂移，数据不一致，难以维护。

### Anti-Pattern 2: 前端直接拼 GitHub 查询

**错误**：
```typescript
// ❌ 前端构造 GitHub API URL
const url = `https://api.github.com/search/repos?q=language:${lang}&sort=stars`;
```

**正确**：
```typescript
// ✅ 通过 invoke 调用 Rust 命令
const results = await invoke<RepositoryCard[]>('search_trending', {
  language: 'Rust',
  timeWindow: '7d',
});
```

**后果**：绕过速率限制、绕过缓存、暴露 token 风险。

### Anti-Pattern 3: 在 command handler 中写业务逻辑

**错误**：
```rust
// ❌ command 直接包含同步逻辑
#[tauri::command]
async fn sync_repo(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client.get(format!("https://api.github.com/repositories/{}", id))
        .send().await.map_err(|e| e.to_string())?;
    let data = resp.json::<Value>().await.map_err(|e| e.to_string())?;
    // 写入数据库...
    Ok(())
}
```

**正确**：
```rust
// ✅ command 只做桥接，业务逻辑在 application 层
#[tauri::command]
async fn sync_repo(
    id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<SignalCard>, CommandError> {
    let service = state.application_service.lock().await;
    service.sync_repo_by_id(id).await
        .map(|signals| signals.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}
```

**后果**：逻辑不可测试、不可复用、破坏分层。

### Anti-Pattern 4: 同步阻塞 Tokio 运行时

**错误**：
```rust
// ❌ 在 async fn 中使用 std::thread::sleep 或同步 IO
async fn poll_github() {
    loop {
        let data = std::fs::read_to_string("cache.json").unwrap(); // 阻塞!
        std::thread::sleep(Duration::from_secs(60)); // 阻塞!
    }
}
```

**正确**：
```rust
// ✅ 使用 Tokio 异步 IO
async fn poll_github() {
    loop {
        let data = tokio::fs::read_to_string("cache.json").await;
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

**后果**：阻塞整个 Tokio runtime，UI 卡死，所有异步任务停滞。

### Anti-Pattern 5: 每个前端状态变更都调 invoke

**错误**：
```typescript
// ❌ 用户输入搜索框就调用 Rust
<input oninput={(e) => invoke('search', { query: e.target.value })} />
```

**正确**：
```typescript
// ✅ 防抖 + 本地缓存
let debounceTimer: ReturnType<typeof setTimeout>;

<input oninput={(e) => {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(async () => {
    results = await invoke('search', { query: e.target.value });
  }, 300);
}} />
```

**后果**：高频 IPC 调用，性能差，速率限制被快速消耗。

### Anti-Pattern 6: 在 SQLite 中存储 GitHub token

**错误**：
```sql
-- ❌ token 明文存储
INSERT INTO config (key, value) VALUES ('github_token', 'ghp_xxxx');
```

**正确**：
```rust
// ✅ 使用 OS 安全存储 (tauri-plugin-stronghold 或 keyring-rs)
use keyring::Entry;
let entry = Entry::new("geek-taste", "github-token")?;
entry.set_password(&token)?;
```

**后果**：数据库泄露 = token 泄露，违反安全规范。

---

## Scalability 考量

| 关注点 | 100 订阅 | 500 订阅 | 2000+ 订阅 |
|--------|---------|---------|-----------|
| 轮询频率 | 15min 全量 | 15min + 优先级队列 | 分层：活跃 15min，休眠 2h |
| SQLite 写入 | 串行即可 | 串行 + WAL mode | 批量写入 + 事务 |
| 信号数量 | 内存可存 | 分页查询 | 分页 + 归档 |
| 前端列表 | 直接渲染 | 虚拟滚动 | 虚拟滚动 + 骨架屏 |
| 速率限制 | 单预算池 | core/search 隔离 | 按端点细分 + 动态调整 |

---

## Confidence Assessment

| 领域 | Confidence | 理由 |
|------|-----------|------|
| Tauri v2 IPC 架构 | HIGH | 官方文档明确，有完整 API 示例 |
| SvelteKit + Tauri 集成 | HIGH | Tauri 官方有专门指南，配置步骤清晰 |
| Cargo workspace 组织 | HIGH | Tauri v2 已修复 workspace 问题，社区有成功案例 |
| 后台任务模式 | HIGH | Tokio + tauri::async_runtime::spawn 是标准做法 |
| State Management | HIGH | 官方文档有明确模式，Mutex/State API 稳定 |
| 类型安全桥 (specta/ts-rs) | MEDIUM | 社区使用中，但具体版本兼容性需实测 |
| 速率限制调度策略 | MEDIUM | 需要实测 GitHub 实际响应行为 |
| 2000+ 订阅扩展性 | LOW | 需要实际负载测试验证 |

---

## Sources

1. Tauri v2 官方文档 - State Management: https://v2.tauri.app/develop/state-management/
2. Tauri v2 官方文档 - Calling Rust: https://v2.tauri.app/develop/calling-rust
3. Tauri v2 官方文档 - Calling Frontend: https://v2.tauri.app/develop/calling-frontend
4. Tauri v2 官方文档 - SvelteKit 集成: https://v2.tauri.app/start/frontend/sveltekit/
5. Tauri v2 官方文档 - Project Structure: https://v2.tauri.app/start/project-structure/
6. Tauri + SvelteKit 讨论: https://github.com/orgs/tauri-apps/discussions/6423
7. Tauri + Workspace 集成讨论: https://github.com/orgs/tauri-apps/discussions/13941
8. 长期后台任务模式: https://sneakycrow.dev/blog/2024-05-12-running-async-tasks-in-tauri-v2
9. Cargo Workspace 最佳实践: https://reintech.io/blog/cargo-workspace-best-practices-large-rust-projects
10. 项目规范 docs/03: 系统架构规范
11. 项目规范 docs/05: 数据模型与契约规范

---

*Research completed 2026-03-22. Confidence: HIGH overall. MEDIUM on specta/ts-rs integration details — needs Phase 1 验证.*
