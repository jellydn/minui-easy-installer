# Plan 002 — Add cancel/abort mechanism for install operations

| Field        | Value                                  |
| ------------ | -------------------------------------- |
| Slug         | `cancel-install-mechanism`             |
| Status       | pending                                |
| Priority     | High                                   |
| Category     | UX (missing feature)                   |
| Impact       | High                                   |
| Effort       | M                                      |
| Risk         | Medium (state-machine addition)       |
| Audit commit | `4d6e95a`                              |
| Depends on   | 001 (streaming downloads — cancel must reach the in-flight HTTP body) |
| Blocks       | none                                   |

## Problem

`.planning/codebase/CONCERNS.md` → "Scaling Limits" → "No Cancellation
Mechanism" calls this out: once an install begins, the only recovery
is force-killing the app. A stalled download, a wrong-drive selection,
or a removed SD card mid-copy leaves the user with no recourse.

## Goal

A user clicking "Cancel" in `InstallProgress` aborts the in-flight
download (or the in-flight copy, or the in-flight extraction) within
~1 s, and the UI returns to the idle state with a clear "Install
cancelled" message. No partial writes remain on the SD card; no temp
files leak (the existing `InstallSession` cleanup handles that).

## Files in scope

- `src-tauri/src/install.rs` — create a `CancellationToken` (use
  `tokio_util::sync::CancellationToken`; add the dep), pass it through
  to the Pipeline. Add a new Tauri command `cancel_install` that
  triggers it.
- `src-tauri/src/pipeline.rs` — accept the token in `Pipeline::run`
  and `run_to_extracted`; `tokio::select!` between the download future
  and the cancellation future.
- `src-tauri/src/download.rs` — `download_archive_streaming` (added
  in Plan 001) must check the token between chunks. If cancelled,
  delete the partial file and return `Err("cancelled")`.
- `src-tauri/src/extract.rs` — same: check the token between entries.
  (Extraction is fast for our archives, so cancellation latency
  during extract is small.)
- `src-tauri/src/package.rs` — the package install uses
  `Pipeline::run_to_extracted` + a synchronous copy step; the copy
  step itself uses `std::fs` (blocking) — wrap that in
  `tokio::task::spawn_blocking` and check the token between files.
  See `src-tauri/src/fs_utils.rs::copy_dir_recursive` — add a
  token-check closure parameter.
- `src-tauri/src/lib.rs` — add the `cancel_install` Tauri command and
  register a per-install token store (`Arc<Mutex<Option<CancellationToken>>>`)
  keyed by an install ID.
- `src/types/install.ts` — add `cancelInstall({ installId })` wrapper.
- `src/Home.tsx`, `src/InstallProgress.tsx` — wire the Cancel button.
- `src-tauri/Cargo.toml` — add `tokio-util` with the `rt` feature.

## Files explicitly out of scope

- `src/PackageStore.tsx` — package installs are short enough that
  cancellation UX is not the highest leverage. Add a TODO comment in
  this plan for the next pass.
- `src/WifiWizard.tsx` — the `setTimeout(onComplete, 1500)` race in
  `WifiWizard.tsx:79` is a separate concern; not in scope.

## Current state (excerpt — `src-tauri/src/install.rs::install_minui`)

```rust
pub async fn install_minui(
    options: &InstallOptions,
    progress: ProgressCallback,
) -> Result<InstallResult, String> {
    let mut session = InstallSession::new();
    // ... no cancellation hook, no token, no select
    let base_files_copied = Pipeline::run(/* ... */).await?;
    // ... the user is locked in
}
```

## Step-by-step execution

### Step 1 — Add `tokio-util` dependency

In `src-tauri/Cargo.toml`:

```toml
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["rt"] }
```

(Verify the version matches what other deps pull in via Cargo.lock; if
not, match the lock.)

**Verification:**

```bash
cd src-tauri && cargo build
```

Expected: compiles. `cargo tree | grep tokio-util` shows the dep.

### Step 2 — Token store in `lib.rs`

In `src-tauri/src/lib.rs`, add a `tauri::State` for an
`Arc<Mutex<HashMap<String, CancellationToken>>>` (or
`Arc<Mutex<Option<CancellationToken>>>` if we only ever have one
install at a time — the existing UI does not run concurrent installs
in one window, so the simpler version is fine).

Add commands:

```rust
#[tauri::command]
async fn start_install(
    state: tauri::State<'_, InstallRegistry>,
    /* ...same args as install_minui... */
) -> Result<String, String> {
    let token = CancellationToken::new();
    let id = uuid::Uuid::new_v4().to_string();
    state.registry.lock().unwrap().insert(id.clone(), token.clone());

    let options = /* ...build... */;
    let handle = tokio::spawn(async move {
        install::install_minui_with_cancel(&options, progress, token).await
    });
    // store handle too if we want to await it from cancel
    Ok(id)
}

#[tauri::command]
fn cancel_install(
    state: tauri::State<'_, InstallRegistry>,
    install_id: String,
) -> Result<(), String> {
    if let Some(token) = state.registry.lock().unwrap().remove(&install_id) {
        token.cancel();
    }
    Ok(())
}
```

> **Note for executor:** if you would rather keep `install_minui` and
> add `install_minui_with_cancel` (the new variant takes the token),
> the cancel command just cancels the token. The frontend can hold the
> `installId` for the duration of the call. **Prefer this variant** —
> it changes fewer call sites.

**Verification:**

```bash
cd src-tauri && cargo build
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Expected: clean compile, all existing tests pass.

### Step 3 — `Pipeline` accepts the token

In `src-tauri/src/pipeline.rs`:

```rust
pub async fn run<Cp>(
    label: &str,
    url: &str,
    checksum: Option<&str>,
    copy: Cp,
    progress: ProgressCallback,
    cancel: CancellationToken,    // new
    session: &mut InstallSession,
) -> Result<u32, String> {
    tokio::select! {
        result = Self::run_to_extracted(/* ... */) => { result?; /* copy */ }
        _ = cancel.cancelled() => Err("Install cancelled".to_string()),
    }
}
```

`copy` is synchronous today. Wrap it: `tokio::task::spawn_blocking`
returning a `JoinHandle<Result<u32, String>>`, then `tokio::select!`
between the handle and the token.

**Verification:**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Expected: still all green.

### Step 4 — Streaming + copy respect the token

In `src-tauri/src/download.rs`, inside the streaming loop, check
`cancel.is_cancelled()` every Nth chunk (say, every 64 KB). If
cancelled, delete the partial file via `fs::remove_file(&file_path)`
and return `Err("cancelled")`.

In `src-tauri/src/fs_utils.rs::copy_dir_recursive`, accept an
optional `cancel: &dyn Fn() -> bool` parameter. The closure is cheap
to check; the recursion hits it once per file.

**Verification:**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Expected: still all green.

### Step 5 — Frontend wiring

In `src/types/install.ts`:

```ts
let activeInstallId: string | null = null;

export async function installMinuiWithCancel(
  options: InstallOptions,
): Promise<InstallResultEither> {
  const { invoke } = await import("@tauri-apps/api/core");
  activeInstallId = await invoke<string>("start_install", options);
  // ... await the result via a separate invoke pattern, or...
}
```

> **Design choice for executor:** the simplest pattern is
> `start_install` returning the id immediately and a `get_install_result`
> command that returns the result when ready. Or use Tauri events
> (`install-complete` and `install-error`) for the result. **Pick one
> and document it in the plan's PR description.**

Add a `Cancel` button in `src/InstallProgress.tsx` visible during
`downloading` / `extracting` / `copying` phases. Click handler calls
`cancelInstall({ installId: activeInstallId })`.

**Verification:**

```bash
bun run typecheck
bun run lint
bun test 2>&1 | tail -10
```

Expected: clean.

### Step 6 — Tests

In `src-tauri/src/install.rs` `mod tests`:

1. `test_install_cancellation_aborts_quickly` — start a "download"
   from a slow local HTTP server (delay each chunk by 50 ms, 100
   chunks = 5 s total), cancel after 200 ms, assert the future
   returns `Err` within 500 ms.
2. `test_cancel_after_download_finishes_is_noop` — cancel after
   download has completed; assert the function continues to extract
   and copy (cancellation is one-shot, not idempotent on success).
3. `test_copy_dir_recursive_respects_cancel` — write a 1000-file
   src tree, cancel after the closure reports `true` on the 5th
   call, assert the function returns Err and the dst has only ~4
   files.

**Verification:**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Expected: 3 new tests pass; total Rust tests now ≥ 58.

## Done criteria (machine-checkable)

- `cd src-tauri && cargo test --lib` shows ≥ 58 passing tests.
- `bun run typecheck` and `bun run lint` pass.
- Manual: start an install, click Cancel during the download phase,
  the install aborts within 1 s and the UI returns to idle.
- Manual: cancel during the copy phase, the SD card shows no partial
  writes (the `Pipeline` rolls back via `InstallSession` drop +
  the `copy_subtree` helper already returns 0 for the partial subtree).

## Test plan

- Unit tests (Step 6) cover the three cancellation points.
- A future integration test (out of scope) can wire a real Tauri
  command and assert the event flow.

## Maintenance note

The cancellation pattern (`CancellationToken` + `tokio::select!`) is
the standard idiom. Any new long-running Tauri command should adopt
the same pattern; reject PRs that add new `tokio::spawn(async move { ... })`
without a token. A grep of `tokio::spawn` should never grow past
~3–4 call sites in this codebase.

## Escape hatches

- **If `tokio-util` 0.7 isn't already in the lockfile tree:** match
  whatever version `tokio` pulls in transitively, or use
  `tokio::sync::Notify` as a poor-man's token (polling-based, less
  ergonomic, but no new dep). Don't pin a different major version.
- **If the `copy` step proves too coarse to cancel mid-directory:**
  the worst case is "we finish copying the current directory, then
  return Err". That's still bounded and acceptable for a 1-2 s UX
  delay. Document the latency in the PR.
- **If the frontend's existing `install-progress` listener can't be
  extended to handle a "cancelled" event cleanly:** add a new event
  `install-cancelled` and let the existing listener ignore it. Do
  NOT remove or rename the existing event.

## Reference

- `.planning/codebase/CONCERNS.md` → "Scaling Limits" → "No Cancellation
  Mechanism" lists the user-visible symptoms.
- `.planning/codebase/CONCERNS.md` → "Performance Bottlenecks" → "No
  Download Progress Reporting" — Plan 001 covers the byte-level
  progress; this plan covers the kill switch.
