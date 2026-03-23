---
phase: 01-scaffold-auth
plan: 02
subsystem: auth
tags: [octocrab, keyring, tauri-commands, github-api, settings]

requires:
  - phase: 01-scaffold-auth
    provides: domain auth models (User, AuthToken, AuthError), shared_contracts crate

provides:
  - GitHub PAT authentication client (validate_token, fetch_repo_info)
  - Keyring-based token storage (store/load/remove)
  - Tauri commands for auth (validate, store, load, remove, get_current_user, fetch_repo_info)
  - Tauri commands for settings (get/update via tauri-plugin-store)
  - Settings domain model with defaults

affects: [data-layer, topk-discovery, subscriptions]

tech-stack:
  added: [keyring (OS secure storage), octocrab (GitHub API client), tauri-plugin-store]
  patterns: [fresh client per call (no persistent octocrab), keyring entry CRUD, DTO↔domain conversion]

key-files:
  created:
    - crates/domain/src/settings.rs — Settings, NotificationFrequency, QuietHours domain structs
    - crates/shared_contracts/src/auth_dto.rs — UserDto, AuthStatus, ValidateTokenResponse
    - crates/shared_contracts/src/settings_dto.rs — SettingsDto with bidirectional domain conversion
    - crates/shared_contracts/src/repo_dto.rs — RepoBasicInfo DTO
    - crates/github_adapter/src/auth.rs — validate_token, fetch_repo_info via octocrab
    - crates/runtime_tauri/src/commands/mod.rs — command module exports
    - crates/runtime_tauri/src/commands/auth.rs — 6 auth Tauri commands
    - crates/runtime_tauri/src/commands/settings.rs — get_settings, update_settings commands
  modified:
    - crates/domain/src/lib.rs — added auth + settings modules
    - crates/domain/Cargo.toml — added serde, serde_json, chrono, thiserror
    - crates/shared_contracts/src/lib.rs — added auth_dto, settings_dto, repo_dto modules
    - crates/shared_contracts/Cargo.toml — added domain dependency
    - crates/github_adapter/src/lib.rs — added auth module
    - crates/github_adapter/Cargo.toml — added shared_contracts dependency
    - crates/runtime_tauri/src/lib.rs — added commands module
    - crates/runtime_tauri/Cargo.toml — added keyring dependency

key-decisions:
  - "Fresh Octocrab client per call — no persistent instance, simpler lifecycle"
  - "Keyring service 'geek-taste' / key 'github-pat' — standard OS secure storage"
  - "ValidateTokenResponse returns success/error instead of throwing — frontend-friendly"
  - "Settings partial update — merge with existing, not full replace"
  - "Settings domain model created alongside DTOs — was implicit dependency from settings commands"

patterns-established:
  - "DTO↔domain conversion: From impls in shared_contracts, not in command handlers"
  - "Keyring entry pattern: Entry::new(SERVICE, KEY) → get/set/delete with .map_err(|e| e.to_string())"
  - "Auth error mapping: HTTP status codes → typed AuthError enum variants"

requirements-completed: [FOUND-01, FOUND-02]

duration: 8min
completed: 2026-03-23
---

# Phase 1 Plan 2: GitHub 认证 Tauri 命令 Summary

**octocrab GitHub PAT 认证客户端 + keyring 安全存储 + 8 个 Tauri 命令 (auth 6 + settings 2) + Settings 领域模型**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-23T03:00:00Z
- **Completed:** 2026-03-23T03:08:00Z
- **Tasks:** 4
- **Files modified:** 16

## Accomplishments
- GitHub PAT 验证客户端（octocrab，401→InvalidToken / 403→RateLimited）
- OS 安全存储集成（keyring: service="geek-taste", key="github-pat"）
- 6 个认证 Tauri 命令：validate / store / load / remove / get_current_user / fetch_repo_info
- 2 个设置 Tauri 命令：get_settings / update_settings（tauri-plugin-store）
- Settings 领域模型（NotificationFrequency, QuietHours, Settings + defaults + tests）

## Task Commits

1. **Domain settings + Shared contracts DTOs** — `2a12649` (feat)
2. **GitHub adapter auth module** — `09075ae` (feat)
3. **Tauri auth commands** — `e586916` (feat)
4. **Settings commands + module wiring** — `39b097b` (feat)

## Files Created/Modified
- `crates/domain/src/settings.rs` — Settings, NotificationFrequency, QuietHours with defaults + tests
- `crates/shared_contracts/src/auth_dto.rs` — UserDto, AuthStatus, ValidateTokenResponse
- `crates/shared_contracts/src/settings_dto.rs` — SettingsDto with bidirectional From conversions
- `crates/shared_contracts/src/repo_dto.rs` — RepoBasicInfo DTO
- `crates/github_adapter/src/auth.rs` — validate_token, fetch_repo_info
- `crates/runtime_tauri/src/commands/auth.rs` — 6 auth Tauri commands
- `crates/runtime_tauri/src/commands/settings.rs` — 2 settings Tauri commands
- + 8 modified module exports and Cargo.toml files

## Decisions Made
- Fresh Octocrab client per call — simpler lifecycle, no shared state
- ValidateTokenResponse wraps success/error — frontend gets structured response
- Settings partial update — only fields provided are updated
- Settings domain model created alongside DTOs — implicit dependency for settings commands

## Deviations from Plan

### Auto-added Missing Critical

**1. [Rule 2 - Missing Critical] Added Settings domain model**
- **Found during:** Task 3 (settings commands)
- **Issue:** settings.rs commands reference `domain::settings::Settings` but module didn't exist
- **Fix:** Created crates/domain/src/settings.rs with Settings, NotificationFrequency, QuietHours, defaults, and tests
- **Files modified:** crates/domain/src/settings.rs, crates/domain/src/lib.rs, crates/domain/Cargo.toml
- **Verification:** cargo check -p runtime_tauri passes
- **Committed in:** 2a12649 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical)
**Impact on plan:** Domain settings model was implicit dependency — necessary for settings commands to compile. No scope creep.

## Issues Encountered
None — all verification checks pass.

## Next Phase Readiness
- GitHub auth commands ready for frontend integration
- Keyring token management ready for onboarding flow
- Settings commands ready for settings UI
- Ready for 01-03 plan

---
*Phase: 01-scaffold-auth*
*Completed: 2026-03-23*
