---
phase: 03-subscription-signal
plan: 04
subsystem: shared-contracts
tags: [dto, subscription, signal]
---

# Phase 03 Plan 04 Summary

已完成订阅与信号 DTO 契约层：新增 `subscription_dto` 与 `signal_dto`，并在 shared_contracts 对外导出。

## 完成项

- 新增 `SubscriptionDto` / `CreateSubscriptionRequest` / `UpdateSubscriptionRequest` / `SubscriptionRowDto`。
- 新增 `SignalDto` / `UnreadCountsDto`。
- 实现 `From<Subscription>` 与 `From<Signal>` 转换。
- 更新 `crates/shared_contracts/src/lib.rs` 导出新模块。

## 关键文件

- `crates/shared_contracts/src/subscription_dto.rs`
- `crates/shared_contracts/src/signal_dto.rs`
- `crates/shared_contracts/src/lib.rs`

## 验证

- `rtk cargo check -p shared_contracts` 通过。
