# Phase 2: 数据层与 TopK 发现引擎 - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 02-topk
**Areas discussed:** RankingView 交互模型, 快照自动化策略, 评分可视化层级, 一键订阅确认流

---

## RankingView 交互模型

### 创建方式

| Option | Description | Selected |
|--------|-------------|----------|
| 先筛选后保存 | 页面顶部筛选器面板，配置好后点击「保存为视图」 | |
| 模板 + 微调 | 3-5 个预设模板，选中即用，可微调后另存 | |
| 两者都支持 | 预设模板 + 从零自定义两种路径并存 | ✓ |

**User's choice:** 两者都支持
**Notes:** 无额外说明

### 编辑方式

| Option | Description | Selected |
|--------|-------------|----------|
| 原地编辑 + 保存/另存 | 直接修改筛选条件，提供「保存」覆盖和「另存为」新建 | ✓ |
| 实时预览 + 手动持久化 | 修改后结果实时刷新但不自动保存，需手动点击保存 | |

**User's choice:** 原地编辑 + 保存/另存
**Notes:** 无额外说明

### 视图切换

| Option | Description | Selected |
|--------|-------------|----------|
| Sidebar 展开子列表 | 左侧 Sidebar TopK 节点展开显示所有已保存视图 | |
| 页面内 Tab 栏 | TopK 页面顶部 Tab 栏展示已保存视图 +「+」新建按钮 | |
| 下拉选择器 | 页面顶部下拉选择器显示当前视图名，展开为所有视图列表 | ✓ |

**User's choice:** 下拉选择器
**Notes:** 无额外说明

---

## 快照自动化策略

### 触发时机

| Option | Description | Selected |
|--------|-------------|----------|
| 后台定时 12h 全量 | tokio-cron-scheduler 每 12h 为所有视图创建快照 | |
| 用户打开时触发 | 仅在用户打开视图且距上次 >12h 时触发 | |
| 定时 + 打开时补充 | 后台 12h 定时为主 + 用户打开时检查补充 | ✓ |

**User's choice:** 定时 + 打开时补充
**Notes:** 无额外说明

### 暖机策略

| Option | Description | Selected |
|--------|-------------|----------|
| 创建时立即拍一张 | 创建视图后立即触发快照，之后进入 12h 定时周期 | ✓ |
| 等待定时器自然触发 | 创建视图后显示「等待首次快照...」，12h 后才出现数据 | |

**User's choice:** 创建时立即拍一张
**Notes:** 无额外说明

---

## 评分可视化层级

### 评分展示

| Option | Description | Selected |
|--------|-------------|----------|
| 仅综合分 + 排序标签 | 每项仅显示综合分 + 排序模式标签 | |
| 综合分 + 悬停展开细分 | 默认展示综合分，悬停/点击展开显示三个维度贡献 | ✓ |
| 始终展示细分维度 | 每项用 mini bar chart 或进度条展示三个维度 | |

**User's choice:** 综合分 + 悬停展开细分
**Notes:** 无额外说明

### 排名变化标识

| Option | Description | Selected |
|--------|-------------|----------|
| 排名变化箭头 + 颜色 | 上升标绿 +↑N，下降标红 -↓N，不变标灰 — | ✓ |
| 仅排名数字，无变化标注 | 排名位置就是唯一标识，不额外标注变化 | |
| 趋势指示器（圆点） | 排名数字旁用小圆点表示变化趋势 | |

**User's choice:** 排名变化箭头 + 颜色
**Notes:** 无额外说明

---

## 一键订阅确认流

### 确认流程

| Option | Description | Selected |
|--------|-------------|----------|
| 弹出预填面板（推荐） | 小型 Popover，预填默认设置，用户可微调后确认 | ✓ |
| 直接订阅 + toast 撤销 | 点击即订阅，通过 toast 显示 + 撤销按钮 | |
| 跳转订阅页完整设置 | 跳转到 Subscriptions 页面打开完整设置表单 | |

**User's choice:** 弹出预填面板（推荐）
**Notes:** 无额外说明

### 默认设置来源

| Option | Description | Selected |
|--------|-------------|----------|
| 固定默认值 | 固定：STANDARD/12h digest/HIGH 立即通知 | ✓ |
| 用户可自定义默认偏好 | Settings 页面增加「订阅默认偏好」配置 | |

**User's choice:** 固定默认值
**Notes:** 无额外说明

---

## Agent's Discretion

无 — 所有灰区均由用户明确选择。

## Deferred Ideas

- 用户自定义 Momentum 权重（v2）
- RankingView 导出/分享功能
- 跨视图批量快照触发
- GitHub Search API 1000 条上限外的候选发现
