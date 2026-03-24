# Phase 05 Context: 打磨与发布准备

**Gathered:** 2026-03-24
**Status:** Ready for planning
**Source:** Assumptions mode (codebase analysis)

## Phase Boundary

Phase 5 is the final v1 hardening phase. It delivers:
1. **Offline degradation** — App opens and functions with cached SQLite data when GitHub API is unreachable
2. **Stale data indicators** — UI shows STALE status on data that hasn't been refreshed
3. **Error handling hardening** — Consistent error patterns across all IPC commands and frontend stores
4. **Performance baseline** — Momentum score warmup (no-snapshot fallback), startup optimization
5. **Release packaging** — Tauri bundle configuration, code signing setup

## Implementation Decisions

### Offline Support (HOME-03)
- SQLite is already local (WAL mode) — data is always available offline
- Offline = GitHub API calls fail (network unreachable)
- Strategy: try API → on failure → serve cached SQLite data → show STALE badge
- Need: network status detection (Tauri `navigator.onLine` + API call error classification)
- Need: `last_synced_at` timestamp per data source (ranking views, subscriptions, signals)
- Need: STALE indicator component — small badge/border treatment, not disruptive
- Home page (today view) must show "数据可能已过期" when offline

### Error Handling
- Current pattern: `Result<T, String>` in Rust commands, `Promise<T>` with `string` error in frontend
- Problem: Frontend stores don't distinguish between "no data", "API error", "offline", "auth error"
- Solution: Typed error codes from Rust → frontend maps to user-friendly messages
- Error categories: `AUTH_EXPIRED`, `NETWORK_ERROR`, `RATE_LIMITED`, `NOT_FOUND`, `INTERNAL`
- Frontend stores: error state with `{ code: string, message: string } | null`

### Performance
- Momentum warmup: When no snapshots exist for a view, fall back to UPDATED_DESC ranking with user-visible hint
- App startup: load cached data from SQLite first (instant) → background refresh from API
- No premature optimization — measure first, optimize what's slow

### Release Packaging
- Tauri bundle: targets `all` (already configured in tauri.conf.json)
- Code signing: document the process, set up placeholder config
- v1 is unsigned for development — code signing is post-v1 milestone
- Icon files need to exist (currently placeholder paths in tauri.conf.json)

## Canonical References

### Architecture
- `crates/domain/src/` — Domain types and rules (pure Rust, no I/O)
- `crates/application/src/` — Use case orchestration
- `crates/persistence_sqlite/src/` — SQLite repository impl
- `crates/github_adapter/src/` — GitHub REST client + rate limiting
- `crates/runtime_tauri/src/commands/` — Tauri IPC commands

### Frontend
- `apps/desktop-ui/src/lib/ipc/tauri.ts` — IPC wrapper (components call this, not invoke directly)
- `apps/desktop-ui/src/lib/stores/` — Svelte stores for each domain
- `apps/desktop-ui/src/lib/types.ts` — Frontend type definitions
- `apps/desktop-ui/src/routes/` — Page components

### Config
- `apps/desktop-ui/src-tauri/tauri.conf.json` — Tauri app config
- `.planning/STATE.md` — Accumulated decisions

### Patterns to follow
- Each IPC command: `fn command_name(...) -> Result<T, String>`
- Each frontend store: loading/error/data pattern
- DB connections: per-call open + WAL + init_db

## Specific Ideas

- Offline banner: thin bar at top of app when offline, dismissible
- STALE badge: small amber dot/badge next to data timestamps
- Error toast: existing toast system for transient errors
- Network check: `navigator.onLine` + fallback API health check (e.g., GitHub `/rate_limit`)
- `last_refreshed_at` field on ranking views and signal feeds
- Momentum warmup: if `ranking_snapshots` table is empty for a view, skip delta calculation, use base score only

## Deferred Ideas

- Code signing certificates (post-v1, needs Apple Developer / Windows cert)
- Auto-update system (post-v1, needs update server)
- Crash reporting / telemetry (post-v1)
- Performance profiling and optimization (measure first, post-v1 if needed)

---

*Phase: 05-polish-release*
*Context gathered: 2026-03-24 via assumptions mode*
