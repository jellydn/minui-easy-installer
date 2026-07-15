# Technical Concerns

## File Size & Complexity

All previously flagged files have been addressed:

| File | Before | After | Resolution |
|------|--------|-------|------------|
| `install_tests.rs` | 789 | Split into 3 files (137 + 478 + 186) | Split by concern: pipeline / copy / extras |
| `lib.rs` | 647 | 335 | Contract tests extracted to `lib_tests.rs` |
| `wifi.rs` | 535 | Split into 4 files (133 + 165 + 87 + 63) | Platform modules: `wifi/{macos,linux,windows}.rs` |
| `package.ts` | 455 | 236 | Registry conversion extracted to `registry-convert.ts` |
| `install.rs` | 429 | 504 | InstallPlan inlined phase logic (acceptable growth) |

No files currently exceed 300 lines in the main source (excluding test-only files and platform modules with inline tests).

## Panic Risks (Rust)

Several `.unwrap()` calls exist in production code (not just tests):

| File | Pattern | Risk |
|------|---------|------|
| `lib.rs` | `registry.token.lock().unwrap()` | Mutex poisoning on `cancel_install` |
| `lib.rs` | `app.get_webview_window("main").unwrap()` | Window not found in `setup` |
| `download.rs` | `tempfile::tempdir().unwrap()` | Disk full / permissions |
| `health.rs` | Various unwraps | IO errors during health checks |

**Note:** The `install_minui_with_cancel` callbacks use `if let Err(e) = ...` for event emission (non-panicking), which is the correct pattern.

## Mutex Patterns

`lib.rs` uses two different patterns:

```rust
// Pattern 1: unwrap (panic on poison)
let mut slot = registry.token.lock().unwrap();

// Pattern 2: if let Ok (silent on poison)
if let Ok(mut slot) = registry_for_task.token.lock() {
    *slot = None;
}
```

The second pattern is used in the `tokio::spawn` cleanup path — it's safer because a Mutex poison in a spawned task shouldn't crash the app. The first pattern is fine for the main thread (poison here means the app is in an unrecoverable state).

## `unsafe` Usage

One `unsafe` block in `src-tauri/src/fs_utils.rs` (line 17-18):

```rust
unsafe { libc::statvfs(path.as_ptr() as *const i8, &mut stat) }
```

This is for calling `libc::statvfs` to get disk space on Unix. It's a well-audited pattern, but any change here should be carefully reviewed.

## Debug Logging in Production

| File | Statement | When |
|------|-----------|------|
| `lib.rs` | `eprintln!("Warning: failed to emit install progress event: {}", e)` | Event emit failure |
| `extract.rs` | `eprintln!("Warning: failed to remove temp archive: {}", e)` | Temp cleanup failure |
| `install.rs` | `eprintln!("Warning: failed to write version metadata: {}", e)` | Metadata write failure |
| `pipeline.rs` | `eprintln!("create_target_within: cleanup failed for escaped path {}: {}", ...)` | Security cleanup failure |

These are all non-fatal warnings — they log to stderr but don't crash. Consider whether these should be surfaced to the UI or redirected to a log file in production builds.

## Frontend `console` Usage

- `src/hooks/useVersionCheck.ts` — `console.error` on version check failure

No `console.log` or `console.warn` in production code (excluding tests).

## TypeScript `any` Usage

Minimal — only a well-documented instance in scripts (`scripts/discover-packages.ts`) for the GitHub API JSON response. No `@ts-ignore` or `@ts-expect-error` directives in the main `src/` codebase.

## Known Gaps

| Area | Gap | Priority |
|------|-----|----------|
| Linux support | Drive detection exists but not tested in CI | Low (not MVP) |
| macOS 14.4+ | `airport` WiFi scanning restored via `system_profiler` fallback — `parse_system_profiler_networks()` extracts all visible networks from "Other Local Wireless Networks" section | ✅ Resolved |
| Windows formatting | `format_drive` returns error — not yet implemented | Low |
| Package registry | Hardcoded versions in `store.json` — mitigated by daily cron auto-update | Low |
| Install cancellation | Frontend cancel button wired up via `useForkInstall` → `cancel_install` IPC | ✅ Resolved |

## TODOs & FIXMEs

No `TODO`, `FIXME`, `HACK`, `XXX`, `WORKAROUND`, or `BUG` annotations found in the codebase. This is a positive signal — tracked work is managed in GitHub Issues rather than inline comments.

## Resolved Concerns

These items from the previous architecture review have been addressed:

| Concern | Resolution |
|---------|------------|
| `install_minui` gated behind `#[cfg(test)]` | Fixed — restored with compile-time guard test |
| Shallow Tauri command layer (9 individual params) | Fixed — `InstallOptions` now accepted as single struct |
| Module-level cache globals in `package.ts` | Fixed — `RegistryCache` class with encapsulated TTL |
| `install_minui_with_cancel` inline orchestration | Improved — extracted `install_base()` + `write_version_metadata()` |
| `drives.rs` platform sprawl (503 lines) | Fixed — extracted `macos.rs` submodule |
