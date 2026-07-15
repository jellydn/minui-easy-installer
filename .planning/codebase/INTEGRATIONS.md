# External Integrations

## GitHub Releases API

- **Endpoint**: `https://api.github.com/repos/{owner}/{repo}/releases/latest`
- **Purpose**: Fetch MinUI release metadata (version, archive URLs, checksums)
- **Config**: `src/types/release.ts` (`fetchMinUIRelease`)
- **Fork support**: User-supplied `ForkConfig` with custom owner/repo determines the API URL

## Package Registry

- **Endpoint**: `https://packages.minui.dev/registry/index.json`
- **Purpose**: Fetch available add-on packages (WiFi, SSH, tools, etc.)
- **Config**: `src/types/package.ts` (`fetchPackageRegistry`)
- **Caching**: Session-scoped (fetched once per app launch)

## CSP (Content Security Policy)

Defined in `src-tauri/tauri.conf.json`. Allowlisted domains:

| Domain | Purpose |
|--------|---------|
| `packages.minui.dev` | Package registry JSON |
| `api.github.com` | MinUI release metadata |
| `github.com` | Release redirects |
| `*.githubusercontent.com` | Archive downloads, fork release assets |

## SSRF Protection

The `fetch_url` Tauri command restricts HTTP fetches to a hardcoded allowlist:

```rust
const ALLOWED_URLS: &[&str] = &["https://packages.minui.dev/registry/index.json"];
```

Add new endpoints to `ALLOWED_URLS` in `src-tauri/src/lib.rs` as needed.

## WiFi Scanning (macOS)

Three-tier fallback (`src-tauri/src/wifi.rs`):

| Tier | Command | macOS Version |
|------|---------|---------------|
| 1 | `/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport -s` | < 14.4 |
| 2 | `system_profiler SPAirPortDataType` | ≥ 14.4 (airport removed) |
| 3 | Current SSID fallback only | All |

## WiFi Scanning (Windows)

- `netsh wlan show networks mode=bssid` for network scanning
- `netsh wlan show interfaces` for current SSID

## Filesystem Detection

| OS | Command | Source |
|----|---------|--------|
| macOS | `diskutil info <mount>` | `src-tauri/src/health.rs` |
| Windows | `fsutil fsinfo volumeinfo <mount>` | `src-tauri/src/health.rs` |
| Linux | Not supported (MVP) | Returns `None` |

## Drive Detection

| OS | Method | Source |
|----|--------|--------|
| macOS | `diskutil list -plist` + parsing | `src-tauri/src/drives.rs` |
| Windows | Win32 API (`GetLogicalDrives`, `GetDriveTypeW`) | `src-tauri/src/drives.rs` |

## HTTP Client

- Single `reqwest::Client` via `OnceLock` with 10-second timeout and connection pooling
- Used for: GitHub API, package registry, archive downloads (streaming)
- Defined in `src-tauri/src/lib.rs` (`http_client()`)

## Archive Downloads

- Streaming download via `reqwest` with progress callbacks
- SHA-256 checksum verification after download (`download.rs`)
- Archives downloaded to temp dir before extraction (`pipeline.rs`)
