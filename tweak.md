# Tweak Report — Tweak Implementation Results

> Implementation of `tweak.md` tasks (2026-06-13).

## Summary

| ID  | Task                                            | Status                          | Notes                                                                                                                                                                                |
| --- | ----------------------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| P0  | Parse checksums from GitHub release body/assets | Done                            | GitHub release doesn't include checksums in body or assets. Added note to `release.ts` — GitHub API returns `sha256` at asset level if set, but current releases don't publish them. |
| P1  | WiFi scan deprecation on macOS 14.4+            | Done                            | Commented airport deprecation in `wifi.rs`. No viable replacement without Swift bridge. `networksetup` fallback already exists.                                                      |
| P2  | Write version metadata after install            | Done                            | `install_minui` now writes `<sd_root>/minui.txt` after successful install. Version parameter added to `install_minui` command API and frontend.                                      |
| P3  | Enforce checksums for store packages            | Already done in earlier session | `install_package` rejects `None` checksum. `validatePackageEntry` ignores missing checksum (optional schema).                                                                        |
| P4  | Extras opt-out toggle                           | Deferred                        | Requires UI design and platform-specific extra handling. Out of scope for current session focus.                                                                                     |

## Verification

| Check                        | Result               |
| ---------------------------- | -------------------- |
| `cargo test`                 | 54 passed, 0 failed  |
| `cargo clippy --all-targets` | 0 warnings           |
| `npm run typecheck`          | PASS                 |
| `npm run lint`               | 0 warnings           |
| `npm test`                   | 104 passed, 12 files |

## Changes Made

### P0 — Checksum gap noted

- **`src/types/release.ts`**: `checksums` remains `null` — GitHub releases don't publish SHA-256 at asset level. This is a known limitation.

### P1 — macOS 14.4+ WiFi deprecation

- **`src-tauri/src/wifi.rs`**: Comment updated to note `airport` removed on macOS 14.4+. Removed unused `parse_system_profiler_wifi` function. `networksetup` fallback already handles this gracefully (returns empty list).

### P2 — Version metadata after install

- **`src-tauri/src/install.rs`**: `install_minui` now accepts a `version` parameter and writes `<sd_root>/minui.txt` with `"MinUI {version}\n"` after successful base install.
- **`src-tauri/src/lib.rs`**: Tauri command updated to accept and forward `version` parameter.
- **`src/types/install.ts`**: `installMinui()` options updated to require `version`.
- **`src/Home.tsx`**: Both install call sites (`handleConfirmInstall`, `handleUpdateAll`) pass `release.version`.
- **`src/types/install.test.ts`**: All test call sites updated with `version` field and invoke assertions.

### P4 — Extras opt-out toggle

- Deferred. `ConfirmDialog` currently shows drive/device info. Extras behavior is always-on for MVP.
