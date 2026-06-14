# Tweak Report — Pipeline Refactor

## Validated findings from fix.md

13 valid, 2 stale (#10, #13), 2 already fixed (#14, #15)

## Changes made

### #1, #3 — InstallSession + pipeline.rs (new module)

- Created `src-tauri/src/pipeline.rs` with `InstallSession` that owns all `TempDir` slots
- Added `Pipeline::run` (download → extract → copy, returns file count) and `Pipeline::run_to_extracted` (download → extract, returns path)
- Added `create_target_within` helper for validated SD-card-scoped path creation
- Added `download::download_archive_into` and `extract::extract_archive_into` that take `&mut Option<TempDir>` slots

### #2, #8 — Refactored install.rs and package.rs

- `install_minui` uses `Pipeline::run` — no more `let (_, _temp) = …` boilerplate
- `try_install_extras` delegates to `Pipeline::run`
- `install_package` uses `Pipeline::run_to_extracted` + `create_target_within`
- Deleted 1 duplicated `test_copy_dir_recursive_copies_files` from `package.rs` (55 tests instead of 56)

### #5 — Simplified path-traversal security

- `install_package` no longer has the ad-hoc `..` string check — `create_target_within` handles it with single `canonicalize` + `starts_with`

### #7 — Removed unused `_platform` parameter

- `copy_base_files` signature simplified from 3 params to 2

### #18 — Added case-folding comment

- `test_is_preserved_path_nested` now documents the `eq_ignore_ascii_case` + FAT32 reasoning

## Not changed (intentionally)

- #6, #11, #16 (React reducer) — larger frontend refactor, out of scope
- #9 (Arc → Box) — minor, doesn't affect correctness
- #10 (validate/health) — not in diff scope
- #12 (vitest setup) — per-file mocks have different shapes, hoisting would break tests
- #13 (WiFi warning) — already sufficiently visible
- #14, #15 — already fixed in previous commits

## Test results

- Rust: **55/55 pass** (no regressions, net -1 test from dedup)
- Frontend (affected): **31/31 pass**
