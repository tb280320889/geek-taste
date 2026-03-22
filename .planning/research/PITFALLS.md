# 域陷阱研究

**Domain:** Tauri v2 + SvelteKit + Rust + SQLite + GitHub API 桌面技术雷达工作台
**Researched:** 2026-03-22
**Confidence:** HIGH（基于官方文档、社区 issue、真实项目经验）

---

## 关键陷阱

### 陷阱 1: Tauri v2 ACL 权限遗漏——"一切正常但功能全挂"

**表现：**
前端 `invoke()` 调用全部返回 `Command not allowed by ACL` 或 `IPC Connection Error: Command not found`。应用能启动，但所有 Rust 后端交互静默失败。

**根因：**
Tauri v2 废弃了 v1 的 `allowlist`，引入了基于 Capabilities 的 ACL 系统。默认状态下，前端与后端之间的所有通道被完全切断。必须在 `src-tauri/capabilities/default.json` 中显式授权每个命令和插件权限。

**预防策略：**
- 项目初始化时立即创建 `capabilities/default.json`，为所有自定义命令和插件（`plugin-sql`、`plugin-store`、`plugin-shell`、`plugin-notification`、`plugin-updater`）显式声明权限
- 前端使用 `@tauri-apps/plugin-*` 替代旧的 `@tauri-apps/api` 子路径导入
- 每添加一个新插件，同步更新 capability 文件

**预警信号：**
- 前端调用 invoke 后 Promise 悬挂不 resolve/reject
- DevTools 控制台出现 `not allowed` 错误
- 插件安装了但完全没效果

**对应阶段：** Phase 1（项目脚手架搭建阶段必须完成）

---

### 陷阱 2: Tauri IPC 大数据序列化瓶颈——"同步到 50 个 repo 后 UI 卡死"

**表现：**
当 Rust 后端向前端传递大量数据（批量 repo 元数据、完整 release 列表）时，IPC 调用耗时从毫秒级飙到数百毫秒甚至秒级。Windows 平台尤其严重（10MB 数据约 200ms）。

**根因：**
Tauri IPC 默认使用 JSON 序列化/反序列化。虽然 v2 支持 Raw Payload 模式绕过 JSON，但默认行为仍是序列化。WebView 的内存限制（约 2GB）也会在传输超大 payload 时产生问题。

**预防策略：**
- 保持 Rust 端为数据主权威，前端只获取当前视图所需的最小数据集
- 分页查询——永远不要一次性把整个 SQLite 结果集通过 IPC 发送
- 考虑对大批量数据使用 Raw Request/Response 模式（v2 新特性）或 `convertFileSrc` 读取文件
- 对 TopK 快照等只读数据，从 SQLite 查询后在 Rust 端缓存，按需分页传递

**预警信号：**
- invoke 调用在 DevTools Network 面板中耗时异常长
- 大列表渲染时 UI 首次可交互时间超过 3 秒
- Windows 上比 macOS 明显更慢

**对应阶段：** Phase 2（数据层与同步引擎开发阶段）

---

### 陷阱 3: SvelteKit static-adapter 误用 server 端功能——"本地开发正常，构建崩溃"

**表现：**
项目中存在 `+page.server.js` 或 `+layout.server.js` 文件。开发模式下一切正常，但 `adapter-static` 构建时报 `all routes must be fully prerenderable` 或部署后出现 500 错误（找不到 `__data.json`）。

**根因：**
Tauri 不支持 server-based frontend。`adapter-static` 要求所有页面可预渲染或使用 SPA fallback 模式。任何 `.server.js` 文件的存在都需要 server runtime 执行，在纯静态环境中无法工作。

**预防策略：**
- 根 `+layout.ts` 必须导出 `export const ssr = false` 和 `export const prerender = true`
- **绝不创建** `+page.server.js` 或 `+layout.server.js` 文件
- 所有数据获取通过 Tauri IPC（`invoke`）完成，不使用 SvelteKit 的 `load` 函数做服务端请求
- `svelte.config.js`（不是 `.ts`）中配置 `adapter-static` 并设置 `fallback: 'index.html'`

**预警信号：**
- 构建输出中出现 "dynamic routes" 警告
- 本地 `npm run dev` 正常但 `npm run build` 失败
- 某些路由在刷新后 404

**对应阶段：** Phase 1（项目脚手架搭建阶段）

---

### 陷阱 4: SQLite 并发写入死锁——"同步时 UI 冻结或数据损坏"

**表现：**
后台同步任务写入 SQLite 时，用户操作触发的查询被阻塞。更糟的情况是 `SQLITE_BUSY` 错误或数据损坏。Tauri SQL 插件曾因使用 `Mutex` 而导致 SELECT 查询也被串行化执行。

**根因：**
SQLite 只允许一个写入者。默认 journal 模式下，写入会阻塞所有读取。即使使用 WAL 模式，多个并发写入仍会产生锁竞争。Tauri 官方 SQL 插件在 v2 中曾有已知的并发执行 bug（issue #2254）。

**预防策略：**
- **启用 WAL 模式**：`PRAGMA journal_mode=WAL;` — 允许读写并发
- **分离读写连接池**：写连接限 1 个，读连接可多个（参考 `tauri-plugin-sqlite` 的 Split Connection Architecture）
- **设置 busy_timeout**：`PRAGMA busy_timeout=5000;` — 让 SQLite 在遇到锁时等待而非立即报错
- **写操作使用 `BEGIN IMMEDIATE`** 代替 `BEGIN DEFERRED`，提前获取写锁避免死锁
- 同步引擎的写操作在独立的 tokio task 中执行，不阻塞 UI 线程
- 仔细评估是否使用官方 `tauri-plugin-sql` 还是社区的 `tauri-plugin-sqlite`（后者有更好的读写分离）

**预警信号：**
- 同步期间用户操作无响应
- 偶发 "database is locked" 错误
- 同一窗口的查询顺序与预期不符

**对应阶段：** Phase 2（数据层设计阶段必须解决）

---

### 陷阱 5: SQLite migration 中断导致数据损坏——"升级后数据全丢"

**表现：**
应用更新后 migration 执行失败，数据库处于半完成状态。下次启动时旧 migration 不会重跑，应用加载了损坏的 schema。

**根因：**
SQLite 的 DDL 操作不是完全事务性的（尤其是 `ALTER TABLE`）。如果 migration 过程中应用崩溃或被强制关闭，数据库可能处于不一致状态。Tauri 插件的 migration 在 `load()` 时执行而非启动时，执行时机容易被误解。

**预防策略：**
- 每个 migration 文件只做单一变更（一个 `CREATE TABLE` 或一个 `ALTER TABLE`）
- migration 必须幂等：使用 `CREATE TABLE IF NOT EXISTS`、`CREATE INDEX IF NOT EXISTS`
- 在 `tauri.conf.json` 中配置 `"preload": ["sqlite:geek_taste.db"]` 确保 migration 在前端 load 之前执行
- 维护 migration 版本号的严格线性序列，禁止回溯修改已执行的 migration
- 添加 migration 失败的集成测试（docs/07 要求）

**预警信号：**
- migration 文件名不连续或有跳跃
- 开发过程中频繁删除重建数据库（说明 migration 不可靠）
- 没有对 migration 失败路径的测试

**对应阶段：** Phase 2（数据层设计阶段）

---

### 陷阱 6: GitHub API rate budget 互相踩踏——"TopK 刷新吃掉了订阅同步的配额"

**表现：**
用户打开 TopK 视图触发大量 Search API 请求（30 req/min 限额），同时后台订阅同步也在消耗 core API 配额（5000 req/hour）。两路请求互相竞争，导致一方触发 429/403 rate limit。

**根因：**
GitHub 的 core API 和 search API 有独立的 rate limit。如果不做预算隔离，高优先级的订阅同步可能被低优先级的资源雷达刷新耗尽配额。Secondary rate limits（并发限制、高频 mutation 限制）更隐蔽，不会在 `X-RateLimit-Remaining` 中体现。

**预防策略：**
- **预算池隔离**：Search budget（30/min）和 Core budget（5000/h）必须分池管理
- **优先级队列**：订阅同步 > 用户主动刷新 > 资源雷达后台刷新
- **ETag 条件请求**：所有 GET 请求必须携带 `If-None-Match`，304 响应不计入 rate limit
- **指数退避**：遇到 429/403 时，读取 `X-RateLimit-Reset` 或 `Retry-After` header，按指数退避重试
- **本地 TTL 缓存**：Search 类端点结果必须有本地 TTL，禁止每次 UI 操作都打 API
- 实现统一的 `RateLimitScheduler` 组件，所有 API 请求必须通过它

**预警信号：**
- 日志中频繁出现 403 或 429 状态码
- 同步任务在整点附近突然变慢（rate limit 重置时间）
- 用户反馈"刷新后没有新数据"

**对应阶段：** Phase 2（同步引擎开发阶段）

---

### 陷阱 7: GitHub Search API 的 1000 结果上限幻觉——"TopK 榜单永远只有 1000 条"

**表现：**
试图翻页获取超过 1000 条搜索结果，API 返回空数据或报错。排行榜在不同时间刷新时结果大幅漂移，用户失去信任。

**根因：**
GitHub Search API 结果上限为 1000 条（10 页 × 100 条），且最多穿透 4000 个匹配仓库。这意味着复杂过滤器组合可能遗漏结果。此外，Search API 的排序不是完全确定性的，相同查询在不同时间可能返回不同顺序。

**预防策略（对应 AP-2）：**
- TopK 必须基于 `RankingView + RankingSnapshot` 模型——快照可复算、排序公开
- 接受 1000 条上限，设计评分公式在本地对结果进行二次排序和过滤
- 不要试图绕过上限——改为优化查询模板减少不必要的结果
- 快照之间对比变化，而非依赖绝对排名

**预警信号：**
- 代码中有 `page > 10` 的翻页逻辑
- 排行榜没有"上次快照时间"的 UI 指示
- 用户反映"每次看排行榜结果都不一样"

**对应阶段：** Phase 2（TopK 引擎开发阶段）

---

### 陷阱 8: 在 async 上下文中使用 std::sync::Mutex 并跨越 .await——"偶发死锁，无法复现"

**表现：**
应用偶发性完全冻结。不是每次发生，但长时间运行后概率增加。日志显示某些任务永远不完成。Tokio worker thread 被阻塞。

**根因：**
`std::sync::MutexGuard` 持有期间跨越 `.await` 点，会阻塞整个 Tokio worker thread。其他 task 无法在该线程上执行，可能导致死锁。特别是当 `spawn_blocking` 中的同步任务与 `spawn` 中的异步任务共享同一个 `std::sync::Mutex` 时，经典的单 mutex 死锁就会发生。

**预防策略：**
- **默认使用 `tokio::sync::Mutex`**（与 Tokio 文档推荐相反，但更安全）
- 如果必须用 `std::sync::Mutex`，绝不跨越 `.await` 点持有 guard
- 用 `spawn_blocking` 包裹长时间运行的同步操作（如 SQLite 写入）
- 使用 Clippy 的 `await_holding_lock` lint 检测问题
- 对共享状态优先考虑 `Arc<tokio::sync::RwLock<T>>` 而非 `Arc<std::sync::Mutex<T>>`

**预警信号：**
- 应用在长时间运行后冻结
- `tokio-console` 显示某 task 永远 pending
- 8 个以上的并发任务时更容易触发

**对应阶段：** Phase 2（Rust 后端架构阶段）

---

### 陷阱 9: Tauri v2 插件系统碎片化——"装了插件但 API 路径变了"

**表现：**
从 v1 迁移或参考旧教程时，使用 `@tauri-apps/api/fs` 等导入路径，得到 `module not found` 错误。

**根因：**
Tauri v2 将核心功能拆分为独立插件（`@tauri-apps/plugin-fs`、`@tauri-apps/plugin-shell`、`@tauri-apps/plugin-dialog` 等）。旧的统一 `@tauri-apps/api` 包不再包含这些子模块。每个插件需要同时在 Rust 端（`Cargo.toml` + `main.rs` 注册）和 JS 端（`npm install` + capability 配置）安装。

**预防策略：**
- 所有 Tauri v2 开发参考官方 v2 文档，不参考 v1 教程
- 建立插件安装 checklist：① `npm install @tauri-apps/plugin-xxx` → ② `Cargo.toml` 添加依赖 → ③ `main.rs` 注册插件 → ④ `capabilities/default.json` 授权
- 在项目 README 或内部文档中维护当前使用的插件清单

**预警信号：**
- 搜索解决方案时找到的都是 v1 的答案
- 某个 API 在 DevTools 中可调用但返回 "not found"
- Rust 和 JS 端的插件版本不匹配

**对应阶段：** Phase 1（项目脚手架阶段）

---

### 陷阱 10: macOS 公证（Notarization）流程卡住发布——"用户下载后提示应用已损坏"

**表现：**
应用在本地运行正常，打包后发给用户，macOS 显示 "App is damaged and can't be opened" 或 Gatekeeper 直接拦截。Windows 上 SmartScreen 警告"无法验证发布者"。

**根因：**
macOS 要求应用经过 Apple 公证（Notarization）才能在 Gatekeeper 下运行。公证需要 Apple Developer Program（$99/年）、Developer ID 证书、以及通过 `xcrun notarytool` 提交扫描。Windows 自 2023 年起要求 OV 证书存储在 HSM 中（不能导出为文件），通常需要 Azure Key Vault。此外 Tauri updater 的签名密钥对是独立于 OS 签名的另一套。

**预防策略：**
- **尽早配置**：不要等到发布前才处理签名。在 Phase 1 就建立 CI/CD 签名流程
- macOS：注册 Apple Developer → 创建 Developer ID Application 证书 → 配置 `tauri.conf.json` 的 `bundle.macOS.signingIdentity` → CI 中配置 `APPLE_CERTIFICATE`、`APPLE_ID`、`APPLE_PASSWORD` secrets
- Windows：考虑 Azure Key Vault + `relic` 签名工具
- Tauri Updater：`tauri signer generate` 生成独立密钥对，`TAURI_SIGNING_PRIVATE_KEY` 存入 CI secrets
- 在 `tauri.conf.json` 中设置 `bundle.createUpdaterArtifacts: true`

**预警信号：**
- CI/CD 中没有签名步骤
- 本地构建的 `.dmg` 在其他机器上打不开
- 没有 Apple Developer 账号或没有计划注册

**对应阶段：** Phase 1-2（CI/CD 与发布管道搭建阶段）

---

### 陷阱 11: WebView 平台差异导致 CSS/JS 行为不一致——"macOS 上完美，Windows 上错位"

**表现：**
macOS 使用 WebKit（WKWebView），Windows 使用 WebView2（Chromium），Linux 使用 WebKitGTK。同一个 CSS 布局或 JS API 在不同平台表现不同。特别是 CSS backdrop-filter、scrollbar 样式、某些 Web API。

**根因：**
Tauri 使用系统原生 WebView 而非捆绑 Chromium。WebKit 和 Chromium 的 CSS 实现存在差异，某些实验性 API 仅在特定引擎中可用。

**预防策略：**
- 在两个目标平台（macOS + Windows）上定期手动测试
- 避免使用 `-webkit-` 前缀的实验性 CSS 特性，除非确认两平台都支持
- 为关键布局准备平台特定的 CSS fallback
- 使用 `@supports` 检测特性可用性

**预警信号：**
- 只在一个平台上开发和测试
- 使用了大量 CSS `backdrop-filter`、`scrollbar-*` 自定义
- 依赖某些 Web API（如 `FileSystemAccessAPI`）而没有 feature detection

**对应阶段：** 贯穿所有 UI 开发阶段

---

### 陷阱 12: 条件请求（ETag）实现不完整——"明明有缓存却还在消耗 rate limit"

**表现：**
实现了缓存逻辑，但 GitHub API 请求仍然消耗 rate limit 配额。304 响应比例远低于预期。

**根因：**
GitHub 的条件请求要求同时满足：① 使用 `Authorization` header 认证；② 从之前的响应中提取 `ETag` 值；③ 在后续请求中通过 `If-None-Match` header 发送。缺少任一环节，请求都会消耗配额。此外，Search API 的 ETag 行为与 core API 不同，需要设置本地 TTL。

**预防策略：**
- 为每个 GitHub API 端点实现 ETag 存储（存入 SQLite `api_cache` 表，包含 endpoint URL、ETag、Last-Modified、缓存数据、TTL）
- 所有 GET 请求自动携带 `If-None-Match` header
- 对 304 响应直接使用本地缓存，不重新解析
- Search API 端点额外设置本地 TTL（建议 5-15 分钟）
- 监控并记录 304 hit rate（docs/07 要求的运行指标）

**预警信号：**
- `X-RateLimit-Remaining` 下降速度与请求频率成正比（没有 304 缓解）
- 日志中没有 304 响应记录
- 缓存表为空或 ETag 字段为空

**对应阶段：** Phase 2（同步引擎开发阶段）

---

### 陷阱 13: 阻塞 Tokio 主线程——"同步任务执行时整个应用无响应"

**表现：**
当执行 SQLite 批量写入、GitHub API 响应解析等 CPU 密集型操作时，整个应用 UI 冻结。窗口无法拖动、按钮无响应。

**根因：**
Tauri 的 `#[tauri::command]` 默认在主事件循环线程上执行。如果命令是同步的或包含阻塞操作（`std::fs`、`std::thread::sleep`、大量计算），会阻塞 WebView 更新和窗口事件处理。

**预防策略：**
- 所有可能耗时的 command 必须标记为 `async fn`
- CPU 密集型操作使用 `tokio::task::spawn_blocking` 移到专用线程池
- SQLite 写入使用异步驱动（`sqlx`）而非同步 `rusqlite`
- 避免在 command 函数中使用 `std::fs::*`，改用 `tokio::fs::*`

**预警信号：**
- command 执行期间窗口拖动卡顿
- 用户操作延迟超过 100ms
- `tauri info` 显示同步 command

**对应阶段：** Phase 2（Rust 后端开发阶段）

---

## 已有反模式验证与补充

以下验证了 docs/02 中定义的 7 个反模式，并补充技术层面的具体陷阱：

| 反模式 | 验证结果 | 技术层面补充 |
|--------|----------|-------------|
| AP-1: 把所有 GitHub 活动都当作更新 | ✅ 正确 | Events API 延迟 30s-6h，不适合做实时信号；Events API 分页限制 300 条 |
| AP-2: 把 TopK 当成一次性搜索结果页 | ✅ 正确 | Search API 结果不可确定性排序；无快照则无法对比变化 |
| AP-3: 把资源雷达做成 AI 工具导航站 | ✅ 正确 | MCP/Skills 分类本身边界模糊，需要限定 scope |
| AP-4: 客户端直接无节制打 GitHub API | ✅ 正确 | Secondary rate limits 无 header 警告，并发请求可能被静默限流 |
| AP-5: 一开始全端都正式上线 | ✅ 正确 | 每个平台的 WebView 差异、签名流程、打包格式都不同 |
| AP-6: 用 SurrealDB 提前抽象 | ✅ 正确 | SQLite + 读写分离对桌面应用足够；SurrealDB 生态不成熟 |
| AP-7: 依赖 LLM 才能生成摘要 | ✅ 正确 | LLM 调用增加启动延迟、成本、网络依赖，与离线优先矛盾 |

---

## "看起来完成但其实没完成"检查清单

- [ ] **ACL 权限**：所有自定义 command 和插件都在 `capabilities/default.json` 中声明——检查方法：新机器上 fresh install 后完整功能回归
- [ ] **SQLite 并发**：WAL 模式已启用、busy_timeout 已设置——检查方法：并行执行 10 个写入操作不报错
- [ ] **SvelteKit SSR 禁用**：根 layout 有 `ssr = false`——检查方法：`npm run build` 无 "dynamic routes" 警告
- [ ] **GitHub ETag 缓存**：304 响应率 > 50%——检查方法：运行指标面板监控
- [ ] **Rate limit 隔离**：Search 和 Core 预算池独立——检查方法：Search 耗尽时 Core 同步不受影响
- [ ] **macOS 签名公证**：`.dmg` 在非开发机器上可正常打开——检查方法：在干净 macOS VM 中测试
- [ ] **Windows 签名**：`.exe` 无 SmartScreen 红色警告——检查方法：在全新 Windows 安装中测试
- [ ] **离线模式**：断网后应用可打开并展示上次缓存——检查方法：禁用网络后启动应用
- [ ] **异步 command**：所有 command 标记为 `async fn`——检查方法：Clippy lint + 手动审查
- [ ] **Migration 幂等**：连续执行两次 migration 无副作用——检查方法：集成测试
- [ ] **Token 安全**：token 未出现在 SQLite、日志、崩溃报告中——检查方法：grep 搜索所有持久化文件
- [ ] **跨平台 CSS**：macOS + Windows 上关键布局一致——检查方法：两个平台截图对比

---

## 技术债务模式

| 捷径 | 短期收益 | 长期代价 | 何时可接受 |
|------|----------|----------|-----------|
| 使用官方 `tauri-plugin-sql` 而非自行封装 | 快速集成、零配置 | 并发性能受限（已知 bug #2254）；migration 时机不直观 | MVP 阶段可接受，但需预留替换接口 |
| Search API 结果不存快照 | 开发速度快 | 排行榜不可信、无法对比变化 | 永远不可接受（AP-2） |
| 暂不做 ETag 条件请求 | 省去缓存层开发 | Rate limit 快速耗尽 | 仅在 Phase 1 原型验证时可接受 |
| 同步 command 处理轻量操作 | 代码更简单 | 并发时 UI 卡顿 | 操作 < 10ms 时可接受 |
| 不配置代码签名 | 省去证书申请流程 | 用户看到安全警告、无法分发 | 仅在内部开发期间可接受 |

---

## 集成陷阱

| 集成 | 常见错误 | 正确做法 |
|------|----------|----------|
| GitHub Search API | 不分页、不设 TTL、不隔离预算 | 分页（≤100/页）、本地 TTL 5-15min、独立 budget pool |
| GitHub Releases API | 把所有 release 视为"更新" | 只处理 `published` 事件；过滤 draft/prerelease |
| GitHub Events API | 当作实时接口 | 接受 30s-6h 延迟；仅作辅助信号源 |
| Tauri plugin-sql | 依赖 `load()` 自动执行 migration | 配置 `preload`；验证 migration 时机 |
| Tauri plugin-notification | 不处理权限拒绝 | 检查通知权限状态；降级为应用内提示 |
| Tauri plugin-updater | 忘记 `createUpdaterArtifacts: true` | 在 `tauri.conf.json` bundle 配置中显式开启 |
| Tokio + SQLite | `std::sync::Mutex` + `.await` = 死锁 | 使用 `tokio::sync::Mutex` 或分离读写连接 |

---

## 性能陷阱

| 陷阱 | 症状 | 预防 | 触发规模 |
|------|------|------|----------|
| IPC 传递完整 SQLite 结果集 | UI 首屏加载 > 2s | 分页查询，每页 ≤ 100 条 | 单表 > 1000 行 |
| 未启用 WAL 模式 | 同步期间写入阻塞读取 | `PRAGMA journal_mode=WAL` | 任何写入并发 |
| Search API 无本地缓存 | 每次操作都消耗 rate limit | ETag + 本地 TTL | > 30 次/min 操作 |
| 前端渲染大量 DOM 节点 | 列表滚动卡顿 | 虚拟滚动（virtual list） | 列表 > 500 项 |
| 同步引擎串行处理 | 50 个 repo 同步 > 5 分钟 | 并发请求（受 rate limit 控制）+ 条件请求 | > 20 个订阅 |
| Tokio spawn 无限制 | 内存增长、调度开销 | 使用 `Semaphore` 限制并发数 | > 100 并发 task |

---

## 安全错误

| 错误 | 风险 | 预防 |
|------|------|------|
| GitHub token 存入 SQLite 明文 | 数据库文件泄露 = token 泄露 | 使用 OS 安全存储（keychain/credential manager） |
| token 出现在日志/崩溃报告 | 意外泄露 | 结构化日志脱敏；崩溃报告排除敏感字段 |
| 过度申请 GitHub scope | 最小权限原则违反 | v1 仅需 `public_repo` 读取权限，UI 明确说明 |
| 本地缓存文件无清理入口 | 用户无法安全清除数据 | 提供"清除本地缓存"功能，不误删 token |
| CSP 配置过于宽松 | XSS 攻击面扩大 | `default-src 'self'`；不使用 `'unsafe-eval'` |

---

## UX 陷阱

| 陷阱 | 用户影响 | 更好方案 |
|------|----------|----------|
| 同步中阻塞 UI | 用户被迫等待 | 允许浏览缓存，同步状态指示器 |
| 数据过期无提示 | 用户以为看到的是最新数据 | 明确标注"上次同步时间"和"数据可能过期" |
| Rate limit 时静默失败 | 用户以为没有新数据 | 告知用户"受限于 API 频率限制，延迟刷新" |
| 空白页无解释 | 用户困惑 | 给出"为什么没有数据"的解释和下一步操作 |
| 通知无法定位到对应 signal | 用户点击通知后找不到相关内容 | 每个通知必须 deep link 到具体 signal |

---

## 陷阱到阶段映射

| 陷阱 | 预防阶段 | 验证方式 |
|------|----------|----------|
| 1. ACL 权限遗漏 | Phase 1 脚手架 | fresh install 功能回归测试 |
| 2. IPC 大数据瓶颈 | Phase 2 数据层 | 性能预算测试（首屏 < 300ms） |
| 3. SvelteKit SSR 误用 | Phase 1 脚手架 | `npm run build` 无警告 |
| 4. SQLite 并发死锁 | Phase 2 数据层 | 并发写入压力测试 |
| 5. Migration 中断 | Phase 2 数据层 | migration 幂等性集成测试 |
| 6. Rate budget 踩踏 | Phase 2 同步引擎 | Search 耗尽时 Core 同步不受影响 |
| 7. Search 1000 上限 | Phase 2 TopK 引擎 | 快照对比功能可用 |
| 8. async Mutex 死锁 | Phase 2 Rust 后端 | 长时间运行压力测试 + tokio-console |
| 9. 插件碎片化 | Phase 1 脚手架 | 插件安装 checklist 审查 |
| 10. 签名公证 | Phase 1-2 CI/CD | 非开发机器上安装测试 |
| 11. WebView 平台差异 | 贯穿所有阶段 | macOS + Windows 截图对比 |
| 12. ETag 不完整 | Phase 2 同步引擎 | 304 hit rate 监控 > 50% |
| 13. 阻塞主线程 | Phase 2 Rust 后端 | Clippy lint + UI 响应性测试 |

---

## 来源

- Tauri v2 ACL 迁移指南: https://coldfusion-example.blogspot.com/2025/12/tauri-v2-upgrade-guide-solving.html
- Tauri v2 迁移陷阱回顾: https://squareuncle.com/en/posts/2026-02-09-tauri-v2-migration-hallucination/
- Tauri IPC 性能讨论: https://github.com/orgs/tauri-apps/discussions/11915
- Tauri SQL 插件并发 bug: https://github.com/tauri-apps/plugins-workspace/issues/2254
- SQLite 并发实战: https://www.drmhse.com/posts/battling-with-sqlite-in-a-concurrent-environment/
- Tokio 单 Mutex 死锁案例: https://turso.tech/blog/how-to-deadlock-tokio-application-in-rust-with-just-a-single-mutex
- SvelteKit static-adapter 讨论: https://github.com/orgs/tauri-apps/discussions/6423
- GitHub API 最佳实践: https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api
- GitHub API rate limit 指南: https://github.com/orgs/community/discussions/163553
- Tauri v2 签名与公证: https://v2.tauri.app/distribute/sign/macos
- Tauri v2 发布管道: https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-code-signing-for-macos-and-windows-part-12-3o9n
- 项目内部文档: docs/02 (反模式), docs/07 (质量与安全规范)

---

*桌面技术雷达工作台域陷阱研究*
*Researched: 2026-03-22*
