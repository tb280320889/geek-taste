---
phase: 02-topk
plan: 06
subsystem: ui
tags: [svelte5, runes, topk, ranking, popover, components]

# Dependency graph
requires:
  - phase: 02-topk
    provides: "Plan 05 前端 IPC + Store + Types（tauri.ts + topk.ts + types.ts）"
provides:
  - "TopK 页面完整 UI：ViewSelector 下拉 + FilterPanel 筛选 + RankingList 排名卡片 + SubscribePopover 订阅"
  - "排名变化标识（green +↑N / red -↓N / grey — / hidden for first snapshot）"
  - "Momentum 评分 Tooltip（star/fork/recency 细分维度）"
  - "一键订阅 Popover（预填 STANDARD/12h digest/high notify）"
affects: [phase-03-subscriptions]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Svelte 5 Runes ($props/$state/$derived/$effect) for all new components"
    - "Popover pattern: fixed backdrop + absolute positioned dropdown"
    - "Card-based ranking list with grid layout"
    - "Skeleton loading animation with CSS shimmer"

key-files:
  created:
    - "apps/desktop-ui/src/lib/components/ViewSelector.svelte"
    - "apps/desktop-ui/src/lib/components/FilterPanel.svelte"
    - "apps/desktop-ui/src/lib/components/RankingList.svelte"
    - "apps/desktop-ui/src/lib/components/SubscribePopover.svelte"
  modified:
    - "apps/desktop-ui/src/routes/topk/+page.svelte"

key-decisions:
  - "SubscribePopover 使用 absolute 定位而非 fixed，相对于 RankingList 容器"
  - "FilterPanel 中 Language 选择器硬编码常用语言列表（复用 LanguagePicker 风格）"
  - "ViewSelector 底部提示文案指向筛选面板创建新视图，而非提供独立入口"
  - "排名变化标识逻辑：rank_change > 0 → 绿色 +↑N，< 0 → 红色 -↓N，=== 0 → 灰色 —，=== null → 不显示"

patterns-established:
  - "Popover pattern: button backdrop + absolute dropdown + z-index layering"
  - "Ranking card: flex layout with rank/info/meta/score/change/action columns"
  - "Tooltip on hover: CSS-only with position absolute + display toggle"
  - "Svelte 5 props with TypeScript type annotations inline"

requirements-completed: [TOPK-01, TOPK-02, TOPK-03, TOPK-04, TOPK-05, TOPK-06, TOPK-07, TOPK-08]

# Metrics
duration: 14min
completed: 2026-03-23
---

# Phase 02 Plan 06: 前端 TopK UI Summary

**TopK 排名发现引擎完整 UI：4 个新组件 + 页面重写，包含视图选择器、筛选面板、排名列表（含变化标识和 Tooltip）、一键订阅 Popover**

## Performance

- **Duration:** 14 min
- **Started:** 2026-03-23T08:27:43Z
- **Completed:** 2026-03-23T08:41:25Z
- **Tasks:** 3
- **Files modified:** 5 (4 created, 1 rewritten)

## Accomplishments
- ViewSelector 下拉组件：pinned 视图优先、Pin/Unpin/Delete 操作、ESC 关闭
- FilterPanel 筛选面板：Language 多选标签、排序模式、Min Stars、K 值滑块、Archived/Forks 排除开关
- RankingList 排名列表：排名卡片（序号/名称/语言/Stars）、Momentum 分 + Tooltip、排名变化标识（绿↑/红↓/灰—/首次不显示）、订阅按钮
- SubscribePopover 订阅弹窗：预填 STANDARD/12h/high-notify、可微调 digest 窗口、确认订阅
- TopK 页面重写：替换搜索框为完整排名引擎 UI，auth guard、视图切换、筛选创建、订阅流程

## Task Commits

1. **Task 1: ViewSelector + FilterPanel** - `4d91931` (feat)
2. **Task 2: RankingList + SubscribePopover** - `05b30ab` (feat)
3. **Task 3: 重写 TopK 页面** - `e341077` (feat)

## Files Created/Modified
- `apps/desktop-ui/src/lib/components/ViewSelector.svelte` - 视图下拉选择器（Pin/Unpin/Delete）
- `apps/desktop-ui/src/lib/components/FilterPanel.svelte` - 筛选条件面板（language/mode/stars/k/toggles）
- `apps/desktop-ui/src/lib/components/RankingList.svelte` - 排名列表（卡片 + 变化标识 + Tooltip + 订阅按钮）
- `apps/desktop-ui/src/lib/components/SubscribePopover.svelte` - 一键订阅弹窗（预填 + 可微调）
- `apps/desktop-ui/src/routes/topk/+page.svelte` - TopK 页面（完整排名引擎 UI）

## Decisions Made
- SubscribePopover 使用 absolute 定位而非 fixed，相对于 RankingList 容器，更紧凑
- FilterPanel 中 Language 选择器硬编码 10 种常用语言，复用 LanguagePicker 的 toggle-tag 样式
- ViewSelector 底部文字提示用户通过筛选面板创建新视图，无需独立「创建视图」按钮
- 排名变化标识：rank_change > 0 → 绿色 +↑N，< 0 → 红色 -↓N，=== 0 → 灰色 —，null → 不显示（首次快照）
- 订阅确认当前仅关闭 Popover，实际订阅逻辑由 Phase 3 实现

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `cargo check -p desktop-ui-tauri` 失败：`icons/icon.ico` not found。这是预存问题（icons 目录缺失），与本次前端改动无关。TypeScript 编译全部通过。

## Next Phase Readiness
- TopK 排名发现引擎 UI 完整交付，可直接使用
- 订阅 Popover 已就绪，Phase 3 只需接入实际订阅 IPC 调用
- 所有组件遵循 Svelte 5 Runes 模式，与项目约定一致

---
*Phase: 02-topk*
*Completed: 2026-03-23*
