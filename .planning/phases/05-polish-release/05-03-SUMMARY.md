---
phase: 05-polish-release
plan: 03
subsystem: ui
tags:
  - bug-fix
  - p1
  - external-links
  - empty-state
  - ux
dependency_graph:
  requires:
    - "05-02"
  provides:
    - "github-external-link-fix"
    - "home-empty-state"
  affects:
    - "topk/+page.svelte"
    - "+page.svelte"
    - "ipc/tauri.ts"
tech_stack:
  added:
    - "@tauri-apps/plugin-shell (frontend IPC)"
  patterns:
    - "openExternal IPC wrapper using @tauri-apps/plugin-shell open()"
    - "Svelte 5 $derived for reactive empty state detection"
    - "onMount + loadSubscriptions for subscription data hydration"
key_files:
  created: []
  modified:
    - path: "apps/desktop-ui/src/lib/ipc/tauri.ts"
      change: "添加 openExternal 函数，使用 @tauri-apps/plugin-shell 的 open()"
    - path: "apps/desktop-ui/src/routes/topk/+page.svelte"
      change: "仓库名链接改为调用 openExternal 打开系统浏览器"
    - path: "apps/desktop-ui/src/routes/+page.svelte"
      change: "添加订阅空状态引导，含导航按钮和动态欢迎语"
decisions:
  - "使用 @tauri-apps/plugin-shell open() 而非自定义 Rust command — 前端直接调用 Tauri 插件 API，减少 IPC 层复杂度"
  - "Home 页面 onMount 加载订阅 — 保证空状态判断基于真实数据，避免闪烁"
  - "空状态引导使用虚线边框 + emoji — 与现有卡片风格一致，视觉区分度适中"
metrics:
  duration: "5min"
  completed: "2026-03-25T04:19:51Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 3
  commits: 2
---

# Phase 05-03: P1 功能缺陷修复 Summary

修复 QA 发现的 2 个 P1 问题：GitHub 外部链接无响应、Home 页面无订阅时不可体验。

## 改动摘要

### Task 1: GitHub 外部链接修复 (QA #4)
- **问题**: TopK 页面点击仓库名链接（`<a target="_blank">`）在 Tauri v2 中无法在系统浏览器打开
- **修复**: 
  - 在 IPC wrapper (`tauri.ts`) 中添加 `openExternal` 函数，使用 `@tauri-apps/plugin-shell` 的 `open()` API
  - TopK 页面仓库名链接改为 onclick 调用 `openExternal(item.html_url)` 
  - 使用 `preventDefault` 阻止默认导航行为
  - 添加 keyboard accessibility 支持（Enter/Space 键）
- **文件**: `tauri.ts`, `topk/+page.svelte`
- **Commit**: `648f921`

### Task 2: Home 页面无订阅引导 (QA #5)
- **问题**: Home 页面在无订阅时仍显示固定的"欢迎回来"内容，新用户无法体验信号聚合功能
- **修复**:
  - 引入 `subscriptions` store，`onMount` 加载订阅数据
  - 使用 `$derived` 计算 `isEmpty` 状态（无订阅且非加载中）
  - 无订阅时展示引导卡片：📡 emoji + 标题 + 描述 + 两个 CTA 按钮（TopK 发现 / 搜索订阅）
  - 有订阅时仍正常展示 3 格导航卡片
  - 欢迎语随订阅状态动态变化
- **文件**: `+page.svelte`
- **Commit**: `87d0bdf`

## 验证结果

- ✅ `openExternal` 函数存在于 IPC wrapper，使用 `@tauri-apps/plugin-shell`
- ✅ TopK 页面仓库名链接调用 `openExternal`
- ✅ Home 页面包含 `empty-state` 类名的引导容器
- ✅ Home 页面导入并调用 `loadSubscriptions`
- ✅ 条件渲染：空状态 vs 导航卡片

## 风险/限制

- `@tauri-apps/plugin-shell` 依赖已存在，无新增依赖
- Home 页面加载时可能短暂显示导航卡片（订阅数据异步加载），后续可加骨架屏优化
- 外部链接打开依赖 Tauri shell plugin 权限配置（当前 `tauri.conf.json` 中 `csp: null`，无限制）

## Self-Check

- ✅ `apps/desktop-ui/src/lib/ipc/tauri.ts` — FOUND (openExternal 函数)
- ✅ `apps/desktop-ui/src/routes/topk/+page.svelte` — FOUND (openExternal 调用)
- ✅ `apps/desktop-ui/src/routes/+page.svelte` — FOUND (empty-state 引导)
- ✅ Commit `648f921` — FOUND
- ✅ Commit `87d0bdf` — FOUND

## Self-Check: PASSED
