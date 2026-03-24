---
phase: 05-polish-release
plan: 01
subsystem: offline-error-handling
tags:
  - offline
  - error-handling
  - momentum
  - network-status
dependency_graph:
  requires: []
  provides:
    - HOME-03
  affects:
    - crates/shared_contracts
    - crates/runtime_tauri
    - crates/persistence_sqlite
    - crates/application
    - apps/desktop-ui
tech_stack:
  added:
    - "error_dto.rs — AppErrorDto + ErrorKind 错误分类类型"
    - "network.ts — Svelte store 网络状态管理"
    - "sync_status.rs — Tauri 命令获取同步状态"
  patterns:
    - "错误分类模式: 原始字符串 → ErrorKind enum"
    - "网络状态检测: IPC 错误 → isOnline store → UI banner"
    - "STALE 检测: lastSyncedAt + 阈值 → UI 圆点标识"
    - "暖机降级: 无快照 → UPDATED_DESC 排序 + warmup 标记"
key_files:
  created:
    - crates/shared_contracts/src/error_dto.rs
    - crates/runtime_tauri/src/commands/sync_status.rs
    - apps/desktop-ui/src/lib/stores/network.ts
  modified:
    - crates/shared_contracts/src/lib.rs
    - crates/shared_contracts/src/ranking_dto.rs
    - crates/runtime_tauri/src/commands/helpers.rs
    - crates/runtime_tauri/src/commands/mod.rs
    - crates/runtime_tauri/src/commands/topk.rs
    - crates/persistence_sqlite/src/ranking_repository.rs
    - crates/persistence_sqlite/src/signal_repository.rs
    - crates/application/src/topk.rs
    - apps/desktop-ui/src-tauri/src/lib.rs
    - apps/desktop-ui/src/lib/types.ts
    - apps/desktop-ui/src/lib/ipc/tauri.ts
    - apps/desktop-ui/src/lib/stores/topk.ts
    - apps/desktop-ui/src/lib/stores/signals.ts
    - apps/desktop-ui/src/lib/stores/subscriptions.ts
    - apps/desktop-ui/src/routes/+layout.svelte
    - apps/desktop-ui/src/routes/+page.svelte
    - apps/desktop-ui/src/routes/topk/+page.svelte
    - apps/desktop-ui/src/app.css
decisions:
  - "RankingResultDto 包装 warmup 标记 — 保持 execute_ranking 返回结构化结果而非裸 Vec"
  - "isStale 使用 subscribe 读取值 — 简单直接，避免引入复杂 derived store"
  - "离线 banner dismiss 设 isOnline=true — 允许用户临时隐藏提示，不阻塞使用"
  - "暖机检测: prev_snapshot.is_none() && Momentum 模式 — 精确匹配暖机场景"
metrics:
  duration: "—"
  completed: "2026-03-24"
---

# Phase 05-01: 离线降级、错误处理与 Momentum 暖机 — Summary

实现 HOME-03 (离线可用 + STALE 标识)，统一错误处理模式，添加 Momentum 暖机降级。

## 改动摘要

### Task 1: AppErrorDto + classify_error + sync_status

**后端错误分类体系：**
- 新建 `error_dto.rs`：定义 `ErrorKind`（5 种分类：AUTH_EXPIRED / NETWORK_ERROR / RATE_LIMITED / NOT_FOUND / INTERNAL）和 `AppErrorDto` 结构体
- `helpers.rs` 添加 `classify_error` 函数：从原始错误字符串模式匹配到 ErrorKind
- `persistence_sqlite` 添加 `get_last_snapshot_time` / `get_last_signal_time` 查询函数
- 新建 `sync_status.rs`：`get_sync_status` Tauri 命令返回各数据源最后同步时间

### Task 2: 前端网络状态 + STALE UI + 离线 banner

**网络状态管理：**
- 新建 `network.ts` store：`isOnline`、`lastSyncedAt`、`isStale()`、`handleIpcError()`、`checkNetworkStatus()`
- `tauri.ts` 添加 `getSyncStatus` IPC 函数
- `+layout.svelte` 添加离线 banner（琥珀色提示条，可 dismiss）
- `+page.svelte` 添加 STALE 琥珀色圆点（Today 标题旁 + TopK 快捷链接旁）
- `app.css` 添加 `.offline-banner`、`.stale-dot`、`.warmup-hint` 全局样式

### Task 3: Momentum 暖机降级 + 错误处理统一

**暖机降级：**
- `ranking_dto.rs` 添加 `RankingResultDto { items, warmup }` 结构
- `application/topk.rs` 的 `execute_ranking` 返回 `RankingResultDto`，在 Momentum 模式无快照时设置 `warmup=true`
- `topk.ts` store 添加 `topkWarmup` store，处理新返回格式
- `topk/+page.svelte` 添加暖机提示条

**统一错误处理：**
- `topk.ts`、`signals.ts`、`subscriptions.ts` 所有 catch 块调用 `handleIpcError`

## Deviations from Plan

None — plan executed exactly as written.

## 验证结果

- ✅ cargo check: 0 errors, 0 warnings (workspace crates)
- ✅ cargo test: 78 passed, 0 failed
- ✅ svelte-check: 0 errors, 10 warnings (pre-existing in ResourceFilters/resources)

## Self-Check: PASSED

All created/modified files exist. All commits verified.
