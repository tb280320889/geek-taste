---
phase: 05-polish-release
plan: 05
subsystem: infra
tags: [tauri, bundle, updater, packaging, release]

# Dependency graph
requires:
  - phase: 05-04
    provides: "P2/P3 polish (toast layout, logout, favicon)"
provides:
  - "Tauri v2 打包配置验证 (bundle active, targets, icons)"
  - "Updater 插件注册 + 端点占位符"
  - "custom-protocol feature flags"
  - "打包发布准备就绪 (待人工验证)"
affects:
  - "Phase 6: QA 整合"

# Tech tracking
tech-stack:
  added: [tauri-plugin-updater]
  patterns: ["Updater 端点使用 URL 模板占位符，实际部署时替换", "Plugin init 与 tauri.conf.json plugins.updater 配置配合"]

key-files:
  created: []
  modified:
    - apps/desktop-ui/src-tauri/tauri.conf.json
    - apps/desktop-ui/src-tauri/Cargo.toml
    - apps/desktop-ui/src-tauri/src/lib.rs

key-decisions:
  - "bundle.targets 保持 'all' — 覆盖全平台安装包格式，CI 时再按需限制"
  - "Updater 端点使用占位符模板 — 实际部署时替换为签名密钥和服务器地址"
  - "custom-protocol feature 按 Tauri v2 标准配置 — 确保打包时前端 dist 正确嵌入"

patterns-established:
  - "Tauri v2 plugin 注册: Cargo workspace dep → desktop-ui dep → lib.rs .plugin(init()) → tauri.conf.json plugins config"

requirements-completed: [HOME-03]

# Metrics
duration: 10min
completed: 2026-03-25
---

# Phase 05 Plan 05: 打包发布 + 自动更新 Summary

**Tauri v2 打包配置验证 + updater 插件注册 + 端点占位符，打包发布准备就绪**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-25
- **Completed:** 2026-03-25 (auto tasks — checkpoint pending)
- **Tasks:** 2 auto tasks completed / 2 total auto tasks + 1 checkpoint pending
- **Files modified:** 3

## Accomplishments
- 验证 Tauri 打包配置：bundle.active=true, targets=all, icons 完整 (32x32/128x128/icns/ico)
- 添加 updater 插件端点占位符到 tauri.conf.json (plugins.updater)
- 注册 tauri_plugin_updater::init() 到 lib.rs
- 添加 custom-protocol feature flags 确保打包时前端嵌入正确
- 添加 tauri-plugin-updater 依赖到 desktop-ui Cargo.toml

## Task Commits

1. **Task 1: Tauri 打包配置** - `7ec72ba` (feat)
   - tauri.conf.json: 添加 plugins.updater 端点占位符
   - Cargo.toml (desktop-ui): 添加 tauri-plugin-updater dep + feature flags
2. **Task 2: 自动更新 + 发布验证** - `0ac8b1f` (feat)
   - lib.rs: 注册 tauri_plugin_updater::init()

## Files Created/Modified
- `apps/desktop-ui/src-tauri/tauri.conf.json` - 添加 plugins.updater 端点占位符和 pubkey 占位符
- `apps/desktop-ui/src-tauri/Cargo.toml` - 添加 tauri-plugin-updater 依赖 + custom-protocol feature flags
- `apps/desktop-ui/src-tauri/src/lib.rs` - 注册 updater 插件

## Decisions Made
- bundle.targets 保持 "all" — 覆盖全平台安装包格式 (msi/dmg/appimage)
- Updater 端点使用 URL 模板占位符 — 实际部署时替换为签名密钥和服务器地址
- custom-protocol feature 按 Tauri v2 标准配置

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] 注册 updater 插件到 lib.rs**
- **Found during:** Task 2 (自动更新 + 发布验证)
- **Issue:** tauri.conf.json 有 updater 配置但 lib.rs 未注册插件，updater 功能不会生效
- **Fix:** 在 lib.rs 中添加 `.plugin(tauri_plugin_updater::init())`
- **Files modified:** apps/desktop-ui/src-tauri/src/lib.rs
- **Verification:** 插件注册与其他插件 (shell/dialog/store/notification) 模式一致
- **Committed in:** 0ac8b1f (Task 2 commit)

**2. [Rule 3 - Path Deviation] 计划路径与实际 monorepo 路径不一致**
- **Found during:** Task 1 pre-execution review
- **Issue:** PLAN.md 引用 `src-tauri/` 路径，但实际文件在 `apps/desktop-ui/src-tauri/` (monorepo 结构)
- **Fix:** 使用实际路径读取和修改文件
- **Impact:** 无功能影响，仅路径映射差异

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 path deviation)
**Impact on plan:** Updater 插件注册为关键功能修复。路径偏差无功能影响。

## Checkpoint Status

**checkpoint:human-verify** — PENDING

需要人工验证:
1. 运行完整 QA 回归测试，确认所有 P0-P3 问题已修复
2. 测试离线流程：断网 → 打开应用 → 验证缓存 + STALE 标识
3. 测试安装包：安装 → 首次启动 → 认证 → 全功能走查
4. 确认 Home 页面在有/无订阅时都正常展示

## Next Phase Readiness
- 打包配置完整，可执行 `cargo tauri build` 产出安装包
- Updater 插件已注册，端点为占位符，实际部署时替换
- 人工验证 checkpoint 通过后可进入 Phase 6

---
*Phase: 05-polish-release*
*Completed: 2026-03-25 (auto tasks — checkpoint pending)*
