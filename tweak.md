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

---

# Tweak Report — BIOS Support (issue #7)

## Goal

Implement the BIOS support request from
[issue #7](https://github.com/jellydn/minui-easy-installer/issues/7).
The user owns the BIOS files (they're copyrighted) and the installer
copies them to the right `Bios/<subdir>/` path on the SD card.

## Catalog (mirrors issue body)

| ID | Subdir | Filename | System |
|---|---|---|---|
| `gb_bios` | GB | `gb_bios.bin` | Game Boy |
| `gbc_bios` | GBC | `gbc_bios.bin` | Game Boy Color |
| `gba_bios` | GBA | `gba_bios.bin` | Game Boy Advance |
| `md_cd_e` / `md_cd_j` / `md_cd_u` | MD | `bios_CD_{E,J,U}.bin` | Sega CD |
| `ps_bios` | PS | `psxonpsp660.bin` | PlayStation |
| `pce_bios` | PCE | `syscard3.pce` | TurboGrafx CD |
| `fc_disksys` | FC | `disksys.rom` | Famicom Disk System |
| `pkm_bios` | PKM | `bios.min` | Pokemon Mini |
| `sgb_bios` | _root_ | `sgb.bios` | Super Game Boy |
| `dc_boot` | DC | `dc_boot.bin` | Sega Dreamcast |
| `dc_naomi` | DC | `naomi.zip` | Dreamcast / Naomi |
| `nds_bios7` | NDS | `bios7.bin` | Nintendo DS ARM7 |
| `nds_bios9` | NDS | `bios9.bin` | Nintendo DS ARM9 |
| `nds_firmware` | NDS | `firmware.bin` | Nintendo DS firmware |

Regression guard: `test_catalog_filenames_match_issue_spec` and
`test_catalog_subdirs_match_issue_spec` pin every filename/subdir to
the issue body so a refactor can't silently break MinUI.

## Rust backend — `src-tauri/src/bios.rs` (new)

- `BiosEntry { id, subdir, filename, description, system }`
- `catalog() -> Vec<BiosEntry>` — the source of truth
- `status(sd_mount) -> Result<BiosStatus, String>` — scans SD card, returns
  per-entry presence
- `install_bios_from_bytes(sd_mount, entry_id, base64_payload) -> Result<String, String>` —
  decodes payload, writes to `Bios/<subdir>/<filename>`, returns the
  written path
- `safe_component` rejects empty-after-strip, NUL bytes, path
  separators, and `.` / `..`
- `canonicalize_existing_ancestor` (private) lifts the existing pattern
  from `pipeline.rs` to the BIOS case

### Security posture

Identical to `pipeline.rs::create_target_within`:

1. Canonicalize SD card root.
2. Build target path from catalog entry + sanitized components.
3. Walk up to the first existing ancestor and canonicalize that.
4. Reject if canonical ancestor is outside the SD card.
5. `create_dir_all` the parent, `fs::write` the file.
6. Canonicalize the written file; reject (and clean up) if it ended up
   outside the SD card.

`test_install_rejects_symlink_escape` proves the symlink-race guard
catches a malicious `Bios/` symlink pointing outside the SD card.

## Frontend — `src/BiosInstaller.tsx` (new) + `src/types/bios.ts` (new)

- New "BIOS" tab in the top nav.
- Catalog is loaded via `list_bios_catalog`; status via `get_bios_status`.
- Each row shows system, description, target path, and a "Installed" /
  "Not installed" badge.
- A hidden `<input type="file">` is per-row; "Choose file" / "Replace"
  triggers `.click()`. The bytes are read with `arrayBuffer()`, encoded
  to base64 in 32 KB chunks (PlayStation BIOS is ~4 MB; chunking avoids
  a stack overflow from a single `String.fromCharCode(...uint8arr)`
  spread), and sent to `install_bios_file`.
- After a successful install the status is refreshed so the badge flips
  to "Installed" and a "Copied `<source filename>`" confirmation shows.
- Errors (e.g. "permission denied", symlink escape) surface inline as
  `error` text without resetting other rows.

## Tests

- **22 new Rust tests** in `bios::tests` + `lib::tests`: catalog
  identity (filenames, subdirs), `safe_component` rejection cases,
  install happy paths (subdir + root), parent-dir creation, overwrite,
  unknown-entry / invalid-base64 / empty-payload / missing-mount
  errors, symlink-escape rejection, and contract tests through the
  Tauri command surface.
- **11 new frontend tests** (`bios.test.ts` + `BiosInstaller.test.tsx`):
  `bufferToBase64` round-trips (empty / 1 MB / binary), catalog load,
  installed count display, error + retry, Back button, install invoke
  wiring, success filename, install error rendering.

## Results

- Rust: **147/147 pass** (was 125; +22 from BIOS work)
- Frontend: **140/140 pass** (was 129; +11 from BIOS work)
- `bun run typecheck`: clean
- `bun run lint`: 0 warnings, 0 errors
- `cargo fmt --check -- src/bios.rs src/lib.rs`: clean
- `cargo clippy --all-targets` on `src/bios.rs`: 0 warnings

## What I deliberately did NOT do

- **No `tauri-plugin-dialog`.** Picking files through a Tauri plugin
  would mean adding a plugin, a new capability, a new dependency, and a
  new permission scope. The hidden `<input type="file">` gets us the
  same UX with zero new infrastructure — the bytes still flow through
  the Tauri command, which still goes through the validated
  `install_bios_from_bytes` path.
- **No symlink dereferencing in Rust.** The current behaviour is to
  reject the write if the canonical target escapes the SD card; that
  matches `pipeline.rs` and `fs_utils.rs` and is what the issue asks
  for. The existing `test_install_rejects_symlink_escape` proves it.
- **No "do not write WiFi passwords" warning for BIOS.** The user is
  intentionally supplying copyrighted files; the UI banner says
  "You are on your own to source these files" instead.

## Future work (not in this issue)

- Streaming chunked upload from the frontend — current code reads the
  whole file into memory in the webview, which is fine for ~4 MB BIOS
  files but not for, say, 64 MB Dreamcast discs.
- Per-device filtering — currently the catalog shows all 11 entries
  regardless of selected device. We could hide PS/PS2 BIOS from a Miyoo
  Mini, etc., but the issue does not ask for that and most users will
  copy what they have regardless.
- A "remove BIOS" button. Skipped — the user can delete files off the
  SD card directly and we don't want to add a write op without a
  confirmation dialog.
