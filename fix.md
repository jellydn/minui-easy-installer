# Fix Plan: `refactor/thermonuclear-code-quality`

## Context

The review identified 18 findings across the Rust backend and React frontend, ranked by impact. The headline finding (#1) is a **functional bug** that masquerades as a code-style comment: `lib.rs` drops `TempDir` returns from `download_archive` / `extract_archive`, deleting files the rest of the install pipeline still needs to read. The second headline (#2) is a structural duplication between `install.rs` and `package.rs` that a `Pipeline` extraction can collapse.

## Validation Summary

| #   | Severity | Valid?     | Item                                                          |
| --- | -------- | ---------- | ------------------------------------------------------------- |
| 1   | Critical | ✅         | `lib.rs` drops `TempDir` mid-pipeline (real bug)              |
| 2   | Critical | ✅         | Duplicate install pipelines in `install.rs` / `package.rs`    |
| 3   | High     | ✅         | `(Result, TempDir)` tuple return type is easy to mishandle    |
| 4   | High     | ✅         | `install_minui` 5× `progress()` boilerplate                   |
| 5   | High     | ✅         | `package.rs` path-traversal validation is 4 ad-hoc steps      |
| 6   | High     | ✅         | React install state fragmented in `Home.tsx`                  |
| 7   | Medium   | ✅         | `_platform` parameter on `copy_base_files` is unused          |
| 8   | Medium   | ✅         | `try_install_extras` re-implements `install_minui` body       |
| 9   | Medium   | ✅         | `ProgressCallback` `Arc<dyn Fn>` for no clear reason          |
| 10  | Medium   | ⚠️ Partial | `validate.rs` / `health.rs` may not be in diff scope — verify |
| 11  | Medium   | ✅         | `PackageStore.tsx` install state map is fragile               |
| 12  | Medium   | ✅         | Test setup duplicates `invoke` mock across files              |
| 13  | Medium   | ✅         | Plaintext warning buried in `WifiWizard.tsx`                  |
| 14  | Medium   | ✅         | Audit `version.rs` for residual string-compare paths          |
| 15  | Medium   | ✅         | `drives.rs` macOS uses `df -k` heuristic                      |
| 16  | Low      | ✅         | Redundant optimistic "installing" map in `handleInstallAll`   |
| 17  | Low      | ✅         | Two path-traversal checks in `install_package`                |
| 18  | Low      | ✅         | `is_preserved_path_nested` test lacks case-folding comment    |

17 valid, 1 partial (#10 needs verification of diff scope before acting).

---

## Critical (must fix — correctness regressions)

### #1 — `lib.rs` Tauri handlers drop `TempDir` and delete artifacts mid-pipeline

**Files:** `src-tauri/src/lib.rs:25-34, 41-50`, `src-tauri/src/install.rs:243-244, 256-257, 280-282`, `src-tauri/src/package.rs:107, 122`

**Problem**

```rust
// src-tauri/src/lib.rs
let (result, _temp_dir) = download::download_archive(&url, checksum_ref).await?;
// _temp_dir drops here, cleaning up the downloaded file
Ok(result)
```

`download_and_verify_archive` and `extract_archive_to_directory` are dead in the frontend (the file path they return points at a file that no longer exists by the time JS reads it), and `install_minui` / `install_package` in `lib.rs` silently drop the temp dir from the inner `download::download_archive` calls — so by the time `extract::extract_archive` runs on `base_result.file_path`, the file is already gone. The same pattern repeats three more times inside `install.rs`.

This is not a code-style issue. The comment "this cleans up the downloaded file" is documenting a bug as a feature.

**Fix — code-judo: introduce an `InstallSession` that owns all temp dirs**

Replace the `(Result, TempDir)` tuple API with a single struct that lives for the lifetime of the install:

```rust
// src-tauri/src/pipeline.rs (new module)
pub struct InstallSession {
    _base_archive: Option<TempDir>,
    _base_extracted: Option<TempDir>,
    _extras_archive: Option<TempDir>,
    _extras_extracted: Option<TempDir>,
    _package_archive: Option<TempDir>,
    _package_extracted: Option<TempDir>,
}

impl InstallSession {
    pub fn new() -> Self { /* all None */ }
    pub fn keep_archive(&mut self, t: TempDir) -> &Path { /* store + expose */ }
    pub fn keep_extracted(&mut self, t: TempDir) -> &Path { /* store + expose */ }
    // Drop runs cleanup at end of install
}
```

`download.rs` and `extract.rs` get new "into" variants that fill a session slot:

```rust
pub async fn download_archive_into(
    out: &mut Option<TempDir>,
    url: &str,
    checksum: Option<&str>,
) -> Result<DownloadResult, String>;

pub fn extract_archive_into(
    out: &mut Option<TempDir>,
    archive_path: &Path,
    destination: Option<&Path>,
) -> Result<ExtractionResult, String>;
```

`install_minui` then reads top-to-bottom with no `let (_, _temp) = …` lines:

```rust
let mut session = InstallSession::new();
let base_path = download::download_archive_into(
    &mut session._base_archive, &options.base_url, options.base_checksum.as_deref(),
).await?;
let base_extracted = extract::extract_archive_into(
    &mut session._base_extracted, &base_path, None,
)?;
copy_base_files(&base_extracted, &options.sd_mount, &options.platform)?;
// session drops at end of fn, cleaning up temps atomically
```

This eliminates:

- All `let (_, _temp) = …` boilerplate at 4 call sites
- The lifetime hazard of accidentally dropping a temp dir early
- The 4 places that document the bug as a "feature"

**Tests:** add a test that proves a downloaded archive file is still readable from the path returned by `download_archive_into` after the call returns, and still exists after `install_minui` returns successfully (cleanup at the end is correct).

### #2 — Two near-duplicate install pipelines (install.rs vs package.rs)

**Files:** `src-tauri/src/install.rs:228-310`, `src-tauri/src/package.rs:101-160`, plus duplicated `test_copy_dir_recursive_copies_files` in both modules

**Problem**

Both functions implement the same `download → extract → copy` skeleton. The package version differs only in (a) what target dir it uses and (b) that it has no extras step. The test `test_copy_dir_recursive_copies_files` is duplicated verbatim across both modules.

**Fix — extract a `Pipeline::run` helper**

```rust
// src-tauri/src/pipeline.rs
pub struct Pipeline;

impl Pipeline {
    pub async fn run<Cp>(
        label: &str,                         // "base" | "extras" | "package"
        url: &str,
        checksum: Option<&str>,
        copy: Cp,                            // &Path -> Result<u32, String>
        progress: ProgressCallback,
        session: &mut InstallSession,        // fills the right slots
    ) -> Result<u32, String>
    where
        Cp: FnOnce(&Path) -> Result<u32, String>,
    {
        progress(InstallProgressEvent { step: "download".into(),
            details: format!("Downloading {} archive", label) });
        let path = download::download_archive_into(
            session.slot_for(label, "archive"), url, checksum).await?;

        progress(InstallProgressEvent { step: "extract".into(),
            details: format!("Extracting {} archive", label) });
        let extracted = extract::extract_archive_into(
            session.slot_for(label, "extracted"), &path, None)?;

        progress(InstallProgressEvent { step: "copy".into(),
            details: format!("Copying {} files", label) });
        copy(Path::new(&extracted.output_path.unwrap()))
    }
}
```

`install_minui` becomes three calls:

```rust
let base_copied = Pipeline::run("base",
    &options.base_url, options.base_checksum.as_deref(),
    |p| copy_base_files(p.to_str().unwrap(), &options.sd_mount, &options.platform),
    progress.clone(), &mut session).await?;

if let Some(url) = options.extras_url.as_deref() {
    match Pipeline::run("extras", url, options.extras_checksum.as_deref(),
        |p| copy_extras_files(p.to_str().unwrap(), &options.sd_mount, &options.extras_platform),
        progress.clone(), &mut session).await
    {
        Ok(n) => extras_files_copied = n,
        Err(e) => extras_warning = Some(e),
    }
}
```

`install_package` becomes one call + the path-validation helper from #5.

**Tests:** delete one of the two `test_copy_dir_recursive_copies_files` tests; the helper itself is in `fs_utils` and gets a single dedicated test there. Add a `test_pipeline_runs_download_extract_copy_end_to_end` covering the helper with a stub download/extract.

---

## High priority — structural code quality

### #3 — `download.rs` and `extract.rs` return `(T, TempDir)` tuples that are easy to mishandle

**Files:** `src-tauri/src/download.rs`, `src-tauri/src/extract.rs`

**Problem**

The `Result, TempDir` API is fragile — every call site must bind both or use `let _ = …`. The dropped-temp comment in `lib.rs` is direct evidence the API invites the wrong behavior.

**Fix:** Subsumed by #1. The pipeline takes `&mut Option<TempDir>` slots to fill, so the helper signatures lose the second tuple element.

### #4 — `install_minui` repeats the same `progress + map err` pattern five times

**Files:** `src-tauri/src/install.rs:228-310`

**Problem**

The function is 80 lines of orchestration with three places that map errors into the same `InstallResult { success: false, error: Some(...) }` shape. The non-fatal `extras` branch is the only one that actually uses `Result` directly.

**Fix:** The `Pipeline::run` helper from #2 absorbs the download+extract orchestration. The remaining top-level orchestration in `install_minui` becomes a `?`-driven flow that only handles the cross-cutting concerns (build session, run pipeline 1×, run pipeline 2×, create ROM dirs, write version metadata). All three `Ok(InstallResult { success: false, … })` early returns become one `?` each.

### #5 — `package.rs::install_package` security validation is 4 ad-hoc steps

**Files:** `src-tauri/src/package.rs:124-153`

**Problem**

Path-traversal string check → canonicalize SD → create-dir → canonicalize pak → starts_with check → 30 lines of security logic. The "early `..` check prevents …" comment is exactly the kind of magic comment that signals the design is wrong.

**Fix:** Extract a single helper that returns a validated, created `PathBuf`:

```rust
// src-tauri/src/install.rs (or fs_utils.rs)
pub fn create_target_within(
    sd_mount: &Path,
    target_dir: &str,
    platform: &str,
    pak_name: &str,
) -> Result<PathBuf, String> {
    let canonical_sd = sd_mount.canonicalize()
        .map_err(|e| format!("Failed to resolve SD card path: {}", e))?;
    let target = sd_mount
        .join(target_dir.trim_start_matches('/'))
        .join(platform)
        .join(format!("{}.pak", pak_name));
    fs::create_dir_all(&target)
        .map_err(|e| format!("Failed to create package directory: {}", e))?;
    let canonical = target.canonicalize()
        .map_err(|e| format!("Failed to resolve package path: {}", e))?;
    if !canonical.starts_with(&canonical_sd) {
        return Err(format!("Security violation: target escapes SD card: {}", target.display()));
    }
    Ok(canonical)
}
```

Body of `install_package` becomes:

```rust
let pak_root = create_target_within(Path::new(sd_mount), &rules.target_dir, platform, &rules.pak_name)?;
let files_copied = fs_utils::copy_dir_recursive(extracted, &pak_root, &|_s, _d| false)?;
```

### #6 — React install state is fragmented across many `useState` calls in `Home.tsx`

**Files:** `src/Home.tsx`

**Problem**

10+ useState calls for install progress fields. Errors and result states drift independently. `InstallProgress` already receives typed `InstallProgressEvent`s, so the state shape can mirror those events one-to-one.

**Fix:** Consolidate into a single `useReducer` (small state machine: `idle → downloading → extracting → copying → done | error`). Pair this with #11 so `PackageStore.tsx` uses the same reducer pattern.

---

## Medium priority

### #7 — Unused `_platform` parameter on `copy_base_files`

**Files:** `src-tauri/src/install.rs:91`

**Fix:** Delete the parameter from the signature and from the caller in `install_minui`. The platform logic now flows through the extracted directory's own contents.

### #8 — `try_install_extras` re-implements `install_minui`'s body

**Files:** `src-tauri/src/install.rs:189-227`

**Fix:** Subsumed by #2. `try_install_extras` becomes `Pipeline::run("extras", …)` with `copy_extras_files` as the copy closure. The function deletes entirely.

### #9 — `ProgressCallback` is an `Arc<dyn Fn>` for no clear reason

**Files:** `src-tauri/src/install.rs:13`, `src-tauri/src/lib.rs:60-66`

**Problem**

`lib.rs` clones the `AppHandle` and moves it into a fresh `Arc` on every install call. Tauri's `Channel` (Tauri 2 native pattern) avoids the Arc dance entirely.

**Fix:** Either (a) keep the callback shape but use a plain `Box<dyn Fn>` since it never needs to be shared between threads, or (b) replace with `tauri::ipc::Channel<InstallProgressEvent>` for a typed, owned channel. Option (b) is the more idiomatic Tauri 2 path; recommend it as a follow-up PR.

### #10 — `validate.rs` / `health.rs` may share filesystem-detection logic

**Files:** `src-tauri/src/validate.rs`, `src-tauri/src/health.rs`

**Valid:** Partial — verify whether `validate.rs` and `health.rs` are in the diff scope (they don't appear in the historical review's modified-file list as having received attention).

**Fix (if both are touched by the branch):** Move the shared helpers into `fs_utils.rs`. Both modules consume the canonical helper.

### #11 — `PackageStore.tsx` install state map is fragile

**Files:** `src/PackageStore.tsx`

**Fix:** Move install status into a `useReducer` keyed by package id, with actions `{started, finished, failed}`. State shape mirrors the install lifecycle.

### #12 — Test setup duplicates `invoke` mock across files

**Files:** `src/types/*.test.ts`, `vitest.setup.ts`

**Fix:** Hoist the common `invoke` mock into `vitest.setup.ts` and delete per-file mocks. Verify after the change that no test lost its specific stub.

### #13 — Plaintext warning is buried in `WifiWizard.tsx`

**Files:** `src/WifiWizard.tsx`

**Problem**

A `type="password"` input is the only UI guard against plaintext-on-SD. The diff added a plaintext warning (per the historical review), but it should be a top-level callout near the input — not a footnote.

**Fix:** Promote the warning to a visible banner above the password field. Verify it renders for both `add` and `edit` flows. Add a test asserting the warning text is present in the rendered output.

### #14 — Audit `version.rs` for residual string-compare paths

**Files:** `src-tauri/src/version.rs`, `src-tauri/src/package.rs::check_package_updates`

**Problem**

The historical review notes semver comparison was added. Confirm `compare_versions` is the only path used in both `check_package_updates` and `check_minui_version`, and that the old `>` operator path is gone.

**Fix:** Audit both call sites; delete any remaining string-compare code path. Add a regression test that pins the new behavior for non-date versions (e.g., "0.12.0" vs "0.9.0").

### #15 — `drives.rs::list_removable_drives` on macOS still uses `df -k` heuristic

**Files:** `src-tauri/src/drives.rs:15-66`

**Problem**

`df -k` output is locale-dependent and includes the Macintosh HD if mounted under `/Volumes/`.

**Fix:** Use `diskutil info -plist` (Apple-blessed, structured output) on macOS. Keep `df -k` only as a fallback. Add a test for a sample `diskutil` plist output.

---

## Low priority

### #16 — Redundant optimistic "installing" map in `handleInstallAll`

**Files:** `src/PackageStore.tsx`

**Fix:** Delete the optimistic map; rely on the reducer state from #11.

### #17 — Two path-traversal checks in `install_package`

**Files:** `src-tauri/src/package.rs:128-148`

**Problem**

A string `..` check runs _and_ a `canonicalize` + `starts_with` check runs. The string check is the kind of heuristic the diff was supposed to delete in favor of canonicalize.

**Fix:** Delete the `..` string check. The canonicalize-based check (consolidated into `create_target_within` per #5) covers the same case more robustly.

### #18 — `is_preserved_path_nested` test mixes case rules and case-insensitivity

**Files:** `src-tauri/src/install.rs:tests::test_is_preserved_path_nested`

**Fix:** Add a one-line comment in the test explaining the case-insensitivity originates from `eq_ignore_ascii_case` and the SD card's FAT32 filesystem.

---

## Non-issues (intentionally left as-is)

- The `windows-sys` removal in `Cargo.toml` is correct; the conditional gating in `drives.rs` is fine for MVP.
- The `version` field in `InstallResult` was correctly removed (installer doesn't write version metadata per AGENTS.md).
- The `unknown` → typed enum in `package.ts` is a real improvement; no follow-up needed.

---

## Suggested order of work

1. **#1 + #3 together** — introduce `InstallSession`, simplify download/extract signatures. One PR-sized change; unblocks #2, #4, #8.
2. **#2 + #5 + #8** — extract `Pipeline::run`, delete `try_install_extras`, simplify `install_package` security check via `create_target_within`.
3. **#6 + #11 + #16** — consolidate React install state into a single reducer in `Home.tsx` and `PackageStore.tsx`.
4. **#7** — delete unused `_platform` (one-line cleanup).
5. **#10, #13, #14, #15, #17, #18** — low-risk, single cleanup PR.
6. **#9** — Tauri `Channel` migration. Separate PR; requires Tauri 2 `Channel` knowledge.

## Validation gates

- After #1 lands: add a test that proves a downloaded archive file is still readable from the path returned by `download_archive_into` after the call returns.
- After #2 lands: `cargo test` passes with one fewer `test_copy_dir_recursive_copies_files` and a new `test_pipeline_runs_end_to_end` test.
- After #6 lands: `bun test` passes with the new reducer.
- Frontend `bun run typecheck` and `bun run lint` must remain clean throughout.
- `cargo clippy --all-targets -- -D warnings` and `bun test` run as a final gate on the cleanup PR.
