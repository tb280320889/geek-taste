# Phase 05 Research: 打磨与发布准备

**Gathered:** 2026-03-24
**Status:** Complete

## RESEARCH COMPLETE

---

## 1. Offline Support (HOME-03)

### 1.1 Current State

- SQLite is local, WAL mode enabled — all data persists across app restarts
- GitHub API calls fail with `octocrab::Error` variants (HttpError, RateLimited, etc.)
- Frontend stores have `loading` and `error` states but no "offline" or "stale" concept
- No `last_synced_at` tracking on any data source

### 1.2 Network Detection in Tauri

**Approach:** Dual-layer detection
1. **Frontend:** `navigator.onLine` (basic, unreliable on some platforms)
2. **Backend:** Attempt a lightweight GitHub API call (e.g., `GET /rate_limit`) — if it fails, classify as offline

**Recommended:** Store `is_online` state in a Svelte store, updated:
- On app startup (API health check)
- Before each major IPC call that touches GitHub
- On IPC error classified as `NETWORK_ERROR`

### 1.3 Error Classification

Current `octocrab::Error` variants that map to offline/network:
- `HttpError { status: 0, .. }` — DNS failure / network unreachable
- `HttpError { status: 502/503/504, .. }` — Gateway errors (transient)
- `RateLimited { .. }` — Not offline, but can't fetch

**Rust side:** Create an `AppError` enum with variants:
```rust
pub enum AppError {
    AuthExpired(String),
    NetworkError(String),
    RateLimited { retry_after: Option<u64> },
    NotFound(String),
    Internal(String),
}
impl std::fmt::Display for AppError { ... }
```

All Tauri commands return `Result<T, AppError>` → serialize to `{ code: string, message: string }` for frontend.

### 1.4 Stale Data Strategy

- Add `last_refreshed_at: Option<String>` to ranking views, signal feeds
- On successful API refresh → update timestamp
- On offline/error → keep existing data, show STALE indicator
- STALE = `last_refreshed_at` older than expected refresh interval:
  - Ranking views: stale if > 1h
  - Signal feed: stale if > 30min
  - Subscriptions: stale if > 1h

### 1.5 UI Indicators

- **Offline banner:** Thin amber bar at app top: "网络不可用，显示缓存数据"
- **STALE badge:** Small amber dot next to data section titles (e.g., "Today ●")
- **No data state:** If SQLite is empty AND offline → show "首次使用需要网络连接" message

---

## 2. Error Handling Hardening

### 2.1 Current Pattern

```rust
// Rust: returns String error
pub fn command(...) -> Result<T, String> { ... }

// Frontend: catches string error
try { await invoke(...) } catch (e) { error = String(e) }
```

**Problem:** No structured error codes. Frontend can't distinguish error types.

### 2.2 Proposed Pattern

```rust
// Rust: structured error
#[derive(serde::Serialize)]
pub struct AppErrorDto {
    pub code: &'static str,  // "AUTH_EXPIRED", "NETWORK_ERROR", etc.
    pub message: String,
}
```

Frontend:
```typescript
interface AppError {
  code: "AUTH_EXPIRED" | "NETWORK_ERROR" | "RATE_LIMITED" | "NOT_FOUND" | "INTERNAL";
  message: string;
}
```

### 2.3 Migration Strategy

- Introduce `AppErrorDto` as new error type
- Update commands incrementally (not all at once)
- Frontend stores add typed error state alongside existing string error
- Toast system already handles errors — just needs to check `error.code`

---

## 3. Momentum Warmup

### 3.1 Current State

- `ranking_snapshots` table stores periodic snapshots
- Momentum score = `0.5 * star_delta + 0.2 * fork_delta + 0.3 * recency`
- If no snapshots exist for a view → `star_delta` and `fork_delta` are 0 → score is just recency

### 3.2 Warmup Fallback

When `ranking_snapshots` is empty for a view:
1. Fall back to `UPDATED_DESC` ranking mode
2. Show a hint: "Momentum 评分将在首次快照后生效"
3. First snapshot creation happens on schedule (12h) — no need to force it

### 3.3 Implementation

- In `execute_ranking`: check if snapshot count > 0 for view
- If 0: return items sorted by `updated_at DESC` with `warmup: true` flag
- Frontend: if `warmup === true`, show hint banner

---

## 4. Release Packaging

### 4.1 Current State

- `tauri.conf.json` has `bundle.active: true`, `targets: "all"`
- Icon paths listed but may be placeholder files
- No code signing configured

### 4.2 Tauri v2 Bundle Steps

1. **Icons:** Generate proper icon set from a source PNG (1024x1024)
   - Tauri CLI: `tauri icon` generates all sizes
   - Required: 32x32, 128x128, 128x128@2x, icon.icns (macOS), icon.ico (Windows)

2. **Build:** `tauri build` produces:
   - Windows: `.msi` or `.exe` installer
   - macOS: `.dmg` or `.app`
   - Linux: `.deb`, `.AppImage`, `.rpm`

3. **Code signing:** Post-v1 — requires:
   - Apple: Developer ID certificate + notarization
   - Windows: Authenticode certificate
   - Document the process, don't implement yet

### 4.3 v1 Release Checklist

- [ ] App icons generated (not placeholders)
- [ ] `tauri build` succeeds on all target platforms
- [ ] Version number in tauri.conf.json set to `0.1.0`
- [ ] CHANGELOG.md with initial release notes
- [ ] README.md with install instructions

---

## 5. Key Decisions for Planning

1. **Offline:** Dual-layer detection (navigator.onLine + API health check), STALE = data older than expected interval
2. **Errors:** Introduce `AppErrorDto` with typed codes, migrate incrementally
3. **Warmup:** Fallback to UPDATED_DESC when no snapshots exist, show hint
4. **Release:** Icons + build verification only, code signing deferred
5. **No new dependencies** — all offline/error logic uses existing infrastructure
6. **Scope:** 1 plan covering offline + error handling + warmup (small phase)

---

## 6. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `navigator.onLine` unreliable | False offline/online states | API health check as fallback |
| Error migration breaks existing commands | Regression risk | Incremental migration, keep String error for unchanged commands |
| Icon generation quality | Poor branding | Use a source icon, generate with `tauri icon` |
| Warmup hint confuses users | UX confusion | Clear message: "首次快照后 Momentum 评分生效" |

---

## 7. Validation Architecture

Not applicable — Phase 5 is hardening/polish, no complex validation needed.
Manual testing checklist covers: offline flow, error states, warmup display, build output.

---

*Research complete: 2026-03-24*
*Ready for: Planning Phase 05*
