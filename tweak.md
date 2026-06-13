# Per-Device Extras Install

**Context**: MinUI extras archives contain Emus, Tools, and Bios folders for ALL supported platforms. The previous code copied the entire extras archive to SD root, dumping every platform's files instead of just the matching platform's.

## Changes

### `src/types/device.ts`

- Added `extrasPlatform` field to `DeviceProfile` interface
- Mapped extras folder names per device (some differ from base archive names):
  - `trimui` (base) → `trimuismart` (extras) — TrimUI Brick/Smart Pro
  - `miyoo`/`miyoo285` (base) → `miyoomini`/`my282` (extras) — Miyoo Mini/Mini+/A30
  - Most others use the same name for both

### `src/types/install.ts`

- Replaced `extrasDir` parameter with `extrasPlatform` in `installMinui()`

### `src/Home.tsx`

- Passes `profile.extrasPlatform` to `installMinui()` in both install and update-all flows

### `src-tauri/src/install.rs`

- `copy_extras_files()` now filters extras by platform:
  - Copies `Bios/` → SD `Bios/` (shared across devices)
  - Copies `Emus/{extras_platform}/` → SD `Emus/{extras_platform}/`
  - Copies `Tools/{extras_platform}/` → SD `Tools/{extras_platform}/`
  - Ignores other platforms' folders entirely
- Removed unused `extras_dir` parameter from `install_minui()` and `try_install_extras()`
- Added `extras_platform` parameter throughout

### `src-tauri/src/lib.rs`

- Added `extras_platform` parameter to `install_minui` Tauri command
- Removed `extras_dir` parameter

## Validation

- All 49 Rust tests pass (including new `test_copy_extras_files_filters_by_platform`)
- TypeScript typecheck passes
- Lint passes (0 warnings, 0 errors)
