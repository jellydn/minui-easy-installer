# Plan 001 — Stream archive downloads and emit progress events

| Field        | Value                                  |
| ------------ | -------------------------------------- |
| Slug         | `stream-archive-downloads`             |
| Status       | pending                                |
| Priority     | High                                   |
| Category     | performance / UX                       |
| Impact       | High                                   |
| Effort       | M                                      |
| Risk         | Medium (touches core download path)    |
| Audit commit | `4d6e95a`                              |
| Depends on   | 003, 004 (test safety net)             |
| Blocks       | 002 (cancel mechanism)                 |

## Problem

`src-tauri/src/download.rs:65` uses `response.bytes().await` to download
the entire archive into memory before writing to disk. MinUI archives
can be 500 MB+ (per `.planning/codebase/CONCERNS.md` → "Performance
Bottlenecks" → "Full Archive Downloaded Into Memory"). This:

- Risks OOM on memory-constrained machines (32-bit support dropped
  recently; macOS 10.15+ floor still ships in low-RAM configs).
- Provides no download progress — the UI only ever shows
  `"Downloading base.zip"` with no percentage.
- Makes cancellation impossible: the bytes are already in RAM before
  the user could signal cancel.

## Goal

Stream each archive directly from the HTTP response into a temp file,
emitting a progress event for every N MB (or every 5 % of the
Content-Length, whichever is smaller) on the `install-progress` channel
the frontend already listens on.

## Files in scope

- `src-tauri/src/download.rs` — refactor `download_archive` and
  `download_archive_into` to stream; add a streaming variant or make
  the existing API stream by default.
- `src-tauri/src/install.rs` — wire progress events through `Pipeline::run`
  (and `run_to_extracted`).
- `src-tauri/src/package.rs` — same wiring for package installs.
- `src-tauri/src/pipeline.rs` — accept a per-byte progress callback in
  `Pipeline::run` / `run_to_extracted`.
- `src/types/install.ts` — extend `InstallProgressEvent` with
  `currentBytes` / `totalBytes` (optional fields; existing consumers
  keep working).
- `src/InstallProgress.tsx` — render a progress bar when bytes are known.
- `src-tauri/src/lib.rs` — no change to the command handlers themselves
  (events already flow through `app_handle.emit`).

## Files explicitly out of scope

- `src-tauri/src/extract.rs` — extraction is already streaming (8 KB
  buffer). Do not touch.
- `src/Home.tsx`, `src/PackageStore.tsx` — they already listen on
  `install-progress`; they'll pick up the new fields automatically.
- Any change to the public Tauri command signatures.

## Current state (excerpt — `src-tauri/src/download.rs:60-75`)

```rust
let bytes = response
    .bytes()
    .await
    .map_err(|e| format!("Failed to read response bytes: {}", e))?;

fs::write(&file_path, &bytes).map_err(|e| format!("Failed to write archive to disk: {}", e))?;
```

The `Pipeline::run` wrapper in `pipeline.rs:42-58` emits a single
`copy` progress event after `run_to_extracted` returns; it never sees
bytes flowing through.

## Step-by-step execution

### Step 1 — Add a streaming download primitive

In `src-tauri/src/download.rs`, add a new function:

```rust
pub async fn download_archive_streaming(
    slot: &mut Option<TempDir>,
    url: &str,
    expected_checksum: Option<&str>,
    progress: impl Fn(u64, Option<u64>) + Send + 'static,
) -> Result<PathBuf, String>
```

- Use `reqwest::Response::bytes_stream()` and a `tokio::io::BufWriter`
  wrapping `tokio::fs::File::create`.
- After each chunk, call `progress(bytes_so_far, content_length)`.
- After the stream ends, run `verify_checksum` (existing helper at
  `download.rs:10`) on the on-disk file.
- Transfer the TempDir into `slot` exactly as `download_archive_into`
  does today.

**Verification:**

```bash
cd src-tauri && cargo build
```

Expected: compiles cleanly. (No new tests yet — covered in Step 4.)

### Step 2 — Plumb a progress callback through `Pipeline`

In `src-tauri/src/pipeline.rs`, add a new optional parameter:

```rust
pub async fn run<Cp>(
    label: &str,
    url: &str,
    checksum: Option<&str>,
    copy: Cp,
    progress: ProgressCallback,           // existing: phase events
    download_progress: impl Fn(u64, Option<u64>) + Send + 'static,  // new: byte progress
    session: &mut InstallSession,
) -> Result<u32, String>
```

Or, to avoid breaking the existing signature, create a
`Pipeline::run_with_download_progress` and refactor `run` to call it
with a no-op closure. Same end result, less churn at the call sites.

**Verification:**

```bash
cd src-tauri && cargo build
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Expected: all 55 existing Rust tests still pass (no behavior change
visible to them).

### Step 3 — Wire download progress events from the install flow

In `src-tauri/src/install.rs` (`install_minui`, `try_install_extras`,
and the package install in `package.rs`), build a `download_progress`
closure that emits an `InstallProgressEvent` with the new fields:

```rust
let download_progress = {
    let progress = progress.clone();
    move |bytes: u64, total: Option<u64>| {
        progress(InstallProgressEvent {
            step: "download".to_string(),
            details: match total {
                Some(t) if t > 0 => format!("Downloading ({} / {})", bytes, t),
                _ => format!("Downloading ({} bytes)", bytes),
            },
        });
        // Note: new fields are added in Step 5.
    }
};
```

**Verification:**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Expected: still all green.

### Step 4 — Add unit tests for streaming

In `src-tauri/src/download.rs` `#[cfg(test)] mod tests`, add:

1. `test_download_archive_streaming_writes_to_disk` — point at a
   1 MB local file served by an in-process `tokio` listener; assert
   the file on disk matches the source byte-for-byte.
2. `test_download_archive_streaming_emits_progress` — assert the
   progress closure is called at least once, with monotonically
   increasing byte counts.
3. `test_download_archive_streaming_checksum_failure` — pass a wrong
   checksum; assert the function returns an `Err` and does NOT
   transfer the TempDir into the slot.
4. `test_download_archive_streaming_checksum_success` — pass the
   right checksum; assert `Ok` and that the slot owns the TempDir.

**Verification:**

```bash
cd src-tauri && cargo test --lib download 2>&1 | tail -20
```

Expected: 7 tests in `download` (3 existing + 4 new) all pass.

### Step 5 — Extend `InstallProgressEvent` and the frontend

In `src/types/install.ts`, make the new fields optional:

```ts
export interface InstallProgressEvent {
  step: string;
  details: string;
  currentBytes?: number;
  totalBytes?: number | null;
}
```

In `src/InstallProgress.tsx`, when `step === "download"` and
`totalBytes` is set, render a `<progress>` element. The DESIGN.md
"Install Progress" section already allows a progress bar; this is the
implementation.

**Verification:**

```bash
bun run typecheck
bun run lint
bun test 2>&1 | tail -20
```

Expected: typecheck clean, lint clean, all existing frontend tests
pass (the new fields are optional so the existing tests don't break).

## Done criteria (machine-checkable)

- `cd src-tauri && cargo test --lib` shows all 55+ existing tests pass
  **and** 4 new `download` tests pass.
- `cd src-tauri && cargo build --release` succeeds.
- `bun run typecheck` and `bun run lint` pass.
- `bun test` (frontend) shows zero regressions.
- Manual check: start a MinUI install, watch the log panel — the
  download step now shows byte progress, and the temp file size on
  disk grows during the download (not after).

## Test plan

- Unit tests for the streaming primitive (Step 4).
- Integration test (out of scope to add here, but documented for the
  next pass): use the `wiremock` or `httpmock` crate to assert
  progress events fire on partial reads, not just on full reads.
- Frontend: extend `InstallProgress` test (currently no test file —
  see Plan 004 to add one) to assert the progress bar renders when
  `totalBytes` is set.

## Maintenance note

Any future Tauri command that downloads a file (e.g. a future "download
custom firmware" feature) should call `download_archive_streaming` with
a closure that emits to its own progress event, NOT call `reqwest` directly.
If you find yourself adding `reqwest::get(...)` anywhere in
`src-tauri/src/`, this is the wrong place — extend `download.rs` instead.

## Escape hatches

- **If `bytes_stream()` turns out to be unstable on the pinned `reqwest`
  0.12 version** (it isn't, but check): fall back to
  `tokio::io::copy(&mut response.bytes().await?.as_ref(), &mut file)`
  which still streams via chunked reads, just loses the mid-stream
  progress visibility. The user-facing change is "no progress bar"
  but memory is still bounded.
- **If the frontend's `listen` doesn't pick up the new optional
  fields**: the fields are added with `?` so the contract stays
  backward-compatible. Stop and report back — do not "fix" by making
  them required.

## Reference

- Real `diskutil info` output: see conversation history (captured for
  the SD card detection fix; not strictly needed for this plan but
  included so an executor can verify the existing test fixtures).
- `.planning/codebase/CONCERNS.md` → "Performance Bottlenecks" lists
  this exact issue.
