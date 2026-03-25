# Next Actions - 2026-03-24

基于 QA 测试发现，下一步修复与开发计划。

## 映射摘要

| 优先级 | 任务数 | Phase | Plans |
|--------|--------|-------|-------|
| P0 | 3 | Phase 5 | 05-02 |
| P1 | 2 | Phase 5 | 05-03 |
| P2 | 2 | Phase 5 | 05-04 |
| P3 | 1 | Phase 5 | 05-04 |
| 剩余 | 5 | Phase 5 | 05-01 ~ 05-05 |

## 总体策略

按优先级逐级修复：P0 → P1 → P2 → P3，同时推进 Phase 5 剩余任务。

---

## 第一阶段：修复 P0 阻塞问题

### 1. 订阅搜索框缺失
- **Phase:** 5 | **Plan:** 05-02
- **问题**: Subscriptions 页面无搜索组件
- **定位**: 检查 `packages/client/src/pages/Subscriptions.svelte` 及相关组件
- **排查方向**: 搜索组件是否被条件渲染隐藏、是否未导入、zustand store 数据是否正常

### 2. TopK 默认视图缺失
- **Phase:** 5 | **Plan:** 05-02
- **问题**: TopK 打开后无预设视图
- **定位**: 检查 `packages/client/src/pages/TopK.svelte` 及 view store
- **方案**: 在初始化时创建默认视图（Trending、Most Starred 等），参考设计文档

### 3. 资源卡片未显示
- **Phase:** 5 | **Plan:** 05-02
- **问题**: Resources 页面无数据展示
- **定位**: 检查 `packages/client/src/pages/Resources.svelte` 及数据获取逻辑
- **排查方向**: 数据源是否接入、mock 数据是否存在、渲染条件是否正确

---

## 第二阶段：修复 P1 功能缺陷

### 4. "在 GitHub 打开"按钮无响应
- **Phase:** 5 | **Plan:** 05-03
- **问题**: 外部链接点击无反应
- **定位**: 检查 Tauri `shell.openExternal` 调用或 `window.open` 实现
- **排查方向**: CSP 策略、Tauri 权限配置、事件绑定

### 5. Home 页面依赖订阅数据
- **Phase:** 5 | **Plan:** 05-03
- **问题**: 无订阅时无法体验信号聚合
- **方案**: 提供默认数据或引导流程，让新用户也能看到示例内容

---

## 第三阶段：优化 P2 体验问题

### 6. Toast 布局抖动
- **Phase:** 5 | **Plan:** 05-04
- **问题**: Settings 页面 Toast 导致组件位移
- **方案**: 使用 `position: fixed` 或独立 toast 容器，避免影响文档流

### 7. 缺少注销功能
- **Phase:** 5 | **Plan:** 05-04
- **问题**: 无注销入口
- **方案**: 在 Settings 或侧边栏添加注销按钮，调用认证 store 的 logout 方法

---

## 第四阶段：处理 P3 小问题

### 8. favicon.ico 404
- **Phase:** 5 | **Plan:** 05-04
- **问题**: 缺少 favicon 文件
- **方案**: 添加 `src-tauri/icons/icon.ico` 或配置 Vite favicon 路径

---

## Phase 5 剩余任务

| 任务 | Phase | Plan | 状态 |
|------|-------|------|------|
| 离线降级 | 5 | 05-01 | ✓ |
| 错误处理规范化 | 5 | 05-01 | ✓ |
| 性能优化 | 5 | 05-04 | 待执行 |
| 打磨体验 | 5 | 05-04 | 待执行 |
| 打包发布 | 5 | 05-05 | 待执行 |

---

## 执行顺序

```
05-01 (离线/错误) → 05-02 (P0) → 05-03 (P1) → 05-04 (P2/P3/性能) → 05-05 (打包)
```
