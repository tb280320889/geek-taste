# Phase 2 Research: 数据层与 TopK 发现引擎

**Phase:** 02-topk
**Date:** 2026-03-23
**Status:** RESEARCH COMPLETE

---

## 1. Technical Approach Summary

Phase 2 requires building three interconnected layers:

1. **SQLite persistence layer** — schema + migrations + repositories (rusqlite + rusqlite_migration)
2. **GitHub REST client expansion** — search API integration with rate budget + ETag (octocrab)
3. **TopK ranking engine** — domain models + application use cases + frontend UI (SvelteKit 5)

## 2. Architecture Decisions

### 2.1 Database Layer

- **Migration framework:** `rusqlite_migration 2.4` (already in workspace Cargo.toml)
- **Initial migration creates 4 tables:** `repositories`, `repo_snapshots`, `ranking_views`, `ranking_snapshots`
- **WAL mode + busy_timeout(5000ms)** for concurrent read/write
- **serde_rusqlite 0.41** for row mapping (already in workspace, compatible with rusqlite 0.38)
- **Pattern:** Single `Connection` per request via `Mutex<Connection>` in Tauri managed state (simple for v1 single-user desktop app)

### 2.2 GitHub Search API

- **octocrab 0.49** already provides `search().repos()` — extend with sort/filter/pagination
- **Rate budget:** Two separate counters — core (5000/h) and search (30/min)
- **ETag:** Use `octocrab`'s built-in `If-None-Match` support via custom headers
- **Pagination:** offset-based, max 1000 results (GitHub hard limit), default page size 30

### 2.3 Momentum Score

Formula from spec (04):
```
score = 0.5 × starDelta + 0.2 × forkDelta + 0.3 × updatedRecency
```

- `starDelta` = normalized star growth since last snapshot
- `forkDelta` = normalized fork growth since last snapshot
- `updatedRecency` = inverse of days since last push (0-1)

Requires: `repo_snapshots` table with previous state for delta computation.

### 2.4 Snapshot Strategy (D-07, D-08)

- `tokio-cron-scheduler 0.15` (already in workspace) for 12h timer
- On-demand: when user opens a view and last_snapshot_at > 12h ago
- New RankingView: immediate warmup snapshot on creation

## 3. Crate Dependency Map

```
domain (new: repository.rs, ranking.rs)
  ↓
shared_contracts (new: ranking_dto.rs)
  ↓
persistence_sqlite (migration + repository impl)
github_adapter (new: search.rs)
  ↓
application (new: topk use cases)
  ↓
runtime_tauri (new: commands/topk.rs)
  ↓
desktop-ui (topk page + stores + components)
```

**Dependency direction is strictly top-down (hexagonal architecture).**

## 4. Key Implementation Patterns (from Phase 1)

### Rust patterns
- Domain structs: `#[derive(Debug, Clone, Serialize, Deserialize)]` + `thiserror` errors
- Tauri commands: thin wrapper — keyring load token → adapter call → return DTO
- DTO conversion: `From<DomainType> for DtoType` in shared_contracts

### Frontend patterns
- IPC wrapper: `$lib/ipc/tauri.ts` — typed `invoke()` calls
- Stores: Svelte writable stores with async load/update functions
- Components: SvelteKit 5 Runes (`$state()`, `$derived()`, `$effect()`, `$props()`)
- Styling: CSS variables (`var(--radius)`, `var(--border)`, `var(--accent)`)

## 5. Repository Table Schema (from docs/05)

Phase 2 creates these 4 tables (subscriptions/signals/resources/deliveries deferred to Phase 3):

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `repositories` | Cached GitHub repo metadata | repo_id (PK), full_name (UNIQUE), stargazers_count, primary_language, updated_at, last_synced_at |
| `repo_snapshots` | Periodic repo state for trend computation | snapshot_id (PK), repo_id (FK), stargazers_count, forks_count |
| `ranking_views` | User-saved filter+sort combinations | ranking_view_id (PK), name, filters_json, ranking_mode, k_value, is_pinned |
| `ranking_snapshots` | View results at point-in-time | ranking_snapshot_id (PK), ranking_view_id (FK), items_json |

## 6. Risk Assessment

| Risk | Mitigation |
|------|------------|
| GitHub Search API non-determinism | Snapshot comparison handles variance — ranking is relative |
| Rate limit exhaustion | Budget manager with core/search pool isolation (D-20) |
| SQLite concurrent writes | WAL mode + busy_timeout handles single-user v1 |
| serder_rusqlite compatibility | Version 0.41 is the only compatible version with rusqlite 0.38 — already pinned |

## 7. Validation Architecture

- **Unit tests:** Domain models (Repository, RankingView, Momentum score), migration SQL correctness
- **Integration tests:** SQLite CRUD roundtrip, GitHub search mock response parsing
- **Frontend:** TopK page renders RankingView list, filter panel changes results, subscribe popover works

## 8. Plan Sizing Estimate

| Layer | Complexity | Est. Plans |
|-------|-----------|-----------|
| SQLite migration + repositories | Medium | 1-2 plans |
| GitHub search client + rate budget | Medium | 1 plan |
| Domain models + ranking logic | Medium | 1 plan |
| Application use cases | Low-Medium | 1 plan |
| Tauri commands + IPC | Low | 1 plan |
| Frontend TopK UI | High (most files) | 2-3 plans |
| **Total** | | **7-9 plans** |

---

*Research completed: 2026-03-23*
