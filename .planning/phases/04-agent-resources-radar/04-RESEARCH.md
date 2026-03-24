# Phase 04 Research: Agent 资源雷达

**Gathered:** 2026-03-24
**Status:** Complete

## RESEARCH COMPLETE

---

## 1. Existing Codebase Patterns

### 1.1 Architecture Layers (must follow per docs/03)

| Layer | Crate | Pattern |
|-------|-------|---------|
| Domain | `crates/domain` | Pure types + rules, no I/O. Modules: `auth.rs`, `ranking.rs`, `repository.rs`, `settings.rs`, `signal.rs`, `subscription.rs` |
| Application | `crates/application` | Use-case orchestration. Modules: `signal.rs`, `subscription.rs`, `topk.rs` |
| Persistence | `crates/persistence_sqlite` | Repository impl. Modules: `repo_repository.rs`, `ranking_repository.rs`, `signal_repository.rs`, `subscription_repository.rs` |
| Runtime | `crates/runtime_tauri/src/commands/` | Tauri IPC commands. Modules: `auth.rs`, `settings.rs`, `topk.rs`, `subscription.rs`, `signal.rs` |
| Frontend | `apps/desktop-ui/src/lib/` | SvelteKit SPA. `stores/`, `ipc/tauri.ts`, `types.ts`, `components/` |
| Contracts | `crates/shared_contracts` | JSON schema / DTO / enum export (currently `.gitkeep`) |

### 1.2 Data Model (already defined in docs/05)

- `resources` table: `resource_id`, `source_repo_id`, `resource_kind`, `title`, `summary`, `source_url`, `languages_json`, `framework_tags_json`, `agent_tags_json`, `curation_level`, `last_scored_at`, `is_active`
- `resource_tags` table: `resource_id`, `tag_type`, `tag_value`
- `signals` table already has `resource_id` column (Phase 3 migration V002)
- Signal types already include `RESOURCE_EMERGED` and `RESOURCE_RERANKED`
- Signal source_kind already includes `Resource`

### 1.3 Contract (already defined in docs/05)

```json
{
  "resourceId": "res_01",
  "resourceKind": "MCP_SERVER",
  "title": "acme/mcp-rust-tools",
  "sourceRepoId": 555,
  "languages": ["Rust"],
  "frameworkTags": ["Axum"],
  "agentTags": ["MCP", "coding-agent"],
  "score": 0.91,
  "whyRecommended": ["matches language Rust", "matches framework Axum", "high recent growth"]
}
```

### 1.4 Frontend Route

`apps/desktop-ui/src/routes/resources/+page.svelte` exists as a placeholder:
```svelte
<section class="card rounded-[var(--radius)] p-5">
  <h1 class="m-0 mb-2.5 text-2xl font-semibold">Resources</h1>
  <p class="muted">阶段 4 将在这里提供 MCP / Skills / Agent 资源雷达。</p>
</section>
```

### 1.5 GitHub Search API Constraints

- Authenticated: 30 req/min (Search API) — same budget as TopK search
- Core budget: 5000 req/hour — pool already exists with ETag support
- GitHub Search supports: `topic:mcp`, `topic:skill`, `topic:agent`, `language:X`
- Need to extend `RateBudget` in github_adapter to support a `resource` budget bucket

### 1.6 Settings State (existing)

User language interests are stored in settings (`.planning/STATE.md` references `Settings 中选择的语言/框架兴趣`). The settings module at `crates/domain/src/settings.rs` and `crates/persistence_sqlite/src` manages this.

Subscription language distribution can be inferred from the `repositories` table joined with `subscriptions`.

---

## 2. Technical Approach

### 2.1 Resource Discovery — GitHub Search Strategy

**Query patterns:**
- `topic:mcp language:Rust stars:>10` — MCP servers in user's primary language
- `topic:skill stars:>5` — Agent skills
- `topic:agent-framework` — Agent frameworks
- Combinations: `(topic:mcp OR topic:skill) language:Rust`

**GitHub search limitations:**
- Max 1000 results per query (pagination: 100/page, max 10 pages)
- Results may include non-relevant repos (topic abuse)
- Need filtering: exclude archived, disabled, forks → leverage existing `github_adapter::search` pattern from Phase 2

**Recommended approach:**
- Periodic batch discovery (e.g., every 24h) with configurable queries
- User curated repos get highest priority (no search needed)
- Discovery queries stored in `ranking_views`-style config, not hardcoded

### 2.2 stack_relevance Algorithm

**Source 1: User Settings Interests**
- Weight: 0.7 (primary signal)
- Languages from `settings` table (languages_json)
- Frameworks from `settings` table (frameworks_json)

**Source 2: Subscription Inference**
- Weight: 0.3 (secondary signal)
- Query: `SELECT primary_language, COUNT(*) FROM repositories JOIN subscriptions ON ... GROUP BY primary_language`
- Normalize to probability distribution

**Score calculation:**
```
stack_relevance = 0.7 * match(settings_langs, resource_langs)
                + 0.3 * match(inferred_langs, resource_langs)
```

Where `match` is a simple overlap ratio: `|intersection| / |union|`

### 2.3 Full Scoring Model

```
score = 0.4 * stack_relevance + 0.35 * star_delta_norm + 0.25 * recency_norm
```

- `star_delta_norm`: Normalized recent star growth (0-1 scale), same formula as MomentumScore
- `recency_norm`: Time decay from last push, e.g., `1.0 - days_since_push / 365`, clamped to [0,1]
- Weights are configurable constants

### 2.4 Recommendation Explanation — Template Rules

Template rules (80%+ coverage):
1. **Language match**: "你关注 Rust，这是 Rust 生态的 {resource_kind}"
2. **Framework match**: "你订阅了 {N} 个 {framework} 仓库，这个资源适用于 {framework}"
3. **Growth signal**: "近期 star 增长 {delta}，社区关注度上升"
4. **User curated**: "你已将此资源加入精选列表"
5. **Multiple signals**: Combine 2-3 templates

LLM fallback: v1 标记为 TODO，先不实现。

### 2.5 Tag System

**Initial tag vocabulary:**
- Agent types: `mcp`, `skill`, `agent`, `agent-framework`, `tool`
- Tech domains: `database`, `web`, `test`, `code-review`, `deploy`, `ci`, `monitoring`
- Languages: `rust`, `typescript`, `python`, `go`, `java`

Tags sourced from:
1. GitHub repo topics (auto-extracted)
2. User free input (free-form)
3. Suggested from vocabulary above

### 2.6 MCP Registry API

**Finding:** No official MCP registry API exists yet. GitHub topic search is the best v1 approach.
- `topic:mcp` on GitHub returns MCP server repos
- ModelContextProtocol org maintains an `awesome-mcp-servers` list that could be scraped
- v1 recommendation: Use GitHub Search only, no registry dependency

### 2.7 Rate Limiting Strategy

Resource discovery shares the Search API budget with TopK. Options:
1. **Shared pool** — Resource discovery and TopK share 30 req/min, with priority (TopK > Resources)
2. **Separate bucket** — Add `resource` budget to existing `RateBudget` struct

Recommendation: Shared pool with priority. Resource discovery is less time-sensitive than TopK refresh.

---

## 3. Integration Points

### 3.1 Domain Layer

New module: `crates/domain/src/resource.rs`
- `Resource` struct (matches `resources` table)
- `ResourceTag` struct (matches `resource_tags` table)
- `ResourceKind` enum: `McpServer`, `Skill`, `AgentFramework`, `Other`
- `ResourceScore` struct with breakdown
- `CurationLevel` enum: `UserCurated`, `SystemDiscovered`

### 3.2 Persistence Layer

New module: `crates/persistence_sqlite/src/resource_repository.rs`
- `insert_resource()`, `upsert_resource()`, `get_resource()`, `list_resources()`
- `insert_tags()`, `get_tags()`, `delete_tags()`
- `search_resources()` with filtering by tags, language, kind, min_score

New migration V003:
- `resources` table (per docs/05 spec)
- `resource_tags` table (per docs/05 spec)

### 3.3 Application Layer

New module: `crates/application/src/resource.rs`
- `discover_resources()` — GitHub Search + upsert
- `score_resources()` — stack_relevance + star_delta + recency
- `explain_recommendation()` — template rule engine
- `curate_resource()` — user add/remove from curated list

### 3.4 Tauri Commands

New module: `crates/runtime_tauri/src/commands/resource.rs`
- `list_resources` — paginated resource list with filters
- `curate_resource` — add/remove from user curated list
- `search_resources` — search by query + filters
- `get_resource_tags` — available tags for filtering

### 3.5 Frontend

New files:
- `apps/desktop-ui/src/lib/stores/resources.ts` — resource store
- `apps/desktop-ui/src/lib/components/ResourceCard.svelte` — resource display card
- `apps/desktop-ui/src/lib/components/ResourceFilters.svelte` — tag/language/kind filters
- Update `apps/desktop-ui/src/routes/resources/+page.svelte` — full page layout
- Update `apps/desktop-ui/src/lib/ipc/tauri.ts` — add resource IPC functions
- Update `apps/desktop-ui/src/lib/types.ts` — add ResourceCard type

---

## 4. Risks & Uncertainties

| Risk | Impact | Mitigation |
|------|--------|------------|
| GitHub topic coverage varies | Some MCP/skill repos don't use standard topics | Use multiple query patterns + user curation |
| Search API budget contention with TopK | Shared 30 req/min | Priority: TopK > Resources, batch resource discovery less frequently |
| Tag vocabulary may be incomplete | Users can't find relevant resources | Free-form tags + expand vocabulary over time |
| stack_relevance accuracy | Poor recommendations if settings empty | Fall back to subscription inference, prompt user to set interests |
| `resources` + `resource_tags` tables not in DB yet | Need V003 migration | Add migration in persistence_sqlite |

---

## 5. Key Decisions for Planning

1. **Tables:** `resources` + `resource_tags` per docs/05 spec (V003 migration)
2. **Domain model:** New `resource.rs` module in domain crate
3. **Scoring:** 0.4 * stack_relevance + 0.35 * star_delta + 0.25 * recency
4. **Explanation:** Template rules only, LLM fallback marked as TODO
5. **Discovery:** GitHub Search + user curation, shared Search API budget with priority
6. **Tags:** Auto-extracted from GitHub topics + user free-form + suggested vocabulary
7. **Frontend:** ResourceCard + ResourceFilters components, update resources page
8. **No new dependencies** — reuses existing octocrab, serde_rusqlite, existing patterns

---

*Research complete: 2026-03-24*
*Ready for: Planning Phase 04*
