---
phase: 03-subscription-signal
plan: 03
subsystem: github-adapter
tags: [octocrab, releases, tags, incremental-sync]
---

# Phase 03 Plan 03 Summary

已完成 GitHub Releases/Tags 拉取模块：新增 `crates/github_adapter/src/releases.rs` 并在 `crates/github_adapter/src/lib.rs` 导出 `releases` 模块。

## 完成项

- 新增 `ReleaseInfo` / `TagInfo` 两个传输结构。
- 实现 `fetch_latest_releases(token, owner, repo, since_cursor)`。
- 实现 `fetch_latest_tags(token, owner, repo, since_tag)`。
- 两个函数均通过 cursor 参数支持增量过滤。

## 关键文件

- `crates/github_adapter/src/releases.rs`
- `crates/github_adapter/src/lib.rs`

## 验证

- `rtk cargo check -p github_adapter` 通过。

## 风险与备注

- 当前实现使用 release 发布时间和 tag 名称进行增量过滤，后续若要完全对齐业务 cursor 语义可进一步收敛。
