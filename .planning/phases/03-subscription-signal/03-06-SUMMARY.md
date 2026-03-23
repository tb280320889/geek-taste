---
phase: 03-subscription-signal
plan: 06
subsystem: runtime-tauri
tags: [ipc, tauri, subscription, signal]
---

# Phase 03 Plan 06 Summary

已完成 Tauri IPC 命令层接入：订阅与信号相关命令全部暴露并注册。

## 完成项

- 新增 `crates/runtime_tauri/src/commands/subscription.rs`：
  - `subscribe` / `unsubscribe` / `pause_subscription` / `list_subscriptions` / `sync_subscriptions`
- 新增 `crates/runtime_tauri/src/commands/signal.rs`：
  - `list_signals` / `list_home_signals` / `ack_signal` / `mark_signal_seen` / `get_unread_counts`
- 更新 `crates/runtime_tauri/src/commands/mod.rs` 导出新命令模块。
- 更新 `apps/desktop-ui/src-tauri/src/lib.rs`，将 10 个新命令注册到 `generate_handler![]`。

## 关键文件

- `crates/runtime_tauri/src/commands/subscription.rs`
- `crates/runtime_tauri/src/commands/signal.rs`
- `crates/runtime_tauri/src/commands/mod.rs`
- `apps/desktop-ui/src-tauri/src/lib.rs`

## 验证

- `rtk cargo check -p runtime_tauri` 通过。
