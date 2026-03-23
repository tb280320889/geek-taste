---
phase: 03-subscription-signal
plan: 09
subsystem: home-signal-ui
tags: [home, signal, topk, subscribe]
---

# Phase 03 Plan 09 Summary

已完成 Home 信号流与订阅联动 UI：新增 SignalCard，接入 Home 页面，并打通 TopK 订阅状态展示。

## 完成项

- 新增 `apps/desktop-ui/src/lib/components/SignalCard.svelte`：
  - 优先级色条、信号类型、相对时间、`markSeen` 与 `ack` 交互。
- 重写 `apps/desktop-ui/src/routes/+page.svelte`：
  - 展示 Home 信号流、未读计数与空状态引导。
- 更新 `apps/desktop-ui/src/lib/components/SubscribePopover.svelte`：
  - 订阅确认接入实际 `addSubscription` 流程。
- 更新 `apps/desktop-ui/src/lib/components/RankingList.svelte` 与 `apps/desktop-ui/src/routes/topk/+page.svelte`：
  - 订阅状态改为从 subscriptions store 动态读取，TopK 订阅后可实时反映。

## 关键文件

- `apps/desktop-ui/src/lib/components/SignalCard.svelte`
- `apps/desktop-ui/src/routes/+page.svelte`
- `apps/desktop-ui/src/lib/components/SubscribePopover.svelte`
- `apps/desktop-ui/src/lib/components/RankingList.svelte`
- `apps/desktop-ui/src/routes/topk/+page.svelte`

## 验证

- `rtk pnpm exec svelte-check --tsconfig ./tsconfig.json` 通过（0 errors, 7 warnings；warnings 为既有项）。
