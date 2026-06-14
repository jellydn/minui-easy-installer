# External Integrations

**Analysis Date:** 2026-06-14

## APIs & External Services

**GitHub Releases (MinUI):**
- GitHub API — Fetch latest MinUI release metadata
- Endpoint: `https://api.github.com/repos/shauninman/MinUI/releases/latest`
- Client: Frontend `fetch()` with `Accept: application/vnd.github+json` header
- Defined in: `src/types/release.ts` (line 22-23, 77-116)
- Auth: None (public API, rate-limited to 60 req/hr unauthenticated)

**GitHub Releases (Package Artifacts):**
- GitHub Releases — Download `.pak.zip` archives for packages
- Pattern: `https://github.com/{repo}/releases/download/{version}/{pak_name}.pak.zip`
- Client: Rust `reqwest` 0.12 (async HTTP GET)
- Defined in: `src-tauri/src/download.rs` (line 37-97)
- Auth: None (public releases)

**Package Registry (Static JSON):**
- Local static JSON — Package store catalog
- Source: `src/types/store.json` (imported at build time)
- Client: TypeScript import (no network fetch)
- Defined in: `src/types/package.ts` (line 258-278)
- Auth: N/A (local file)

## Data Storage

**Databases:**
- None — No database in use

**File Storage:**
- Local filesystem only — All data stored on user's SD card or temp directories
- Temporary directories: `tempfile` crate creates temp dirs for downloads/extractions
- SD card writes: MinUI files, WiFi config (`wifi.txt`), package PAKs

**Caching:**
- None — No caching layer; archives downloaded fresh each time

## Authentication & Identity

**Auth Provider:**
- None — No user authentication system
- GitHub API accessed unauthenticated (public endpoints only)

## Monitoring & Observability

**Error Tracking:**
- None — No Sentry, Bugsnag, or similar error tracking service

**Logs:**
- Tauri debug console — DevTools open automatically in debug builds (`src-tauri/src/lib.rs` line 185-189)
- Rust `println!`/`eprintln!` — Standard output/error logging
- Frontend console — Browser console via WebView

## CI/CD & Deployment

**Hosting:**
- Not yet configured — No CI/CD files detected (no `.github/workflows/`, no `Jenkinsfile`, etc.)

**CI Pipeline:**
- `just pre-commit` — Runs `prek run --all-files` (pre-commit hook runner)
- `just check` — Full check: `bun run lint` + `bun run typecheck` + `cargo fmt --check` + `cargo clippy -- -D warnings`

## OS-Level Integrations

**Drive Detection (macOS):**
- Command: `df -k` — Lists volumes mounted under `/Volumes/`
- Command: `diskutil info` — Gets filesystem type per volume
- C FFI: `libc::statvfs` — Gets total disk size via syscall
- Defined in: `src-tauri/src/drives.rs` (line 14-66)

**Drive Detection (Windows):**
- Command: `powershell Get-CimInstance Win32_LogicalDisk` — Lists removable drives (DriveType=2)
- Defined in: `src-tauri/src/drives.rs` (line 238-298)

**Drive Formatting (macOS):**
- Command: `diskutil info` — Finds device node and partition info
- Command: `diskutil unmount` — Unmounts partition before format
- Command: `diskutil eraseDisk FAT32` — Formats drive to FAT32 (max 11-char volume label)
- Defined in: `src-tauri/src/drives.rs` (line 75-150)
- NOTE: Formatting NOT supported on Windows (stub returns error)

**WiFi Scanning (macOS):**
- Command: `/System/Library/PrivateFrameworks/Apple80211.framework/.../airport -s` — Scans available networks
- Fallback: `system_profiler SPAirPortDataType` — Gets currently connected SSID (macOS 14.4+ compatible)
- Defined in: `src-tauri/src/wifi.rs` (line 141-166, 77-113)

**WiFi Scanning (Windows):**
- Command: `netsh wlan show networks mode=bssid` — Scans available WiFi networks
- Defined in: `src-tauri/src/wifi.rs` (line 217-255)

**WiFi Scanning (Linux):**
- Command: `nmcli -t -f SSID dev wifi` — Scans via NetworkManager
- Defined in: `src-tauri/src/wifi.rs` (line 189-215)

**WiFi Config Writing:**
- File write: `{sd_mount}/wifi.txt` — MinUI WiFi config format (`SSID:PASSWORD` per line)
- Handles: SSIDs with spaces, comment preservation, SSID deduplication on update
- Defined in: `src-tauri/src/wifi.rs` (line 18-59)

**Free Space Detection:**
- macOS/Unix: `libc::statvfs` — Gets available bytes from filesystem
- Fallback: `diskutil info` — Parses "Total Size" / "Disk Size" lines
- Defined in: `src-tauri/src/validate.rs` (line 353-374)

**Filesystem Detection:**
- macOS: `diskutil info` — Parses "File System Personality" line
- Defined in: `src-tauri/src/validate.rs` (line 330-351)

## Tauri Command Surface (IPC)

All 16 registered commands (registered in `src-tauri/src/lib.rs` line 166-183):

| # | Command | Frontend Consumer | Purpose |
|---|---------|-------------------|---------|
| 1 | `get_removable_drives` | Drive selection UI | Lists removable drives |
| 2 | `format_drive` | Drive format UI | Formats drive to FAT32 |
| 3 | `download_and_verify_archive` | Install flow | Downloads + checksums archive |
| 4 | `verify_archive_checksum` | Install flow | Verifies SHA-256 of local file |
| 5 | `extract_archive_to_directory` | Install flow | Extracts ZIP to temp dir |
| 6 | `install_minui` | Install flow | Full MinUI install with progress events |
| 7 | `validate_installation` | Post-install | Validates MinUI files on SD card |
| 8 | `format_validation_report` | Post-install | Formats validation result to text |
| 9 | `check_minui_version` | Update check | Checks installed vs latest version |
| 10 | `install_package` | Package store | Downloads + installs a package PAK |
| 11 | `write_wifi_config` | WiFi setup | Writes SSID:PASSWORD to wifi.txt |
| 12 | `scan_wifi_networks` | WiFi setup | Scans nearby WiFi networks |
| 13 | `get_current_wifi_ssid` | WiFi setup | Gets currently connected SSID |
| 14 | `detect_installed_packages` | Package store | Lists packages already on SD card |
| 15 | `check_package_updates` | Package store | Compares installed vs registry versions |
| 16 | `check_sd_card_health` | Diagnostics | Comprehensive SD card health check |

**Progress Events:**
- `install-progress` — Emitted during `install_minui` with `InstallProgressEvent` payloads (`src-tauri/src/install.rs`)

## Environment Configuration

**Required env vars:**
- `TAURI_DEV_HOST` — Optional; Vite dev server HMR host for remote debugging (`vite.config.ts` line 4)

**Secrets location:**
- None — No secrets, API keys, or tokens required
- GitHub API accessed without authentication

## Webhooks & Callbacks

**Incoming:**
- None — No webhook endpoints

**Outgoing:**
- None — No outbound webhooks or callbacks
