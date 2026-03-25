---
phase: 05-polish-release
verified: 2026-03-25T12:00:00Z
status: gaps_found
score: 9/14 must-haves verified
re_verification: false
gaps:
  - truth: "离线时应用顶部显示网络状态提示"
    status: failed
    reason: "isOnline store 存在但 +layout.svelte 未导入也未渲染离线 banner；app.css 无 offline-banner 样式；checkNetworkStatus() 从未从任何组件调用"
    artifacts:
      - path: "apps/desktop-ui/src/routes/+layout.svelte"
        issue: "未导入 isOnline，无离线 banner 渲染代码"
      - path: "apps/desktop-ui/src/app.css"
        issue: "无 .offline-banner 样式类"
    missing:
      - "+layout.svelte 导入 isOnline 并在 {#if !$isOnline} 中渲染离线 banner"
      - "app.css 添加 .offline-banner 样式"
      - "onMount 中调用 checkNetworkStatus()"
  - truth: "过期数据有明确的 STALE 标识"
    status: failed
    reason: "lastSyncedAt store 存在但 +page.svelte 未导入或使用；app.css 无 .stale-dot 样式"
    artifacts:
      - path: "apps/desktop-ui/src/routes/+page.svelte"
        issue: "未导入 lastSyncedAt，未渲染 STALE 圆点"
      - path: "apps/desktop-ui/src/app.css"
        issue: "无 .stale-dot 样式类"
    missing:
      - "+page.svelte 导入 lastSyncedAt 并在 Today 标题旁/TopK 卡片旁渲染 STALE 圆点"
      - "app.css 添加 .stale-dot 样式（琥珀色圆点）"
  - truth: "sync_status IPC 命令可运行"
    status: failed
    reason: "sync_status.rs 文件存在但未在 commands/mod.rs 中声明，也未在 lib.rs handler 中注册；get_sync_status IPC 调用将在运行时返回 'command not found'"
    artifacts:
      - path: "crates/runtime_tauri/src/commands/mod.rs"
        issue: "缺少 pub mod sync_status; 声明"
      - path: "apps/desktop-ui/src-tauri/src/lib.rs"
        issue: "invoke_handler 未注册 get_sync_status"
    missing:
      - "commands/mod.rs 添加 pub mod sync_status;"
      - "lib.rs handler 注册 get_sync_status"
  - truth: "所有 TopK/订阅/信号/资源 IPC 命令可运行"
    status: failed
    reason: "topk.rs, signal.rs, subscription.rs, resource.rs 文件均存在但未在 commands/mod.rs 中声明，handler 未注册。前端 ~20 个 IPC invoke 调用将在运行时失败。此为 Phase 02-04 遗留问题"
    artifacts:
      - path: "crates/runtime_tauri/src/commands/mod.rs"
        issue: "仅声明了 auth 和 settings，缺少 topk/signal/subscription/resource 模块"
      - path: "apps/desktop-ui/src-tauri/src/lib.rs"
        issue: "handler 仅注册 8 个 auth/settings 命令，缺少 topk/signal/subscription/resource/sync_status 命令"
    missing:
      - "commands/mod.rs 添加 pub mod topk; pub mod signal; pub mod subscription; pub mod resource; pub mod sync_status;"
      - "lib.rs invoke_handler 注册所有缺失的命令"
  - truth: "列表渲染性能达标，无明显卡顿"
    status: partial
    reason: "SUMMARY 声称已有 keyed each blocks（已验证存在），但计划中的 CSS 过渡动画未实现（app.css 无 transition/animation 相关样式）"
    artifacts:
      - path: "apps/desktop-ui/src/app.css"
        issue: "无页面切换过渡、卡片 hover 微交互、列表入场动画"
    missing:
      - "app.css 添加必要的 CSS 过渡动画"
human_verification:
  - test: "离线流程：断网 → 打开应用 → 验证缓存数据展示 + STALE 标识 + 离线 banner"
    expected: "应用顶部显示琥珀色网络不可用 banner；Home 页 Today 标题旁显示 STALE 琥珀色圆点"
    why_human: "离线 banner 和 STALE 标识的 UI 渲染需要在真实 Tauri 应用中断网验证"
  - test: "暖机流程：新建 ranking view → 首次执行时显示暖机提示"
    expected: "TopK 页显示 '首次加载数据，可能较慢...' 提示"
    why_human: "暖机提示的实际显示需要首次创建 ranking view 时验证"
  - test: "打包安装包测试：cargo tauri build → 安装 → 首次启动 → 全功能走查"
    expected: "安装包可正常安装启动，所有页面功能可用"
    why_human: "打包流程和安装包功能无法通过 grep 验证"
---

# Phase 05: 打磨与发布准备 — Verification Report

**Phase Goal:** P0-P3 bug fixes, performance polish, Tauri packaging config, auto-update setup — ship v1.0 ready build
**Verified:** 2026-03-25T12:00:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #   | Truth                                        | Status     | Evidence                                                                        |
| --- | -------------------------------------------- | ---------- | ------------------------------------------------------------------------------- |
| 1   | 应用在 GitHub API 不可达时仍能展示 SQLite 缓存数据 | ⚠️ PARTIAL | network.ts store 存在，error_dto.rs + classify_error 实现完整，但离线 banner 未渲染，sync_status 命令未注册 |
| 2   | 过期数据有明确的 STALE 标识                     | ✗ FAILED   | isStale() 函数存在但未在 UI 中调用；+page.svelte 未导入 lastSyncedAt              |
| 3   | 离线时应用顶部显示网络状态提示                   | ✗ FAILED   | isOnline store 存在但 +layout.svelte 未导入；无 offline banner 渲染             |
| 4   | 无快照时 Momentum 降级为 UPDATED_DESC 并提示   | ✓ VERIFIED | warmup 在 Rust→DTO→store→page 全链路 wired，topk page 渲染暖机提示条            |
| 5   | Subscriptions 页面有搜索输入框可搜索仓库        | ✓ VERIFIED | searchQuery + handleSearch + placeholder="搜索仓库..." + $derived 过滤         |
| 6   | TopK 打开后即展示预设 Trending 排名视图         | ✓ VERIFIED | ensureDefaultViews() 在 onMount 调用，3 个预设视图                              |
| 7   | Resources 页面有资源卡片列表展示                | ✓ VERIFIED | resources store + $resources 渲染 + tag 筛选 + 空状态处理                        |
| 8   | 点击'在 GitHub 打开'按钮能在浏览器打开链接      | ✓ VERIFIED | openExternal 使用 @tauri-apps/plugin-shell open()，TopK 页面 onclick 绑定      |
| 9   | Home 页面无订阅时仍展示引导内容                 | ✓ VERIFIED | empty-state 类 + $derived isEmpty + CTA 按钮（TopK/订阅）                       |
| 10  | Settings 页面 Toast 不导致布局抖动              | ✓ VERIFIED | position: fixed + slideIn 动画 + toastType 区分样式                             |
| 11  | 用户可从 Settings 或 Sidebar 注销              | ✓ VERIFIED | Settings 有注销按钮 + Sidebar 有注销按钮（双入口）                               |
| 12  | 应用有 favicon，控制台不报 404                  | ✓ VERIFIED | favicon.ico 存在 (1.2K) + app.html 有 link 标签                                |
| 13  | 应用可打包为安装包                             | ✓ VERIFIED | bundle.active=true, targets="all", icons 完整 (5 种格式)                        |
| 14  | 自动更新机制可用                               | ⚠️ PARTIAL | updater 插件已注册 + 端点为占位符 + pubkey 为占位符 → 需部署时配置              |

**Score:** 9/14 must-haves verified (5 fully verified + 4 partial/failed)

### Critical Pre-Existing Issue: IPC Commands Not Wired

发现一个关键的架构问题——**不只是 Phase 05 的问题，而是 Phases 02-04 遗留的根本性问题**：

`crates/runtime_tauri/src/commands/mod.rs` 仅声明了 `auth` 和 `settings` 两个模块：

```rust
pub mod auth;
pub mod settings;
pub use auth::*;
pub use settings::*;
```

以下模块文件存在于 `commands/` 目录但**未被编译**：
- `topk.rs` — 5 个 Tauri 命令（list_ranking_views, create_ranking_view, delete_ranking_view, toggle_pin_ranking_view, execute_ranking）
- `signal.rs` — 5 个 Tauri 命令（list_signals, list_home_signals, ack_signal, mark_signal_seen, get_unread_counts）
- `subscription.rs` — 5 个 Tauri 命令（subscribe, unsubscribe, pause_subscription, list_subscriptions, sync_subscriptions）
- `resource.rs` — 4 个 Tauri 命令（list_resources, search_resources, curate_resource, deactivate_resource）
- `sync_status.rs` — 1 个 Tauri 命令（get_sync_status）

`apps/desktop-ui/src-tauri/src/lib.rs` 的 `invoke_handler` 仅注册了 8 个 auth/settings 命令。

**影响：** 前端 IPC wrapper (`tauri.ts`) 包含 28 个 invoke 调用，其中 ~20 个调用未注册的命令，运行时将返回 "command not found"。

**这意味着：** 所有依赖 TopK/订阅/信号/资源 IPC 的前端功能（Subscriptions 搜索、TopK 排名、Resources 列表、Home 信号摘要）在运行时不可用。SUMMARIES 中的 Self-Check PASSED 基于文件内容 grep 验证，未验证实际编译注册状态。

### Required Artifacts

| Artifact                                                  | Expected               | Status           | Details                                              |
| --------------------------------------------------------- | ---------------------- | ---------------- | ---------------------------------------------------- |
| `crates/shared_contracts/src/error_dto.rs`                | AppErrorDto + ErrorKind | ✓ VERIFIED       | 23 行，5 种 ErrorKind 分类，Display 实现              |
| `crates/runtime_tauri/src/commands/helpers.rs`            | classify_error 函数    | ✓ VERIFIED       | 13 处引用，完整 5 分支模式匹配                        |
| `crates/runtime_tauri/src/commands/sync_status.rs`        | get_sync_status 命令   | ⚠️ ORPHANED      | 文件存在但未在 mod.rs 声明，未在 handler 注册          |
| `apps/desktop-ui/src/lib/stores/network.ts`               | isOnline/lastSyncedAt  | ✓ VERIFIED       | 48 行，5 个导出函数/变量                             |
| `apps/desktop-ui/src/routes/+layout.svelte`               | 离线 banner            | ✗ MISSING        | 未导入 isOnline，未渲染离线提示                       |
| `apps/desktop-ui/src/routes/+page.svelte`                 | STALE 标识             | ✗ MISSING        | 未导入 lastSyncedAt，未渲染 STALE 圆点                |
| `apps/desktop-ui/src/routes/subscriptions/+page.svelte`   | 搜索功能               | ✓ VERIFIED       | searchQuery + handleSearch + 输入框 + 过滤           |
| `apps/desktop-ui/src/routes/topk/+page.svelte`            | 默认视图 + 暖机提示    | ✓ VERIFIED       | ensureDefaultViews + warmup 渲染 + openExternal       |
| `apps/desktop-ui/src/routes/resources/+page.svelte`       | 资源卡片列表           | ✓ VERIFIED       | 资源渲染 + 标签筛选 + 空状态                         |
| `apps/desktop-ui/src/routes/settings/+page.svelte`        | Toast fixed + 注销     | ✓ VERIFIED       | position:fixed + slideIn + logout 按钮               |
| `apps/desktop-ui/src/lib/stores/auth.ts`                  | logout 方法            | ✓ VERIFIED       | Sidebar + Settings 双入口引用                        |
| `apps/desktop-ui/static/favicon.ico`                      | favicon 文件           | ✓ VERIFIED       | 1.2K，app.html 有 link 标签                         |
| `apps/desktop-ui/src-tauri/tauri.conf.json`               | 打包 + updater 配置    | ✓ VERIFIED       | bundle.active=true + updater 端点占位符              |
| `apps/desktop-ui/src-tauri/Cargo.toml`                    | updater 依赖           | ✓ VERIFIED       | tauri-plugin-updater + custom-protocol feature       |
| `apps/desktop-ui/src-tauri/src/lib.rs`                    | updater 插件注册       | ✓ VERIFIED       | .plugin(tauri_plugin_updater::init())                |

### Key Link Verification

| From                                              | To                                       | Via                          | Status       | Details                                       |
| ------------------------------------------------- | ---------------------------------------- | ---------------------------- | ------------ | --------------------------------------------- |
| `helpers.rs`                                      | `error_dto.rs`                           | classify_error → AppErrorDto  | ✓ WIRED      | use shared_contracts::error_dto::{AppErrorDto, ErrorKind} |
| `topk.ts` store                                   | `network.ts`                             | handleIpcError               | ✓ WIRED      | 所有 catch 块调用 handleIpcError              |
| `+layout.svelte`                                  | `network.ts`                             | $isOnline                    | ✗ NOT WIRED  | layout 未导入 isOnline                        |
| `+page.svelte`                                    | `network.ts`                             | lastSyncedAt                 | ✗ NOT WIRED  | +page.svelte 未导入 lastSyncedAt              |
| `subscriptions/+page.svelte`                      | `subscriptions.ts` store                 | 搜索过滤                      | ✓ WIDER      | $derived + searchQuery 过滤                   |
| `topk/+page.svelte`                               | `topk.ts` store                          | ensureDefaultViews           | ✓ WIRED      | onMount 调用                                  |
| `topk/+page.svelte`                               | `tauri.ts` IPC                            | openExternal                 | ✓ WIRED      | import + onclick 调用                         |
| `settings/+page.svelte`                           | `auth.ts` store                          | logout                       | ✓ WIRED      | import + handleLogout                         |
| `src-tauri/tauri.conf.json`                       | `src-tauri/Cargo.toml`                   | bundle 配置与 Cargo 对齐      | ✓ WIRED      | tauri-plugin-updater 依赖存在                 |
| `commands/mod.rs`                                 | `topk.rs` / `signal.rs` / etc.          | pub mod 声明                  | ✗ NOT WIRED  | 仅 auth + settings 声明                       |
| `lib.rs handler`                                  | `topk/signal/subscription/resource` 命令 | generate_handler!            | ✗ NOT WIRED  | 仅 8 个 auth/settings 命令                    |

### Behavioral Spot-Checks

| Behavior                                  | Command                      | Result              | Status |
| ----------------------------------------- | ---------------------------- | ------------------- | ------ |
| Rust workspace 编译通过                    | `cargo check -p runtime_tauri` | ✅ 0 errors        | ✓ PASS |
| 前端 svelte-check                          | `npx svelte-check` (from SUMMARY) | ✅ 0 errors (SUMMARY) | ✓ PASS |
| cargo test 通过                            | (SUMMARY: 78 passed)         | ✅                  | ✓ PASS |
| offline banner 渲染                        | grep +layout.svelte          | ❌ 未发现            | ✗ FAIL |
| STALE 标识渲染                             | grep +page.svelte            | ❌ 未发现            | ✗ FAIL |
| IPC 命令注册完整度                         | grep mod.rs + lib.rs         | ❌ 仅 8/28 命令      | ✗ FAIL |

### Requirements Coverage

| Requirement | Source Plan | Description                    | Status          | Evidence                                                       |
| ----------- | ---------- | ------------------------------ | --------------- | -------------------------------------------------------------- |
| HOME-03     | 05-01, 05-05 | 离线可用 + STALE 标识          | ⚠️ PARTIALLY SATISFIED | backend 部分完整（error_dto, classify_error, sync_status.rs），但 UI 未接入 |
| HOME-01     | 05-03      | Home 页面信号摘要              | ⚠️ BLOCKED      | 依赖 signal IPC（未注册）+ 前端实现存在但 IPC 不通               |
| HOME-02     | 05-03      | Home 空状态引导                | ✓ SATISFIED     | 纯前端逻辑，不依赖 IPC，empty-state 渲染 + CTA 按钮             |

### Anti-Patterns Found

| File                                       | Line | Pattern                     | Severity   | Impact                                          |
| ------------------------------------------ | ---- | --------------------------- | ---------- | ----------------------------------------------- |
| `commands/mod.rs`                          | -    | Missing module declarations | 🛑 Blocker | 20+ IPC 命令在运行时不可用                       |
| `lib.rs` (handler)                         | -    | Missing command registration | 🛑 Blocker | 同上                                             |
| `+layout.svelte`                           | -    | Unwired store (isOnline)    | ⚠️ Warning | 离线 banner 不显示                                |
| `+page.svelte`                             | -    | Unwired store (lastSyncedAt)| ⚠️ Warning | STALE 标识不显示                                  |
| `network.ts`                               | 37   | Unused export (checkNetworkStatus) | ℹ️ Info | 函数定义但从未被调用                              |

### Human Verification Required

#### 1. 离线降级流程

**Test:** 断网 → 打开应用 → 检查 UI 状态
**Expected:** 应用顶部显示琥珀色"网络不可用"banner；Home 页 Today 标题旁显示 STALE 琥珀色圆点
**Why human:** 离线 banner 和 STALE 标识需要在真实 Tauri 应用中断网验证（当前代码未渲染这些 UI 元素，预期会失败）

#### 2. 暖机流程

**Test:** 首次创建 ranking view → 检查 TopK 页面
**Expected:** TopK 页显示"首次加载数据，可能较慢..."提示条
**Why human:** 暖机提示的触发需要首次创建 view 时验证（代码已 wired，但需运行验证）

#### 3. 打包安装包

**Test:** `cargo tauri build` → 安装 → 首次启动 → 全功能走查
**Expected:** 安装包可正常安装启动，所有页面功能可用
**Why human:** 打包流程和安装包质量无法通过 grep 验证

### Gaps Summary

Phase 05 的 5 个子计划存在以下关键差距：

**1. UI 集成未完成（05-01 核心问题）**
后端错误分类体系（error_dto.rs, classify_error, sync_status.rs）实现完整且代码质量良好，但**前端 UI 组件未接入**：
- `+layout.svelte` 未导入 `isOnline`，未渲染离线 banner
- `+page.svelte` 未导入 `lastSyncedAt`，未渲染 STALE 圆点
- `app.css` 缺少 `.offline-banner`、`.stale-dot`、`.warmup-hint` 样式
- `checkNetworkStatus()` 定义但从未被调用
- `sync_status.rs` 未在 mod.rs 中声明（文件存在但不编译）

这导致 HOME-03（离线可用 + STALE 标识）的**核心用户体验需求未实现**。后端工件完整，但从前端用户视角看，离线功能不存在。

**2. IPC 命令注册（Phases 02-05 遗留问题）**
`commands/mod.rs` 仅声明 `auth` + `settings` 模块，`lib.rs` handler 仅注册 8 个命令。Phase 02-05 创建的 topk/signal/subscription/resource/sync_status 命令共 ~20 个均未注册。这是一个**横跨多个 phase 的系统性问题**，Phase 05 的 SUMMARIES 未检测到此问题（Self-Check 仅做了 grep 验证，未做编译注册验证）。

**3. 需要修复的最小变更集：**
```
commands/mod.rs:  添加 pub mod topk; pub mod signal; pub mod subscription; pub mod resource; pub mod sync_status;
lib.rs handler:   注册所有缺失的 Tauri 命令
+layout.svelte:   导入 isOnline + onMount 调用 checkNetworkStatus + 渲染离线 banner
+page.svelte:     导入 lastSyncedAt + 渲染 STALE 圆点
app.css:          添加 .offline-banner / .stale-dot / .warmup-hint 样式
```

**4. 与 SUMMARIES 的差异：**
所有 5 个 SUMMARIES 的 Self-Check 均为 PASSED，声称"all created/modified files exist"并"cargo check 通过"。但 cargo check 仅验证了已声明模块的编译，未验证模块注册完整性和 UI wiring。此差异说明 SUMMARY 验证方法存在系统性盲区——**grep 验证存在性不等于运行时功能验证**。

---

_Verified: 2026-03-25T12:00:00Z_
_Verifier: gsd-verifier (code inspection)_
