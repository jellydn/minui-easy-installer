# External Integrations

## GitHub API

**Endpoint**: `https://api.github.com/repos/{owner}/{repo}/releases/latest`

Used by `src/types/release.ts` (`fetchMinUIRelease`) to fetch the latest MinUI release from any configured fork. Defaults to `shauninman/MinUI`.

### Data Flow

1. Frontend calls `fetchMinUIRelease(fork)` with a `ForkConfig` (`src/types/fork.ts`)
2. Release data is parsed by `parseGitHubRelease()` — extracts `tag_name`, finds assets with "base" and "extras" in filenames
3. Results cached in a session-scoped `Map<string, MinUIRelease>` keyed by `owner/repo`

### CSP Allowlist

`tauri.conf.json` CSP includes:
- `api.github.com`
- `github.com`
- `*.githubusercontent.com`

### Fork Support

Three preset forks in `src/types/fork.ts`:
- `shauninman/MinUI` (official)
- `danklammer/MinUI-Zero`
- `jellydn/MinUITSP` (TrimUI focus)

Custom forks accepted via `owner/repo` input in Settings UI.

## Package Registry

**Endpoint**: `https://packages.minui.dev/registry/index.json`

Fetched by `src/types/package.ts` (`fetchPackageRegistry`). Static JSON with package metadata, download URLs, and per-device platform paths.

### Caching

- `RegistryCache` class with configurable TTL (default 5 minutes)
- `clearRegistryCache()` exported for testing
- Schema validated by `src/types/validate.ts` before use

### Store Schema

`src/types/store.json` defines the expected schema for the registry JSON, including `StoreEmuPak`, `StoreToolPak`, and `StoreRegistry` interfaces.

## CSP

`tauri.conf.json` enforces a tightly scoped Content Security Policy:
- `packages.minui.dev` — package registry
- `api.github.com` — release API
- `github.com` + `*.githubusercontent.com` — release downloads
- `tauri://localhost` — IPC

## No Other Integrations

- No database — all state is filesystem-based (SD card, localStorage)
- No auth providers — desktop app, no user accounts
- No payment processing
- No monitoring/analytics
- No email/SMS
- No webhooks
