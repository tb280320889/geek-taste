---
phase: 05-polish-release
plan: 04
subsystem: polish
tags: [ui, toast, logout, favicon, performance]
dependency_graph:
  requires: ["05-02", "05-03"]
  provides: [toast-layout-fix, logout-ui, favicon, sidebar-logout]
  affects: [settings-page, sidebar, app-shell]
tech_stack:
  added: []
  patterns: [fixed-position-toast, keyed-each-blocks]
key_files:
  created:
    - apps/desktop-ui/static/favicon.ico
  modified:
    - apps/desktop-ui/src/routes/settings/+page.svelte
    - apps/desktop-ui/src/lib/components/Sidebar.svelte
    - apps/desktop-ui/src/app.html
decisions:
  - "Toast 使用 position: fixed 而非 CSS animation 仅处理入场，避免引入新依赖"
  - "注销按钮同时放置于 Settings 和 Sidebar，满足 plan 的两处注销需求"
metrics:
  duration: ~8min
  completed: "2026-03-25"
  tasks: 3
  commits: 3
  files_changed: 4
---

# Phase 05-04: P2/P3 + 性能优化 + 打磨 Summary

## One-liner

Toast 固定定位消除布局抖动，Settings/Sidebar 双入口注销，favicon 修复 404。

## Task Results

### Task 1: Toast 布局修复 + 注销功能 (QA #6 + #7) ✅

**Commit:** `e43d5b5`

**改动:**
- Toast 改为 `position: fixed` 定位在右下角，脱离文档流不再引起布局抖动
- 添加 `toastType` 区分 success/error 样式
- 添加 `slideIn` 入场动画
- Settings 页面底部添加"注销"按钮
- `handleLogout` 调用 auth store `logout()` 后 `goto("/onboarding")`

**验证:** grep 确认 position:fixed + logout + 注销 按钮均存在

### Task 2: favicon 修复 (QA #8) ✅

**Commit:** `95727f1`

**改动:**
- 创建 `apps/desktop-ui/static/favicon.ico` (16x16 深色主题 G 图标)
- `app.html` head 中添加 `<link rel="icon" href="%sveltekit.assets%/favicon.ico" />`

**验证:** 文件存在 (1.2K)，app.html 中有 favicon link

### Task 3: 性能优化 + 打磨体验 ✅

**Commit:** `ea2cf4c`

**改动:**
- Sidebar 添加注销按钮（双入口：Settings + Sidebar）
- 审查：已有 each blocks 均使用 keyed iteration `(item.id)`
- 审查：无 legacy `$:` 响应式声明
- svelte-check 验证：**0 errors**，10 warnings（均为预存在未修改文件中的警告）

**验证:** svelte-check 0 errors 通过

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Self-Check: PASSED

- [x] apps/desktop-ui/src/routes/settings/+page.svelte — position:fixed toast + logout button
- [x] apps/desktop-ui/src/lib/stores/auth.ts — logout method (已存在，未修改)
- [x] apps/desktop-ui/static/favicon.ico — 存在
- [x] apps/desktop-ui/src/app.html — favicon link 引用
- [x] apps/desktop-ui/src/lib/components/Sidebar.svelte — logout button
- [x] 3 commits: e43d5b5, 95727f1, ea2cf4c
