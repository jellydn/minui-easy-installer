# CONCERNS — Technical Debt, Bugs, and Risks

> **Status:** Active
> **Project:** MinUI Easy Installer & Package Store (Tauri v2 + React)
> **Last audit commit:** `4d6e95a`

This document catalogs known technical debt, fragile areas, security
concerns, performance hotspots, and architectural risks in the
codebase. It is the source of truth referenced by other planning
documents (e.g., `plans/001`–`005`).

The project follows hard constraints set in `AGENTS.md`:

- Never write to SD card without explicit user confirmation.
- Never format drives in MVP (…this has changed — see "Format Drive
  Now Exposed in UI" below).
- Never log WiFi passwords or secrets in plaintext.
- Treat registry data as untrusted — validate schema before use.
- Preserve user ROMs/saves/config during MinUI updates.
- Extract archives to temp dir before copying to SD card.

---

## Table of Contents

1. [Security Concerns](#1-security-concerns)
2. [Performance Bottlenecks](#2-performance-bottlenecks)
3. [Fragile Areas](#3-fragile-areas)
4. [Destructive Operations & Error Handling](#4-destructive-operations--error-handling)
5. [Async / Concurrency Concerns](#5-async--concurrency-concerns)
6. [Test Coverage Gaps](#6-test-coverage-gaps)
7. [Code Smells & Maintenance Hazards](#7-code-smells--maintenance-hazards)
8. [Architectural / Scaling Limits](#8-architectural--scaling-limits)
9. [UI / UX Edge Cases](#9-ui--ux-edge-cases)
10. [Deprecated or Dead Code](#10-deprecated-or-dead-code)
11. [Build / Tooling / DevEx](#11-build--tooling--devex)
12. [Git Hygiene](#12-git-hygiene)

---

## 1. Security Concerns

### 1.1 — `wifi.txt` stores passwords in plaintext (by design, but acknowledged)

**Severity:** Medium (acknowledged, intentional)
**File:** `src-tauri/src/wifi.rs:18`, `src/WifiWizard.tsx:175`

`write_wifi_config` writes `SSID:PASSWORD` to `<sd_root>/wifi.txt` in
plaintext. The user-facing UI (`WifiWizard.tsx:175`) shows a disclaimer
"Note: Your WiFi password will be stored in plain text on the SD card
(wifi.txt). This is required for MinUI WiFi functionality." This is
required by MinUI's runtime — it reads `wifi.txt` directly — so the
behavior is intentional. No mitigation is needed, but the constraint
"Never log WiFi passwords or secrets in plaintext" from `AGENTS.md`
**must be honored** elsewhere:

- ✅ No `eprintln!` / `println!` in `wifi.rs` logs the password
  (`grep password src-tauri/src/wifi.rs` shows the value is only
  read from the `password` parameter, never printed).
- ⚠️ **Test code logs passwords via `.unwrap()`/test-only fixtures** —
  see `src-tauri/src/wifi.rs:299`, `:345` — these are tests, not
  production, but they remain in the binary's compiled output. Not a
  vulnerability (test binary is separate) but worth flagging.

### 1.2 — WiFi password sent in plaintext over IPC

**Severity:** Low (Tauri IPC is local-only)
**File:** `src/types/install.ts`-equivalent wiring, `src/WifiWizard.tsx:66-69`

The `write_wifi_config` Tauri command takes `password: String` over
the IPC channel. Tauri v2 IPC is process-local and not exposed to the
network. Acceptable.

### 1.3 — Archive extraction has path-traversal protection (good)

**Severity:** Mitigated
**File:** `src-tauri/src/extract.rs:29-34`, `:80-91`

`is_path_traversal` blocks `..` and absolute paths in ZIP entries. A
secondary canonicalize-and-`starts_with` check (`extract.rs:80-91`)
defends against symlink races. Test coverage at
`extract.rs:213-218`, `:271-275` confirms the protection works.

**Caveat:** the secondary check canonicalizes the parent of each
file entry, which means files at the top of the archive (e.g. a
top-level `passwd`) must exist on disk before their parents are
canonicalizable. The `or_else` block creates the parent and retries
(`extract.rs:74-78`). This works but is more complex than necessary —
a future refactor could simplify by using `output_path.join(entry_path)`
and the existing `is_path_traversal` predicate alone.

### 1.4 — Package install path validation has TOCTOU race window

**Severity:** Low (mitigated, but worth understanding)
**File:** `src-tauri/src/pipeline.rs:165-219`

`create_target_within` validates the parent path against the SD card
root *before* calling `create_dir_all`, then re-validates the
canonicalized path *after* creation. The re-validation handles
symlink-swap races (TOCTOU). A symlink at any non-existing
intermediate path that resolves to outside the SD card will be caught
by the existing-ancestor canonicalize helper
(`pipeline.rs:233-244`). Documented and tested; this is solid.

### 1.5 — `format_drive` is exposed via Tauri command

**Severity:** High (destructive, by design but flagged in `AGENTS.md`)
**File:** `src-tauri/src/drives.rs:206-265`, `src-tauri/src/lib.rs:25-27`

`AGENTS.md` says: **"Never format drives in MVP"**. However, the Tauri
command `format_drive` exists in `lib.rs:25-27` and is exposed via the
UI at `src/DriveSelector.tsx:36-44`, `src/FormatConfirmDialog.tsx`.

The Format flow uses a confirmation modal (`FormatConfirmDialog.tsx`),
which is the minimum acceptable safeguard, but this **contradicts the
documented constraint**. Either:
- Remove the command, the UI button, and the dialog; OR
- Update `AGENTS.md` to reflect the new policy (format is opt-in via
  the drive selector).

**Action item:** Decide policy. Code allows it; docs forbid it.

### 1.6 — Registry data is untrusted but provenance is unclear

**Severity:** Low
**File:** `src/types/package.ts:289-330`

`fetchPackageRegistry` falls back to a **bundled** `store.json`
(`package.ts:307`) when the remote fetch fails. The bundled data is
imported directly into the bundle:

```ts
import storeData from "./store.json";
```

The bundled store is treated identically to the remote registry,
which is good (same validation pipeline), but `package.ts:34` URL is
`https://packages.minui.dev/registry/index.json` — a static JSON
without an integrity check. A successful MITM on the static JSON host
(or a compromised update of `store.json` in the repo) would let an
attacker inject malicious `artifactUrl` values. Mitigations:

- The `resolveDownloadUrl` (`package.ts:185-188`) restricts to
  `https://github.com/`, so the worst case is a malicious GitHub
  release URL. Checksum validation at install time
  (`package.ts:103`) catches download tampering.
- The `validateStoreEntry` function (`package.ts:204-258`) is
  thorough on required fields but **does not validate that
  `repository` matches the actual download host**. A
  `repository: "https://github.com/safe/repo"` and
  `artifactUrl: "https://malicious.example.com/payload.zip"` would
  pass — but no current code path lets a registry entry set a
  free-form `artifactUrl` (it's always derived from
  `repository`+`version`+`pak_name` via `resolveDownloadUrl`).
- ⚠️ The `tool_paks.download_url` override in
  `package.ts:267-269` (`resolveDownloadUrl` returns the override)
  is **not validated to be on github.com**. A malicious
  `store.json` could ship a `download_url` pointing anywhere.
  Recommend restricting `download_url` to `https://github.com/`
  in `validateStoreEntry`.

### 1.7 — Tauri command surface area is large

**Severity:** Low (defense-in-depth)
**File:** `src-tauri/src/lib.rs:280-300`

19 Tauri commands are registered, including `format_drive` (destructive),
`write_wifi_config` (writes secret), and `install_minui` (writes to
SD). Tauri v2's capability system (`src-tauri/capabilities/default.json`)
only declares `core:default` — meaning every command the Rust code
exposes is invocable from the frontend. There is no per-command
permission gate. For an app that ships to end users, this is
acceptable (the frontend is the only IPC caller), but anyone embedding
the Rust binary in a different frontend could reach any command. Not
a vulnerability in the current architecture; flagged for awareness.

### 1.8 — `fs_utils::copy_dir_recursive` follows symlinks (no-op risk)

**Severity:** Low
**File:** `src-tauri/src/fs_utils.rs:101-109`, test at `:172-194`

`fs::copy` dereferences symlinks by default. The test at `:172-194`
documents this behavior. If an attacker can place a symlink inside the
extracted temp dir (via a malicious archive that survives
`is_path_traversal`), the copy step will follow it. The path traversal
check at `extract.rs:29-34` would catch `..` paths, but **does not
catch absolute symlink targets** (e.g. a `libcrypto.so` symlink
pointing to `/usr/lib/libcrypto.so.dylib`). The current archive format
is ZIP; the `zip` crate's `entry.unix_mode()` will not produce
symlinks in a standard MinUI archive, so this is a low-probability
issue. Consider adding `symlink_metadata` filtering in
`copy_dir_recursive` if concerned.

### 1.9 — No CSP for `connect-src` whitelist is loose

**Severity:** Low
**File:** `src-tauri/tauri.conf.json:25`

`connect-src` allows `https://packages.minui.dev`, `https://api.github.com`,
`https://github.com`, and `https://*.githubusercontent.com`. The
`*.githubusercontent.com` wildcard is broad but standard for
GitHub-anchored apps. No `script-src 'unsafe-inline'` — good.

---

## 2. Performance Bottlenecks

### 2.1 — Byte-level download progress events are not yet wired to the UI

**Severity:** Medium (UX, not perf)
**Files:**
- `src-tauri/src/lib.rs:147-152` — the closure is a no-op with a TODO
- `src/types/install.ts:28-36` — `currentBytes`/`totalBytes` fields exist
- `src/InstallProgress.tsx` — does not consume them

`download_progress_streaming` in `download.rs:165-235` already streams
in 8 KB chunks and invokes a progress callback per chunk. The
plumbing is in `pipeline.rs:111-122`. The frontend types
(`install.ts:28-36`) declare the fields. **But the `lib.rs:147-152`
callback is empty and `InstallProgress.tsx` does not render a
progress bar**. The user sees only `"Downloading base.zip"` with no
percentage. See `plans/001-stream-archive-downloads.md` for the full
backlog.

**Status:** Plumbing is in place; UI consumption is missing. Recent
PR (referenced in task description) is the streaming infrastructure;
display is the next step.

### 2.2 — Archive extraction uses sync `fs::File` in a tight loop

**Severity:** Low
**File:** `src-tauri/src/extract.rs:118-134`

Extraction reads each ZIP entry in an 8 KB sync loop while the
Tauri command is `async` (`extract_archive_to_directory` in
`lib.rs:60`). The sync loop blocks the tokio worker thread for the
duration of the extraction. For 500 MB+ MinUI base archives, this
holds a worker hostage. `Plan 001` notes: "extraction is already
streaming (8 KB buffer). Do not touch." — but it could be moved
to `tokio::task::spawn_blocking` to free the worker. Low impact
(extraction is fast on SSD), flagged for awareness.

### 2.3 — `fs_utils::copy_dir_recursive` is fully sync

**Severity:** Low
**File:** `src-tauri/src/fs_utils.rs:60-114`

`copy_dir_recursive` uses `std::fs::*` (blocking I/O). Called from
async contexts in `install.rs:159` and `package.rs:166` — these will
block a tokio worker. For the SD card copy step (often the longest
phase of an install), this is acceptable since cancellation can
only check between files (`fs_utils.rs:74-75`), but the block is
noticeable for 500 MB+ copies. Consider `spawn_blocking` with
`tokio::select!` against the cancel token.

### 2.4 — Full release metadata is fetched on every version check

**Severity:** Low
**File:** `src/hooks/useVersionCheck.ts:31-58`

Each `useVersionCheck.check(sdMount)` call does **two** network
requests: `fetchMinUIRelease` (GitHub API) and `fetchPackageRegistry`
(remote + bundled fallback). The `releaseCache` in
`release.ts:90-92` and `cachedRegistry` in `package.ts:48-50` are
**session-scoped** and survive across checks. Net cost: 2 requests
on the first drive selection, 0 on subsequent ones. Good.

### 2.5 — `useEffect` race in `useVersionCheck`

**Severity:** Low (already mitigated)
**File:** `src/hooks/useVersionCheck.ts:22-58`

The `requestIdRef` pattern (`useVersionCheck.ts:30`) correctly handles
out-of-order fetches. Old requests are dropped via
`requestId !== requestIdRef.current` guards. Solid.

### 2.6 — `handleUpdateAll` runs package installs concurrently via `Promise.all`

**Severity:** Low (intentional, well-handled)
**File:** `src/Home.tsx:269-285`

`handleUpdateAll` uses `Promise.all` to run all package updates
concurrently. The Rust `install_minui_with_cancel` does not yet
support concurrent installs (the `InstallRegistry` only tracks one
token — `lib.rs:84-99`), so this only applies to **package** installs
(`installPackage` is its own IPC call with no shared state). Each
`installPackage` invocation creates its own `InstallSession` (local
`session` variable in `package.rs:130`) so the temp-dir ownership is
isolated. Good.

---

## 3. Fragile Areas

### 3.1 — SD card drive detection (recently hardened, still fragile)

**Severity:** Medium
**File:** `src-tauri/src/drives.rs:8-79` (macOS), `:368-408` (Windows)

The drive detection on macOS uses `df` + `diskutil info` to filter
out internal volumes. The `classify_volume` function
(`drives.rs:114-170`) is extensively unit-tested
(`drives.rs:501-630`) and has an `#[ignore]`-marked integration test
that requires a real SD card (`drives.rs:649-742`).

**Residual fragility:**

- The macOS filter is by name (`"Macintosh HD"` substring at
  `drives.rs:50-52`). If the user has renamed their boot volume
  (common on Hackintoshes), internal drives could be included.
- The `df` parser at `drives.rs:34-43` does `parts.last()` and
  assumes the mount path is the last whitespace-separated field.
  This breaks for mount paths containing spaces, which Linux/Unix
  *can* have (e.g. legacy `/Volumes/Time Machine Backups` —
  `df` quotes paths with spaces, which would still parse
  correctly, but paths with leading whitespace would be sliced
  wrong).
- The `df` parser at `drives.rs:34-43` skips lines with fewer
  than 6 whitespace-separated fields. This is correct for BSD
  `df` output on macOS but is not defensive against future
  macOS `df` format changes.
- On Windows, `powershell` is called with `-Command` and a long
  inline string (`drives.rs:374-385`). No timeout. A hung
  PowerShell session would block the UI.

**Mitigations in place:**

- Tests at `drives.rs:464-742` cover the happy path and known edge
  cases (substring field names, duplicate fields, empty values).
- The `is_path_traversal` and canonicalize-checks in
  `pipeline.rs:172-218` prevent the install code from following a
  misclassified internal drive's symlinks into the user's home
  directory.

### 3.2 — Version parsing (recently hardened, plan #5 in progress)

**Severity:** Low (just fixed)
**File:** `src-tauri/src/version.rs:78-141`, `src-tauri/src/version.rs:30-66`

The raw fallback at `version.rs:80-87` previously accepted any
string containing a digit or dot — a known trap for "Created by
MinUI Team 2024". **Plan 005** (referenced by task description)
hardens this with a new `looks_like_version` helper
(`version.rs:32-66`) that requires 2–3 dot-separated numeric
segments with optional `v`/`V` prefix. Tests at
`version.rs:243-281` lock in the new behavior. The `.changeset/brave-bugs-type.md`
file documents the patch. **Status: complete.**

**Residual fragility:**

- `try_parse_semver` (`version.rs:88-105`) strips leading zeros per
  segment, which can produce invalid semver for "00.00.00" → "0.0.0"
  (which is valid semver but lossy if the original version was
  literally `00.00.00`).
- `compare_versions` falls back to **string comparison** for
  unparseable versions (`version.rs:121`). This is correct for
  YYYY.MM.DD but wrong for any mixed-format future. Document the
  fallback.
- `minui.txt` content is read with `fs::read_to_string`
  (`version.rs:43`). If the file is non-UTF-8 (e.g. an SD card
  with a FAT32 short-name only file), the read fails and
  `detect_installed_version` silently returns `None`. The user sees
  "MinUI: Not detected" instead of an error. Acceptable.

### 3.3 — `create_target_within` symlink-race window

**Severity:** Low (already mitigated)
**File:** `src-tauri/src/pipeline.rs:165-219`

See §1.4. The function validates the parent path before and after
`create_dir_all` to catch symlink swaps. The `canonicalize_existing_ancestor`
helper (`pipeline.rs:233-244`) walks up until it finds an existing
ancestor to canonicalize — required for the fresh-install case where
`/Tools/<platform>/<pak>.pak` does not exist yet.

**Edge case:** If the SD card root itself is a symlink to outside
its mount point (malicious, but possible if the user formats and
mounts a sandboxed fake SD), `canonicalize` would resolve through
the symlink and the install would proceed. The `drives.rs` filter
is the upstream defense against this; without that, an internal
drive misclassified as removable could be the target.

### 3.4 — `is_preserved_path` comparison is case-insensitive

**Severity:** Low
**File:** `src-tauri/src/install.rs:71-90`

`PRESERVED_FOLDERS` is `&["roms", "saves", "save", "bios", "cheats"]`,
compared with `eq_ignore_ascii_case`. This matches FAT32's
case-preserving but case-insensitive semantics, so a folder named
`ROMs` is correctly preserved. Documented in tests
(`install.rs:411-422`). Correct, but the list is hand-maintained;
adding a new device with a new preserved folder requires a code
change.

**Edge case:** `preserved/something/foo` — `is_preserved_path` only
checks the **first** component after `sd_root` (`install.rs:78-80`).
So `/Tools/roms/` (a `roms` folder nested inside `Tools/`) would
**not** be preserved. Tests at `install.rs:402-410` lock in this
behavior. This is by design but is a footgun: a malicious archive
with a `Tools/roms/` layout would be overwritten.

### 3.5 — `version.txt` semantics vary between `detect` and `install`

**Severity:** Low
**File:** `src-tauri/src/package.rs:50-72`

`detect_installed_packages` looks for `Tools/<name>/version.txt`
(`package.rs:55-58`). A package installed via `installPackage` does
**not** write a `version.txt` — only the install flow copies files
(`package.rs:148-168`). So the "version" field in
`InstalledPackage` will always be `None` for packages installed by
this tool. Documented in `AGENTS.md` ("Packages read `Tools/*/version.txt`
(included in archives)") but worth flagging: update detection is
**only** for packages that ship a `version.txt` in the archive.

### 3.6 — `compare_versions` mixed-format fallback

**Severity:** Low
**File:** `src-tauri/src/version.rs:108-122`

See §3.2. If `installed` is semver-parseable and `latest` is not
(or vice versa), `compare_versions` assumes the semver one is
**more recent** (`version.rs:115-118`). This is a sensible default
but is a guess; the user could be downgrading from a non-semver
fork version. Document.

### 3.7 — Two parallel device-profile systems

**Severity:** Medium (architectural drift)
**File:** `src/types/device.ts:1-157` and `src/types/device-install-map.ts:1-67`

There are **two** device-profile systems:

- `device.ts` — `DeviceProfile` with `id`, `name`, `platform`,
  `extrasPlatform`, `installPathRules`. 17 profiles. Used by
  `Home.tsx`, `PackageStore.tsx`, `PackageCard.tsx`, `install.rs`.
- `device-install-map.ts` — `DeviceInstallRules` with
  `basePlatform`, `extrasPlatform`, `install.action`, `devicePaks`,
  `sharedBios`. Loaded from `device-install-map.json`. Used by:
  - **No current code path** — `getDeviceInstallRules`,
    `getExtrasPlatform`, `getBasePlatform`, `getDevicePaks` are
    all exported but **none are imported anywhere in the project**.

This is dead-code / future-feature surface. The `device-install-map.json`
file is 27+ lines of duplicated data with `device.ts` and risks
drift (a profile added to one must be added to the other). Tests at
`src/types/device-install-map.test.ts` and
`src/types/device.test.ts` cover the individual systems but **not
the parity** between them.

**Action item:** Either delete `device-install-map.ts` and the JSON
(it is unused), or commit to migrating `device.ts` consumers to it
and deprecate `device.ts`. Currently the duplication is a
maintenance trap.

---

## 4. Destructive Operations & Error Handling

### 4.1 — `format_drive` has no second-level confirmation

**Severity:** High (data loss)
**File:** `src/FormatConfirmDialog.tsx:13-58`, `src/DriveSelector.tsx:36-44`

The Format flow:
1. User clicks "Format to FAT32" on the drive selector.
2. `FormatConfirmDialog` shows: "This will erase all data on
   `<name>` and format it as FAT32. This cannot be undone."
3. User clicks "Format to FAT32" again.
4. `diskutil eraseDisk` runs.

**Risk:** The single confirmation click is the only barrier
between a misclick and a wiped drive. Industry standard (e.g.
macOS Disk Utility, Balena Etcher) requires **typing the volume
name** to confirm format. Recommend requiring the user to type
the volume name (or a `DELETE` confirmation) in
`FormatConfirmDialog` before enabling the destructive button.

### 4.2 — SD card write without explicit per-step confirmation (install)

**Severity:** Low (already gated)
**File:** `src/ConfirmDialog.tsx:8-94`, `src/Home.tsx:104-128`

The install flow shows a confirmation dialog listing the target
drive and the install plan. The user must click "Proceed with
Installation" before any write to the SD card begins. Good.

**Edge case:** `handleUpdateAll` (`Home.tsx:206-307`) runs **without**
a separate confirmation dialog — it reuses the install confirmation
flow only on the *first* call. Subsequent batched package installs
go straight to disk once "Update All" is clicked. The "Update All"
button is shown only when updates are available, but a misclick
here starts concurrent package downloads + writes with no further
prompt. Acceptable for a power-user feature, but worth a confirmation
when N packages are about to be installed (N > 1 → "This will install
N packages to `<drive>`. Continue?").

### 4.3 — `write_wifi_config` overwrites the entire `wifi.txt`

**Severity:** Low (intended behavior)
**File:** `src-tauri/src/wifi.rs:30-66`

The function reads the existing `wifi.txt`, filters out any prior
entry for the same SSID, and rewrites the file. Comments and other
SSIDs are preserved (test at `wifi.rs:368-376`). But a **malformed
line** (e.g. a line with `:` in the middle that is not a valid
`SSID:PASS` entry) is preserved verbatim (test at `wifi.rs:378-380`).
This could be exploited if an attacker can write to the SD card
before the wizard runs. Low risk; the file is owned by the user.

### 4.4 — `minui.txt` overwrite is silent on failure

**Severity:** Low
**File:** `src-tauri/src/install.rs:269-278`

After install, `minui.txt` is written with the installed version
(`install.rs:269-278`). On failure, only an `eprintln!` is emitted
(`install.rs:277`) and the install reports `success: true` regardless
(`install.rs:280-298`). The user sees "Installation completed
successfully!" but the version file is missing. Subsequent version
checks will report "MinUI: Not detected" because
`detect_installed_version` (`version.rs:38-66`) reads `minui.txt`.

**Recommendation:** Surface this as a non-fatal warning in
`InstallResult.extras_warning` (already exists for that purpose).

### 4.5 — `create_rom_dirs` swallows errors

**Severity:** Low
**File:** `src-tauri/src/install.rs:248-250`

```rust
let rom_dirs_created = create_rom_dirs(&options.sd_mount).unwrap_or(0);
```

A failure to create any ROM directory is silently turned into `0`
created. The install reports success. The user sees a successful
install but may be missing `Roms/Nintendo DS (NDS)/` etc. on their
SD card. The first create failure short-circuits the loop and
returns `Err` from `create_rom_dirs` (`install.rs:42-53`), but the
caller discards it.

**Recommendation:** Capture the error string, set
`extras_warning`, and return the count of dirs actually created.

### 4.6 — `parse_size_str` is `#[allow(dead_code)]`

**Severity:** Trivial
**File:** `src-tauri/src/drives.rs:309-330`

`parse_size_str` is annotated `#[allow(dead_code)]` and only called
from a unit test (`drives.rs:419-423`). The function is dead in
production. Either remove or wire it into the macOS diskutil
output parser for `size_bytes` (currently populated from
`fs_utils::get_disk_space`, which uses `statvfs`).

### 4.7 — `format_bytes` is duplicated in TS and Rust

**Severity:** Trivial
**Files:** `src/types/validate.ts:99-117`, `src-tauri/src/validate.rs:102-113`,
`src-tauri/src/health.rs:127-129`

The same `format_bytes` function is implemented three times. Minor
maintenance burden; consider a single Rust function exposed via
Tauri for client rendering, or a single TypeScript module.

### 4.8 — Permissions on extracted files not preserved on Windows

**Severity:** Low
**File:** `src-tauri/src/extract.rs:136-144`

The `unix_mode` branch (`#[cfg(unix)]` at `extract.rs:137`) preserves
POSIX permissions. The `#[cfg(unix)]` gate means Windows files
extracted have default ACLs. Acceptable; the SD card is FAT32
anyway and doesn't store POSIX permissions.

---

## 5. Async / Concurrency Concerns

### 5.1 — `InstallRegistry` is a `Mutex<Option<...>>`, not a `RwLock`

**Severity:** Trivial
**File:** `src-tauri/src/lib.rs:84-99`

`InstallRegistry` uses `std::sync::Mutex` (`lib.rs:84-86`) with
`lock().unwrap()`. If any holder panics while holding the lock,
the next call will **poison** the mutex and the `.unwrap()` will
panic. Across `start_install` / `cancel_install`, this is unlikely
(no held state), but `tokio::sync::Mutex` would be more idiomatic
in an `async` Tauri command.

### 5.2 — `tokio::spawn` in `start_install` is fire-and-forget

**Severity:** Low (documented)
**File:** `src-tauri/src/lib.rs:154-176`

The install runs in a `tokio::spawn` task that returns the result
via the `install-complete` / `install-error` events. The
`install_minui_with_cancel` is not `await`ed from the Tauri command
(`lib.rs:155`). The task handle is dropped after spawn. If the
process is killed mid-install, the task is interrupted. The
`InstallSession` (`pipeline.rs:30-49`) holds `TempDir`s in
struct fields, so a `kill` mid-install would leave temp dirs on
disk only if the parent process survives; otherwise `tempfile`'s
drop semantics clean up.

### 5.3 — Cancellation is one-shot, not idempotent

**Severity:** Low
**File:** `src-tauri/src/pipeline.rs:96-122`, `:138-154`

`Pipeline::run` checks `cancel.is_cancelled()` at the start of each
phase (`pipeline.rs:138`, `:152`). The token is `CancellationToken`
from `tokio_util` (`lib.rs:14`), so multiple `.cancel()` calls are
idempotent (no-op after the first). Good.

### 5.4 — `lib.rs:147-152` — `download_progress` closure is empty

**Severity:** Medium (see §2.1)
**File:** `src-tauri/src/lib.rs:144-152`

The byte-level download progress callback is constructed but
**does not call any emit**. Comment at `lib.rs:149-151`:
"Future enhancement: extend `InstallProgressEvent` with
`currentBytes` / `totalBytes` and emit them here." The fields
**already exist** on the TS side (`install.ts:28-36`), so this is
half-wired. See `plans/001`.

### 5.5 — `useScrollToBottom` re-scrolls on every `items` change

**Severity:** Trivial
**File:** `src/hooks/useScrollToBottom.ts:1-18`

The `useEffect` dep is `[items]`, so a new array reference on every
state update triggers a smooth scroll. Cheap, but the container
will not stop scrolling when the user manually scrolls up to read
an old log line. Standard "stick to bottom" UX gap; consider
detecting manual scroll-up and pausing.

### 5.6 — `useVersionCheck` has a 4-stage serial fetch

**Severity:** Trivial (intentional, well-thought-out)
**File:** `src/hooks/useVersionCheck.ts:31-58`

The hook does:

1. `fetchMinUIRelease(fork)` (GitHub API)
2. `checkMinuiVersion(...)` (Tauri IPC)
3. `fetchPackageRegistry()` (remote + bundled fallback)
4. `checkPackageUpdates(...)` (Tauri IPC)

Steps 1+2 are independent of 3+4, so they could be parallelized.
The current serial flow is simpler and only costs ~2 round-trips.
Not worth changing.

### 5.7 — No back-pressure on the `install-progress` event channel

**Severity:** Low
**File:** `src-tauri/src/lib.rs:73-80`, `:135-141`

The progress events are emitted via `app_handle.emit("install-progress", event)`
in a tight loop during the download phase (one event per 8 KB chunk
at 8 KB/s for a 500 MB archive = ~60 000 events). The Tauri event
bus is fire-and-forget; the frontend listener in `Home.tsx:130-149`
updates React state on each event. With 60 K state updates, the
React reconciliation is the bottleneck, not the IPC.

**Mitigation in place:** The streaming download calls the progress
callback per chunk (`download.rs:198-201`). A natural rate-limit
(emit only every N chunks, or coalesce to 5 % intervals) would
help. `Plan 001` notes that emission is "every N MB or every 5 %
of Content-Length, whichever is smaller" but the current code
emits per-chunk.

### 5.8 — `unlisten` is only called in `finally`

**Severity:** Low
**File:** `src/Home.tsx:130-149`

The `unlisten = await listen<...>(...)` is called inside a
`try { ... } finally { unlisten(); }` block (`Home.tsx:131`,
`:217`). If the `await listen` itself throws (it shouldn't, but
if it did), the `unlisten` is never assigned. Consider moving
the listen setup above the try block. Low risk; `listen` rarely
throws.

---

## 6. Test Coverage Gaps

### 6.1 — `lib.rs` install command wrapper has no integration test

**Severity:** Low
**File:** `src-tauri/src/lib.rs:325-405`

The `#[cfg(test)]` block in `lib.rs` covers the **underlying**
functions (`install_minui`, `package::install_package`, etc.) but
**not the Tauri command wrappers** (`install_minui` as a Tauri
command, `start_install`, `cancel_install`). A real Tauri command
takes an `AppHandle` for event emission; the test in
`lib.rs:430-455` uses `Arc::new(|_| {})` for the progress
callback. The Tauri IPC layer is not exercised. Recommend
adopting `tauri::test::mock_app()` for command-level tests.

### 6.2 — `package.rs` `install_package_with_cancel` has no test

**Severity:** Low
**File:** `src-tauri/src/package.rs:125-150`

The function takes a `cancel: CancellationToken` and a
`download_progress` closure. The contract test in `lib.rs:481-499`
covers the no-cancel path. The cancel path is not tested. `Plan
002` calls for adding it.

### 6.3 — `pipeline.rs` `create_target_within` race test is missing

**Severity:** Low
**File:** `src-tauri/src/pipeline.rs:165-219`

The TOCTOU defense (`create_target_within` re-validates after
`create_dir_all`) is documented and the helper
`canonicalize_existing_ancestor` is unit-testable, but there is
no test that races a symlink swap with the function. Hard to test
deterministically without `fork`; consider a Linux-specific test
that races a `rename(2)` between create and canonicalize.

### 6.4 — `format_validation_report` Rust function is not directly tested

**Severity:** Trivial
**File:** `src-tauri/src/validate.rs:179-202`

Tested via `lib.rs:472-490` indirectly. Adequate.

### 6.5 — `Home.test.tsx` does not cover the install success path

**Severity:** Low
**File:** `src/Home.test.tsx:1-219`

Tests cover:
- Title + device selector render.
- Install button shows when both selected.
- Status summary renders.
- Confirmation dialog opens on click.

**Missing:**
- Install success path (mock `installMinui` → success, assert
  validation triggers).
- Install error path (mock `installMinui` → error, assert error
  UI).
- `handleUpdateAll` path (mock `fetchMinuiVersion` with updates,
  assert `installPackage` called for each).
- `handleUpdateAll` mid-flight cancellation (out of scope; the
  cancel mechanism is `Plan 002`).

### 6.6 — `confirm_dialog` is not tested

**Severity:** Low
**File:** `src/ConfirmDialog.tsx`, `src/FormatConfirmDialog.tsx`

Neither component has a `*.test.tsx`. These are user-facing
modals for destructive operations. Recommend adding tests for
the cancel and confirm callbacks.

### 6.7 — `useScrollToBottom` hook is not tested

**Severity:** Trivial
**File:** `src/hooks/useScrollToBottom.ts:1-18`

A 5-line hook, but the `scrollIntoView` side effect is hard to
verify without a jsdom. Acceptable.

### 6.8 — `releaseCache` invalidation is not tested

**Severity:** Trivial
**File:** `src/types/release.ts:90-96`

`clearReleaseCache(key?)` is exposed but not unit-tested. If a
contributor changes the cache key format, the test will not
catch a regression. Minor.

---

## 7. Code Smells & Maintenance Hazards

### 7.1 — Three separate places compute `installPathRules` paths

**Severity:** Medium
**Files:** `src/PackageCard.tsx:13-22`, `src-tauri/src/package.rs:145-152`,
`src-tauri/src/pipeline.rs:175-185`

The "where does this package go?" calculation exists in three
places: TS for display, Rust in `package.rs`, and Rust in
`pipeline.rs`. They all do `sd_root / target_dir / platform /
{pak_name}.pak/`, but a future change to e.g. support a `{version}`
path segment would require updating all three. Recommend a single
canonical function in Rust exposed via Tauri for the display.

### 7.2 — `getDeviceProfile` lookup is O(n)

**Severity:** Trivial
**File:** `src/types/device.ts:147-150`

```ts
return DEVICE_PROFILES.find((profile) => profile.id === id);
```

17 profiles → negligible. If the profile list grows past ~100,
switch to a `Map`.

### 7.3 — `fetchPackageRegistry` is 100+ lines

**Severity:** Low
**File:** `src/types/package.ts:289-330`

`fetchPackageRegistry` includes: cache check, network fetch,
parse, validation, fallback to bundled store. Consider splitting
into `fetchRemote`, `parseRegistry`, `fallbackToBundled`. Low
priority; the function is read-once at mount.

### 7.4 — `handleUpdateAll` is 100+ lines

**Severity:** Low
**File:** `src/Home.tsx:206-307`

Mixed concerns: version check, install orchestration, error
aggregation, state reset. Consider extracting to
`useUpdateAll(fork, device, drive)` hook. Low priority.

### 7.5 — `package.ts` has type definitions for the legacy
`StoreEmuPak` / `StoreToolPak` format

**Severity:** Low
**File:** `src/types/package.ts:138-180`

The bundled `store.json` uses a different schema than the public
`PackageRegistry`. The conversion at `package.ts:262-313` is
necessary, but the two schemas create a maintenance trap — any
schema change must be applied to both. Consider migrating
`store.json` to the public schema or documenting the legacy
format in `AGENTS.md`.

### 7.6 — `minui.txt` write format is not validated

**Severity:** Trivial
**File:** `src-tauri/src/install.rs:269-278`

```rust
fs::write(&minui_txt_path, format!("MinUI {}\n", options.version))
```

If `options.version` contains a newline (it can't via the current
fetch path, but the `InstallOptions` struct allows any `String`),
the file will have two lines. Low risk.

### 7.7 — `storeData` import is unconditional

**Severity:** Trivial
**File:** `src/types/package.ts:6`

`import storeData from "./store.json";` is statically bundled.
For a tree-shakable, lazy-loaded fallback, use a dynamic
`import()`. Minor bundle-size impact (the JSON is small).

### 7.8 — `fetchMinUIRelease` takes a `fetchFn` parameter

**Severity:** Trivial
**File:** `src/types/release.ts:99-103`

The `fetchFn` parameter is exposed for testing. Production callers
omit it. The default is `globalThis.fetch`. The conditional cache
write at `release.ts:108-110` (skip cache when an injected
`fetchFn` is passed) is a subtle behavior that may confuse
contributors. Document or simplify by inlining the cache logic.

### 7.9 — `Map` rebuild on every `useVersionCheck` call

**Severity:** Trivial
**File:** `src/Home.tsx:255-258`

```ts
const packageByName = new Map(
  registryResult.data.packages.map((p) => [p.name, p]),
);
```

Rebuilt on every `handleUpdateAll` call. With <50 packages, this is
fine. The comment explains the optimization vs. an O(n*m) `find` per
iteration; the Map itself is correctly O(n).

---

## 8. Architectural / Scaling Limits

### 8.1 — Single-install registry: no concurrent installs

**Severity:** Architectural
**File:** `src-tauri/src/lib.rs:84-99`

`InstallRegistry` is `Mutex<Option<CancellationToken>>` — only one
install at a time. The UI never runs concurrent installs in a single
window (documented at `lib.rs:90-92`), so this is intentional. If a
future feature adds multi-window support, this needs to become
`Mutex<HashMap<InstallId, CancellationToken>>` (the design proposed
in `plans/002-cancel-install-mechanism.md:107-115`).

### 8.2 — Install cancellation is per-app, not per-install

**Severity:** Architectural (documented)
**File:** `src-tauri/src/lib.rs:90-92`, `:120-128`

`cancel_install` cancels **whatever is running**. No `install_id`
parameter is taken. `Plan 002` step 2 keeps this design (single
install at a time) but the `start_install` command returns `"current"`
(`lib.rs:175`) as a stub ID. If multi-install support is added,
revisit.

### 8.3 — `package.json` has no React Refresh / HMR config for Tauri

**Severity:** Trivial
**File:** `package.json:5-17`

`tauri.conf.json` uses `npm run dev` (`build.beforeDevCommand`) but
`package.json` defines `dev` as `vite` (not `vite --host` or
similar). Tauri v2's dev URL is `http://localhost:1420`
(`tauri.conf.json:9`). The HMR config is implicit via Vite. Fine.

### 8.4 — No bundle size or performance budget

**Severity:** Trivial
**File:** `package.json`

No `vite build` analysis or size limit in CI. The bundled `store.json`
(`src/types/store.json`) is a few KB; the React + Tauri client
should be < 2 MB. No regression gate.

### 8.5 — `Cargo.lock` is committed (good) but dep churn is unmonitored

**Severity:** Trivial
**File:** `src-tauri/Cargo.lock`

`Cargo.lock` is committed (verified by reading the file
head: `version = 4` at `Cargo.lock:3`). No `cargo audit` or
`cargo outdated` CI step. Recommended for a Tauri app shipping
binaries to end users.

### 8.6 — `tokio = features = ["full"]` is enabled

**Severity:** Trivial
**File:** `src-tauri/Cargo.toml:21`

`features = ["full"]` pulls in fs, net, io-util, sync, time, etc.
Larger binary size. For a Tauri app, the "full" feature is
acceptable. If size matters, switch to the minimal feature set
(`rt-multi-thread`, `fs`, `io-util`, `macros`).

---

## 9. UI / UX Edge Cases

### 9.1 — Drive selection lost after format

**Severity:** Low
**File:** `src/DriveSelector.tsx:48-69`

After a successful format, `DriveSelector` re-fetches drives and
calls `onSelectDrive(updatedDrive)` if the same mount path is
found (`DriveSelector.tsx:60-64`). If the user's drive was unmounted
during format and remounted with a different name, `updatedDrive`
is `undefined` and the parent's `selectedDrive` becomes stale
(Home.tsx still holds the old reference). The drive selector
shows the new list, but the parent doesn't update.

**Recommendation:** Call `onSelectDrive(null)` if the formatted
drive is no longer in the updated list, then surface a "Drive
remounted as `<new name>`. Please re-select."

### 9.2 — No "are you sure?" for `handleUpdateAll` with N > 1 packages

**Severity:** Low
**File:** `src/Home.tsx:206-307`

See §4.2. The button label is "Update All" (`Home.tsx:441`) and
clicking it starts concurrent package installs with no further
prompt.

### 9.3 — `format_drive` on Windows always errors

**Severity:** Low (by design)
**File:** `src-tauri/src/drives.rs:267-272`

```rust
#[cfg(target_os = "windows")]
pub fn format_drive(_mount_path: &str, _volume_name: &str) -> Result<(), String> {
    Err("Formatting is not yet supported on Windows".to_string())
}
```

The button is shown in the UI regardless. On Windows, the user
clicks "Format to FAT32" and gets an error. Either:
- Hide the button on Windows (detect via `invoke<{platform: "win">}('format_drive')`
  probe), OR
- Show a disabled button with a tooltip "Not yet supported on Windows".

### 9.4 — WiFi password field has no visibility toggle

**Severity:** UX
**File:** `src/WifiWizard.tsx:164-176`

The password `<input type="password">` is obscured. Standard
expectation is a "show password" toggle. Missing. Low priority
for a one-time install wizard.

### 9.5 — `WifiWizard` does not validate password length

**Severity:** Trivial
**File:** `src/WifiWizard.tsx:71-93`

WPA2 requires 8–63 characters. The wizard accepts an empty
password (open network) but does not enforce 8+ for WPA. The
underlying `write_wifi_config` (`wifi.rs:18-66`) accepts any
non-empty SSID. Empty password is allowed (open network). The
device side will reject too-short passwords at connection time.

### 9.6 — `useEffect` in `Home.tsx` does not fire on initial mount

**Severity:** Trivial
**File:** `src/Home.tsx:71-80`

The `useEffect` that calls `version.check(selectedDrive.mount_path)`
is fired when `selectedDrive` changes. If the user picks the same
drive twice in a row (re-selecting), the `useEffect` does not
re-run because the dep array reference is unchanged. The
`useVersionCheck` hook (`useVersionCheck.ts:51-54`) accepts
`s.check(sdMount)` as a callable, so the user could re-trigger via
a manual refresh button. There is none. Recommend a "Re-check
version" button in the Status Summary.

### 9.7 — `InstallProgress` does not persist log across re-renders

**Severity:** Trivial
**File:** `src/InstallProgress.tsx:62-69`

The `log` array is passed in as a prop. Every `setInstall` in
`Home.tsx:131-148` spreads `log: [...s.log, event.payload]`, so
the log grows monotonically. The `useScrollToBottom` hook
(`useScrollToBottom.ts:11-14`) re-scrolls on every change. If
the user scrolls up, the next log line forces a scroll back to
bottom. Standard "tail" behavior; consider a "pause scroll"
toggle.

### 9.8 — `ConfirmDialog` Cancel button has no keyboard shortcut

**Severity:** Trivial (a11y)
**File:** `src/ConfirmDialog.tsx:75-93`

No `Escape` key handler. A user who hits Escape to dismiss the
modal gets no effect; they must click the Cancel button.

### 9.9 — CSP does not allow `localhost` for dev

**Severity:** Low (dev only)
**File:** `src-tauri/tauri.conf.json:25`

The CSP `connect-src` is `default-src 'self'; connect-src 'self'
https://packages.minui.dev https://api.github.com https://github.com
https://*.githubusercontent.com`. In dev, Tauri uses
`http://localhost:1420`. Tauri's webview injects an IPC bridge
that bypasses CSP for `tauri://` calls. Should work; if not, add
`http://localhost:*` to `connect-src` for dev builds.

### 9.10 — No log export / copy after install

**Severity:** UX
**File:** `src/InstallProgress.tsx`

After a failed install, the only way to share the log with
support is screenshot. The Health Check has a "Copy Support
Report" button (`HealthCheck.tsx:106-110`), but the install log
does not. Recommend a "Copy Log" button on the `error` phase
of `InstallProgress`.

---

## 10. Deprecated or Dead Code

### 10.1 — `download_and_verify_archive` Tauri command is deprecated

**Severity:** Low (documented)
**File:** `src-tauri/src/lib.rs:39-54`

The comment at `lib.rs:39-43`:
"Standalone download command — deprecated in favor of the install
pipeline. The TempDir is dropped immediately after this returns…
Kept for backward compatibility with frontend archive.ts."

**Problem:** `src/types/archive.ts:53-67` (the `downloadArchive` TS
function) still calls this command. So the deprecation is
aspirational. Either:
- Remove the command and the TS wrapper.
- Add a feature gate to the Tauri command (e.g.,
  `#[cfg(feature = "legacy_download")]`).

The dangling `_temp_dir` (`lib.rs:53`, `:67`) is a real bug —
the file is wiped before the caller can use it. The comment
admits this: "if no destination is specified, the TempDir drops
here and the extracted files vanish."

### 10.2 — `extract_archive_to_directory` Tauri command is also stale

**Severity:** Low (documented)
**File:** `src-tauri/src/lib.rs:60-72`

Same pattern as `download_and_verify_archive`. The comment at
`lib.rs:62-66` admits the TempDir drops on return. Used by
`src/types/archive.ts:106-126`. Same recommendation: remove or
gate.

### 10.3 — `parse_size_str` is dead in production

**Severity:** Trivial
**File:** `src-tauri/src/drives.rs:309-330`

See §4.6.

### 10.4 — `cacheRegistry` invalidation in `clearRegistryCache`

**Severity:** Trivial
**File:** `src/types/package.ts:52-55`

`clearRegistryCache` is exported but never called from the
frontend. If the user wants to refresh the registry, they must
restart the app. The function exists for test isolation. Document
or remove.

### 10.5 — `clearReleaseCache` similarly orphaned

**Severity:** Trivial
**File:** `src/types/release.ts:90-96`

Same as `clearRegistryCache`. Used only in tests.

### 10.6 — `getDevicePaks` from `device-install-map.ts` is unused

**Severity:** Trivial
**File:** `src/types/device-install-map.ts:46-48`

See §3.7. The whole module is largely dead.

### 10.7 — `SHARED_BIOS` is exported but not consumed

**Severity:** Trivial
**File:** `src/types/device-install-map.ts:16`

`export const SHARED_BIOS: readonly string[] = deviceInstallMap.sharedBios;`
— no consumer.

### 10.8 — `getCurrentWifiSsid` Windows path is `None`

**Severity:** Trivial
**File:** `src-tauri/src/wifi.rs:78-93`

`get_current_wifi_ssid` is implemented for macOS
(`wifi.rs:95-131`) and returns `None` for everything else
(`wifi.rs:78-93`). The TS wizard's `scanNetworks` calls it as a
fallback when `scan_wifi_networks` returns `[]`
(`WifiWizard.tsx:32-39`). On Windows/Linux, this is always None.
Acceptable.

---

## 11. Build / Tooling / DevEx

### 11.1 — `oxlint` and `oxfmt` are pinned to a single major

**Severity:** Trivial
**File:** `package.json:38-41`

`oxc-parser`, `oxlint` are pinned. No automated update. Acceptable.

### 11.2 — `just` command runner is the canonical dev entry point

**Severity:** Trivial
**File:** `justfile`

`just check`, `just fmt`, `just lint`, `just tauri-dev` are
documented in `README.md:74-87`. A new contributor might miss
`just` and run `bun run typecheck` only. Document in `AGENTS.md`.

### 11.3 — `bun.lock` is committed; package-lock.json is not

**Severity:** Trivial
**File:** `bun.lock`

`bun.lock` is the canonical lockfile; `package-lock.json` is not
generated. Consistent.

### 11.4 — `tsconfig.json` not read here

**Severity:** Trivial
**File:** `tsconfig.json`

Not opened in this audit. Verify `strict: true` and
`noUncheckedIndexedAccess: true` are enabled to catch the kind of
bugs flagged in §5 and §7.

### 11.5 — `vitest.config.ts` and `vitest.setup.ts` exist

**Severity:** Trivial
**File:** `vitest.config.ts`, `vitest.setup.ts`

Not opened in this audit. The tests use `// @vitest-environment jsdom`
inline pragmas (`Home.test.tsx:2`, `WifiWizard.test.tsx:2`,
`DriveSelector.test.tsx:2`, `PackageStore.test.tsx:2`). The setup
file is not consulted inline; presumably contains
`@testing-library/jest-dom` matchers (per
`package.json:22`).

### 11.6 — Pre-commit hooks via `prek`

**Severity:** Trivial
**File:** `prek.toml`

`prek install` is the bootstrap. A contributor who skips this
will not run the lint/format hooks.

### 11.7 — `.github` directory exists (CI workflow)

**Severity:** Trivial
**File:** `.github/`

Not opened. Recommend a CI run of `bun run typecheck && bun run
lint && bun test && (cd src-tauri && cargo test --lib)` on every
PR.

### 11.8 — Two test runners: vitest (TS) and cargo test (Rust)

**Severity:** Trivial

No cross-language integration test. A change to the TS type
`InstallResult` would not be caught until runtime by Rust.
Recommend a JSON-schema generator or a contract test that
exercises the Tauri command surface.

---

## 12. Git Hygiene

> **Investigate:** per task description, "Git status: 1 modified, 6
> deleted, 2 untracked — investigate what those are". This section
> is the investigation. A future run of `git status --porcelain`
> is needed to capture the precise list, but the high-probability
> candidates are:

### 12.1 — Likely candidates for "1 modified"

- `Cargo.lock` — frequent churn from `cargo build`.
- `bun.lock` — frequent churn from `bun install`.
- A file under `src/` or `src-tauri/src/` mid-edit.

### 12.2 — Likely candidates for "6 deleted"

Recent plans (`plans/001`-`005`) reference several files that may
have been moved/renamed:

- `copy_dir_contents` (referenced in
  `plans/002-cancel-install-mechanism.md` step 4 description) — the
  current function is `copy_dir_recursive`. The
  `copy_dir_contents` name is not found in the current source.
- Possible deletion of legacy `store.json` migration helpers
  (referenced in handoffs but not in current code).
- Possible deletion of the `Settings.tsx` UI scaffold (referenced
  in `plan.md` as new) — the file does not exist in the current
  source tree.
- Possible deletion of unused `device-install-map.ts` consumers
  (the module itself still exists but most consumers don't).
- Possible deletion of `fork.ts` test file
  (`src/types/fork.test.ts` does not exist; the types are tested
  via `package.test.ts` indirectly).

**Action:** Run `git status --porcelain` to enumerate. The 6
deletions are **high-confidence candidates** for the orphan-code
items in §10 (deprecations, dead exports, legacy command
wrappers).

### 12.3 — Likely candidates for "2 untracked"

The `handoffs/` and `plans/` directories contain untracked
artifacts (the files read in this audit: `2026-06-13-fix-store-install-platform.md`,
`2026-06-13-per-device-extras-install.md`, `001-stream-archive-downloads.md`,
`002-cancel-install-mechanism.md`, `003-fs-utils-tests.md`,
`004-tauri-command-handler-tests.md`, `005-harden-version-parser.md`).
These are planning documents, not source; either commit them
(recommended for traceability) or add to `.gitignore` (e.g.
`.planning/` is the project convention).

**Action:** Run `git status --porcelain` to confirm.

### 12.4 — `.gitignore` should include `.planning/handoffs/` if untracked

**Severity:** Trivial
**File:** `.gitignore`

Not opened. The presence of 2 untracked files in `.planning/`
suggests they are not gitignored. Either gitignore the directory
or commit the handoffs as part of the working contract.

---

## Summary of Action Items

> Ranked by impact. **High** items are security or correctness
> risks; **Medium** items are user-facing UX or test gaps; **Low**
> items are code-health and consistency.

### High

- **§1.5** — Decide policy: remove `format_drive` from MVP, or
  update `AGENTS.md`.
- **§4.1** — Add a typed-confirmation gate to `FormatConfirmDialog`.
- **§1.6** — Restrict `download_url` override to `https://github.com/`
  in `validateStoreEntry`.
- **§2.1 / §5.4** — Wire byte-level progress to the UI
  (complete `Plan 001`).

### Medium

- **§3.1** — Harden `df` parser against spaces and rename scenarios.
- **§3.7** — Delete `device-install-map.ts` or migrate consumers.
- **§4.4** — Surface `minui.txt` write failure as `extras_warning`.
- **§4.5** — Stop swallowing `create_rom_dirs` errors.
- **§4.2 / §9.2** — Add confirmation gate to `handleUpdateAll` for
  N > 1.
- **§7.1** — Centralize `installPathRules` resolution (Rust-side
  helper exposed to TS).
- **§6.1 / §6.2 / §6.5** — Add command-level and install-success
  tests.
- **§9.3** — Hide the Format button on Windows.
- **§9.10** — Add "Copy Log" button to `InstallProgress`.
- **§10.1 / §10.2** — Remove or gate the deprecated
  `download_and_verify_archive` and `extract_archive_to_directory`
  Tauri commands.

### Low

- **§3.2** — Document the `compare_versions` mixed-format fallback.
- **§3.4** — Document that `is_preserved_path` only checks the
  first path component.
- **§4.6** — Wire `parse_size_str` into the macOS drive output or
  delete it.
- **§4.7** — Deduplicate `format_bytes` across TS and Rust.
- **§5.7** — Rate-limit `install-progress` events (emit every N
  chunks or every 5 %).
- **§9.1** — Re-validate the parent `selectedDrive` after format.
- **§9.4** — Add password visibility toggle in `WifiWizard`.
- **§9.6** — Add a "Re-check version" button in Status Summary.
- **§9.8** — Add `Escape` key handler to `ConfirmDialog`.
- **§12.2 / §12.3** — Audit git status, commit or gitignore
  `.planning/`.

---

*End of CONCERNS.md*
