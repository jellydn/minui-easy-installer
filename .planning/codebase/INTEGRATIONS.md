# External Integrations

**Analysis Date:** 2026-06-13

## APIs & External Services

**MinUI Release Metadata (GitHub Releases API):**

- Service - GitHub REST API, fetches the latest MinUI release (tag, base/extras archive asset URLs)
- Endpoint: `https://api.github.com/repos/shauninman/MinUI/releases/latest` (`src/types/release.ts` `GITHUB_API_URL`)
- Client: browser `fetch` from the frontend with header `Accept: application/vnd.github+json` (`src/types/release.ts` `fetchMinUIRelease`)
- Parsing: `parseGitHubRelease` strips leading `v` from `tag_name`, finds assets by name keyword `base`/`extras` via `browser_download_url` (`src/types/release.ts`)
- Auth: none (unauthenticated public API); handles 404 → `NOT_FOUND`, other non-OK → `NETWORK_ERROR`
- Consumed by the install flow UI (`src/Home.tsx` calls `fetchMinUIRelease()`)

**Package Store Registry (static JSON):**

- Service - Static package registry JSON describing installable packages
- Endpoint: `https://packages.minui.dev/registry/index.json` (`src/types/package.ts` `REGISTRY_URL`)
- Client: browser `fetch` with header `Accept: application/json` (`src/types/package.ts` `fetchPackageRegistry`)
- Validation: treated as untrusted — `validatePackageEntry` / `parsePackageRegistry` validate schema, categories, and required fields before use (`src/types/package.ts`)
- Auth: none; 404 → `NOT_FOUND`, other failures → `NETWORK_ERROR`/`PARSE_ERROR`
- Consumed by the package store UI (`src/PackageStore.tsx`)

**Archive Downloads (release & package artifacts):**

- Service - Direct HTTP(S) download of archive files referenced by release asset URLs and registry `artifactUrl`
- Client: Rust `reqwest::get` (async) (`src-tauri/src/download.rs` `download_archive`)
- Integrity: SHA-256 checksum verified via `sha2` + `hex`, case-insensitive compare (`src-tauri/src/download.rs` `verify_checksum`)
- Staging: downloaded to a `tempfile::TempDir` before extraction/copy (`src-tauri/src/download.rs`)

## Data Storage

**Databases:**

- None (no database; registry is a remote static JSON file)

**File Storage:**

- Local filesystem only — reads/writes the user's SD card mount point
  - MinUI base/extras files copied recursively to SD root/platform dir (`src-tauri/src/install.rs`)
  - Package artifacts extracted and copied per `installPathRules` (`src-tauri/src/package.rs`)
  - WiFi config written as `wifi.txt` at SD root in `SSID:/PASS:` format for MinUI Wifi.pak (`src-tauri/src/wifi.rs`)
  - Installed-package detection / version reads off the SD card (`src-tauri/src/package.rs`, `src-tauri/src/version.rs`)
  - Temp extraction staging via `tempfile` + `zip` (`src-tauri/src/download.rs`, `src-tauri/src/extract.rs`)

**Caching:**

- None

## Authentication & Identity

**Auth Provider:**

- None — all external endpoints (GitHub API, registry, downloads) are accessed unauthenticated; no API keys, tokens, or login (`src/types/release.ts`, `src/types/package.ts`, `src-tauri/src/download.rs`)

## Monitoring & Observability

**Error Tracking:**

- None (no Sentry/telemetry). Errors are surfaced as typed results to the UI (`NETWORK_ERROR`, `PARSE_ERROR`, etc.) (`src/types/release.ts`, `src/types/install.ts`)

**Logs:**

- No structured logging framework. Tauri opens devtools in debug builds (`src-tauri/src/lib.rs` `window.open_devtools()` under `#[cfg(debug_assertions)]`)

## CI/CD & Deployment

**Hosting:**

- Desktop application distributed as bundled installers via Tauri (`src-tauri/tauri.conf.json` `bundle.targets = "all"`); registry/assets hosted externally at `packages.minui.dev` and GitHub

**CI Pipeline:**

- None found in repo (no `.github/workflows` referenced); local `scripts/ralph/ralph.sh` autonomous coding loop per `AGENTS.md`

## OS-Level Integrations

**Removable drive detection (`src-tauri/src/drives.rs`, command `get_removable_drives`):**

- macOS: `diskutil list external`, with fallback `df -k` filtered to `/Volumes/` (`src-tauri/src/drives.rs`)
- Windows: `powershell -Command "Get-CimInstance -ClassName Win32_LogicalDisk | Where-Object { $_.DriveType -eq 2 } ... | ConvertTo-Json"` (DriveType 2 = removable) (`src-tauri/src/drives.rs`)
- Other OSes: returns "Unsupported platform" error (`src-tauri/src/drives.rs`)

**SD card health check (`src-tauri/src/validate.rs`, command `check_sd_card_health`):**

- macOS: `diskutil` invoked for device/health details (`src-tauri/src/validate.rs:346`)

**WiFi scanning (`src-tauri/src/wifi.rs`, command `scan_wifi_networks`):**

- macOS: Apple80211 `airport -s` (parsed by `parse_airport_output`), fallback `networksetup -listallhardwareports` (`src-tauri/src/wifi.rs`)
- Linux: `nmcli -t -f SSID dev wifi` (`src-tauri/src/wifi.rs`)
- Windows: `netsh wlan show networks mode=bssid` (parsed by `parse_netsh_output`) (`src-tauri/src/wifi.rs`)
- Returns empty list when unsupported or on failure

## Tauri Command Surface (IPC)

Registered in `src-tauri/src/lib.rs` via `tauri::generate_handler!`; invoked from frontend with `@tauri-apps/api/core` `invoke`:

- `get_removable_drives` → `src/DriveSelector.tsx`
- `download_and_verify_archive`, `verify_archive_checksum`, `extract_archive_to_directory` → `src/types/archive.ts`
- `install_minui` → `src/types/install.ts`
- `validate_installation`, `format_validation_report`, `check_sd_card_health` → `src/types/validate.ts`
- `check_minui_version` → `src/types/version.ts`
- `install_package`, `detect_installed_packages`, `check_package_updates` → `src/types/package.ts`
- `write_wifi_config`, `scan_wifi_networks` → `src/WifiWizard.tsx`

## Environment Configuration

**Required env vars:**

- None required at runtime; `TAURI_DEV_HOST` is an optional dev-only HMR host (`vite.config.ts`)
- Service URLs are hard-coded constants, not env-driven (`src/types/release.ts`, `src/types/package.ts`)

**Secrets location:**

- No secrets stored or required. Per `AGENTS.md`, WiFi passwords must never be logged in plaintext; `wifi.txt` is written to the SD card only on explicit user action (`src-tauri/src/wifi.rs`)

## Webhooks & Callbacks

**Incoming:**

- None

**Outgoing:**

- None (no webhook/callback dispatch; only outbound GET requests to GitHub API, registry, and archive URLs)

---

_Integration audit: 2026-06-13_
