# Integrations

This project is a Tauri v2 desktop app. Most "integrations" are outbound HTTP calls to fetch MinUI release artifacts and a package registry, with everything else operating on the local SD card via the OS.

## External HTTP APIs

### 1. GitHub Releases API — MinUI fork release metadata

- **Endpoint template**: `https://api.github.com/repos/{owner}/{repo}/releases/latest`
- **Built by**: `buildReleaseUrl(fork)` — `src/types/fork.ts:24-26`
- **Default fork (Official)**: `https://api.github.com/repos/shauninman/MinUI/releases/latest` — also kept as the legacy constant `GITHUB_API_URL` in `src/types/release.ts:29` (marked `@deprecated`)
- **Preset forks** (`src/types/fork.ts:7-19`):
  - `official` → `shauninman/MinUI`
  - `minui-zero` → `danklammer/MinUI-Zero`
- **Custom forks**: any `owner/repo` string via `buildCustomFork()` — `src/types/fork.ts:39-47`
- **Caller**: `fetchMinUIRelease(fork, fetchFn?)` — `src/types/release.ts:85-138`
  - Header: `Accept: application/vnd.github+json`
  - Returns `MinUIRelease { version, baseArchiveUrl, extrasArchiveUrl, checksums, fork }`
  - Session-scoped cache keyed by `owner/repo` via `getForkCacheKey()` — `src/types/release.ts:77-83`
  - Testable: accepts a `fetchFn` for dependency injection (`src/types/release.ts:88`)
- **Error codes** (`src/types/release.ts:11-14`): `NETWORK_ERROR`, `PARSE_ERROR`, `NOT_FOUND`
- **Parser**: `parseGitHubRelease()` — `src/types/release.ts:32-69` — extracts `tag_name` (strips leading `v`), then scans assets for names containing `base` and `extras` keywords to pick `browser_download_url` per asset

### 2. GitHub Releases — package artifact downloads

- **URL pattern**: `https://github.com/{owner}/{repo}/releases/download/{version}/{fileName}.pak.zip`
- **Built by**: `resolveArtifactUrl()` — `src/types/package.ts:154-161`
- **Source**: `repository` + `version` + `pakName` from each package entry
- **Override path**: any package can supply an explicit `download_url` field (`src/types/package.ts:280-285`) — the registry entries in `src/types/store.json:229` use this for `Grout` (RomM): `https://github.com/rommapp/grout/releases/download/v4.8.1.0/Grout-MinUI.zip`
- **Downloaded by Rust backend**: `src-tauri/src/download.rs` via `reqwest` with `stream` feature; consumed by `install_package` (`src-tauri/src/lib.rs:189-204`) and the install pipeline (`src-tauri/src/pipeline.rs`)
- **Verified via SHA-256** — `src-tauri/src/download.rs` + `verify_archive_checksum` command (`src-tauri/src/lib.rs:48-50`)

### 3. MinUI package registry

- **Endpoint**: `https://packages.minui.dev/registry/index.json`
- **Constant**: `REGISTRY_URL` — `src/types/package.ts:46`
- **Caller**: `fetchPackageRegistry()` — `src/types/package.ts:316-364`
  - **Primary path**: invoked through the Rust `fetch_url` Tauri command (`src-tauri/src/lib.rs:268-291`) — passes the URL to `reqwest::Client` with a 10s timeout
  - **Fallback path**: bundled `src/types/store.json` (32 packages) when remote fails or `fetch_url` errors (`src/types/package.ts:344-362`)
  - Session cache: `cachedRegistry` module-level variable (`src/types/package.ts:49-55`)
  - Errors: `INVALID_ENTRY`, `VALIDATION_ERROR`, `PARSE_ERROR`, `NETWORK_ERROR`, `NOT_FOUND` (`src/types/package.ts:34-41`)
- **Schema contract** (validated by `parseRegistryFromJson` + `isStoreRegistry` + `validateStoreEntry` — `src/types/package.ts:182-323`):
  - Top-level: `{ emu_paks: [...], tool_paks: [...] }`
  - Each entry must have `name`, `repository` (must start with `https://github.com/`), `version`, `pak_name`
  - Emu paks require `rom_folder`; tool paks may have `device[]`, `download_url`, `checksum` (64-char hex)
  - Validation is strict — invalid entries are dropped with logged reasons (`src/types/package.ts:294-322`)
- **Allow-listed in CSP** (`src-tauri/tauri.conf.json:24`): `connect-src ... https://packages.minui.dev ...`

## Local OS / Hardware Integrations (Rust backend)

These are not "external services" but are first-class integrations with the host OS — documented here for completeness.

### Removable drive enumeration & formatting

- **macOS path** (`src-tauri/src/drives.rs:17-77`):
  - `df -k` to list volumes, then filter `/Volumes/*` excluding `Macintosh HD*`
  - `diskutil info` via `is_removable_volume()` to confirm external/removable
  - `diskutil eraseDisk` via `format_drive` command (`src-tauri/src/lib.rs:22-24`)
- **Windows path**: `windows-sys` `Win32_Storage_FileSystem` (`src-tauri/Cargo.toml:28-30`) — used to enumerate drives / format via Win32 APIs (implementation in `src-tauri/src/drives.rs` cfg-gated sections)
- **Linux**: not in MVP per `AGENTS.md:5` — no integration present
- IPC commands: `get_removable_drives`, `format_drive` (`src-tauri/src/lib.rs:18-25`)

### WiFi scanning & configuration (macOS only)

- `src-tauri/src/wifi.rs`
  - `get_current_wifi_ssid()` — uses `system_profiler SPAirPortDataType` on macOS (`src-tauri/src/wifi.rs:75-99`); returns `None` on Windows
  - `scan_wifi_networks()` — platform-specific shell-out (cfg-gated)
  - `write_wifi_config(sd_mount, ssid, password)` — writes `<sd_root>/wifi.txt` in MinUI format `SSID:PASSWORD` per line, comments with `#`, deduplicates existing SSID
- IPC commands: `scan_wifi_networks`, `get_current_wifi_ssid`, `write_wifi_config` (`src-tauri/src/lib.rs:217-235`)

### Filesystem operations on SD card

- `src-tauri/src/fs_utils.rs` — `get_disk_space`, path helpers
- `src-tauri/src/install.rs`, `extract.rs` — archive extraction with `zip` crate
- `src-tauri/src/validate.rs` — post-install validation by checking expected files on SD
- `src-tauri/src/health.rs` — SD card health check (free space, required folders)

### Tauri IPC events (in-process, not external)

The backend emits events to the frontend over Tauri's event bus:
- `install-progress` — `InstallProgressEvent` payloads (`src-tauri/src/lib.rs:67-73`)
- `install-complete` — `InstallResult` (`src-tauri/src/lib.rs:174-176`)
- `install-error` — error string (`src-tauri/src/lib.rs:178-180`)

## Authentication

**None.** The app makes unauthenticated requests to public GitHub APIs and the public MinUI registry. There is no user auth, no token storage, no OAuth flow. Note: GitHub's API has an unauthenticated rate limit (60 req/h/IP) that the app accepts (`src/types/release.ts:97-127` shows only the public endpoint with no Authorization header).

## Webhooks

**None.** The app does not register, send, or receive webhooks. All calls are outbound from the desktop client to public REST endpoints.

## Tauri Capability Surface

- Single capability: `default.json` (`src-tauri/capabilities/default.json`) — grants only `core:default` to the `main` window
- No filesystem, dialog, shell, or HTTP permissions explicitly granted (the Rust backend handles all of these server-side)
- `withGlobalTauri: false` in `tauri.conf.json:13` — frontend uses dynamic `await import("@tauri-apps/api/core")` to call `invoke` (e.g. `src/types/package.ts:62, 75, 89, 332`)

## Data Assets Bundled With the App

These are static JSON shipped inside the binary — no network fetch needed at runtime:
- `src/types/store.json` — 32-package fallback catalog (6 emu paks + 26 tool paks) — see `src/types/store.json:1-234`
- `src/types/device-install-map.json` — 17-device install map with `devicePaks` and `sharedBios` list — `src/types/device-install-map.json:1-217`

## Network Surface Summary (for security review)

| Direction | Host | Purpose | Auth |
|-----------|------|---------|------|
| Outbound HTTPS GET | `api.github.com` | Latest release metadata for MinUI / MinUI-Zero / custom fork | None |
| Outbound HTTPS GET | `github.com/.../releases/download/...` | MinUI base + extras zip downloads | None |
| Outbound HTTPS GET | `github.com/.../releases/download/...` | Per-package `.pak.zip` artifact downloads | None |
| Outbound HTTPS GET | `packages.minui.dev` | Package registry `index.json` | None |
| Outbound | `system_profiler`, `df`, `diskutil` (macOS) | Local OS shell-outs | OS-level |
| Inbound | None | — | — |
| Webhooks | None | — | — |

All hosts are pinned in CSP `connect-src` (`src-tauri/tauri.conf.json:24`): `https://packages.minui.dev https://api.github.com https://github.com https://*.githubusercontent.com`.
