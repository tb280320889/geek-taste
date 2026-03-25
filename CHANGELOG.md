# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0.0] - 2026-03-25

### Added

- **IPC Command Registration:** Register all 28 frontend IPC commands in Tauri handler (topk, signal, subscription, resource, sync_status)
- **Offline Banner UI:** Amber banner displayed at app top when network is unavailable
- **STALE Indicator UI:** Amber dot next to home page title when TopK data is stale (>1h)
- **Crate module declarations:** Complete `pub mod` declarations in domain, shared_contracts, application, persistence_sqlite, github_adapter lib.rs files
- **init_db function:** Database migration initializer in persistence_sqlite crate
- **send_high_signal_notification:** Desktop notification function in notification_adapter crate

### Changed

- **Cargo.toml dependencies:** Added missing dependencies (anyhow, chrono, rusqlite, serde_json, rusqlite_migration, thiserror, tokio, tauri) across application, persistence_sqlite, github_adapter, runtime_tauri, notification_adapter crates
- **helpers.rs:** Explicit type annotations for rusqlite/anyhow error types

### Fixed

- **Compilation:** All 6 crates now compile with 0 errors (previously 60+ unresolved import errors)
- **Frontend:** svelte-check passes with 0 errors
