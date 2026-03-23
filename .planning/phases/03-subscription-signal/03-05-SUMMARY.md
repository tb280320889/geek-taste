---
phase: 03-subscription-signal
plan: 05
subsystem: application
tags: [subscription, signal, sync, topk]
---

# Phase 03 Plan 05 Summary

已完成 application 层订阅/信号用例编排，并接入 TopK 的 `is_subscribed` 实际查询。

## 完成项

- 新增 `crates/application/src/subscription.rs`：
  - `subscribe` / `unsubscribe` / `pause_subscription` / `list_subscriptions` / `sync_subscriptions`
  - 同步逻辑支持 release/tag 拉取并写入幂等 signal。
- 新增 `crates/application/src/signal.rs`：
  - `list_signals` / `list_home_signals` / `mark_seen` / `ack_signal` / `get_unread_counts`
- 更新 `crates/application/src/topk.rs`：
  - 使用 `list_active_repo_ids` + `HashSet<i64>` 计算 `is_subscribed`。
- 更新 `crates/application/src/lib.rs` 导出新模块。

## 关键文件

- `crates/application/src/subscription.rs`
- `crates/application/src/signal.rs`
- `crates/application/src/topk.rs`
- `crates/application/src/lib.rs`

## 验证

- `rtk cargo check -p application` 通过。
