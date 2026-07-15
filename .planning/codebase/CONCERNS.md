# Technical Concerns

## Known Gaps

| Area | Gap | Priority | Status |
|------|-----|----------|--------|
| Linux support | Drive detection + WiFi scanning exist, `ubuntu-latest` in build matrix — not in MVP release | Low | ✅ Resolved |
| macOS 14.4+ | `airport` WiFi scanning restored via `system_profiler` fallback — `parse_system_profiler_networks()` extracts all visible networks from "Other Local Wireless Networks" section | Medium | ✅ Resolved |
| Install cancellation | Frontend cancel button wired up via `useForkInstall` → `cancel_install` IPC | Medium | ✅ Resolved |
| Windows formatting | `format_drive` returns error — not yet implemented | Low | Open |
| Package registry | Hardcoded versions in `store.json` — mitigated by daily cron auto-update | Low | Open |

## File Size & Complexity

All previously flagged files have been addressed:

| File | Before | After | Resolution |
|------|--------|-------|------------|
| `install_tests.rs` | 789 | Split into 3 files | Split by concern |
| `lib.rs` | 647 | 335 | Contract tests extracted |
| `wifi.rs` | 535 | Split into 4 files | Platform modules |
| `package.ts` | 455 | 236 | Registry conversion extracted |

Largest source files (non-test):
- `install.rs` (512 lines) — acceptable, orchestration logic
- `useForkInstall.ts` (425 lines) — acceptable, install state machine
- `validate.rs` (420 lines) — acceptable, post-install checks

## Panic Risks (Rust)

| File | Pattern | Risk |
|------|---------|------|
| `lib.rs` | `registry.token.lock().unwrap()` | Mutex poisoning on `cancel_install` |
| `lib.rs` | `app.get_webview_window("main").unwrap()` | Window not found in `setup` |
| `download.rs` | `tempfile::tempdir().unwrap()` | Disk full / permissions |
| `health.rs` | Various unwraps | IO errors during health checks |

The `install_minui_with_cancel` callbacks use `if let Err(e) = ...` for event emission (non-panicking).

## Mutex Patterns

```rust
// Pattern 1: unwrap (panic on poison) — main thread
let mut slot = registry.token.lock().unwrap();

// Pattern 2: if let Ok (silent on poison) — spawned tasks
if let Ok(mut slot) = registry_for_task.token.lock() { ... }
```

## `unsafe` Usage

One `unsafe` block in `src-tauri/src/fs_utils.rs`:
```rust
unsafe { libc::statvfs(path.as_ptr() as *const i8, &mut stat) }
```
Well-audited FFI for disk space on Unix.

## Debug Logging in Production

| File | Statement | When |
|------|-----------|------|
| `lib.rs` | `eprintln!("Warning: failed to emit install progress event: {}", e)` | Event emit failure |
| `extract.rs` | `eprintln!("Warning: failed to remove temp archive: {}", e)` | Temp cleanup failure |
| `install.rs` | `eprintln!("Warning: failed to write version metadata: {}", e)` | Metadata write failure |
| `pipeline.rs` | `eprintln!("create_target_within: cleanup failed ...")` | Security cleanup failure |

All non-fatal warnings. Consider surfacing to UI or redirecting to log file in production.

## Frontend `console` Usage

- `src/hooks/useVersionCheck.ts` — `console.error` on version check failure
- No `console.log` or `console.warn` in production code (excluding tests)

## TypeScript `any` Usage

Minimal — only in `scripts/discover-packages.ts` for GitHub API JSON. No `@ts-ignore` or `@ts-expect-error` in `src/`.

## TODOs & FIXMEs

Zero `TODO`, `FIXME`, `HACK`, `XXX`, `WORKAROUND`, or `BUG` annotations found. Tracked work in GitHub Issues.
