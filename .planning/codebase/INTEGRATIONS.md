# External Integrations

## GitHub API

**Endpoint:** `https://api.github.com/repos/shauninman/MinUI/releases/latest`

Used by `src/types/release.ts` to fetch the latest MinUI release metadata:
- Version number
- Archive URLs (base + extras)
- Checksums
- Release notes

Also used by fork support (`src/types/fork.ts`) to query alternative MinUI forks.

**Rate limiting:** 60 req/hr unauthenticated, 5,000 req/hr with `GITHUB_TOKEN`.

## GitHub Releases (Archive Downloads)

**Endpoint:** `https://github.com/*/releases/download/*`

Archive files (`.zip`) are downloaded from GitHub Releases. The CSP allows `https://github.com` and `https://*.githubusercontent.com`.

## Package Registry

**Endpoint:** `https://packages.minui.dev/registry/index.json`

Fetched by `src/types/package.ts` → `fetchPackageRegistry()`.

**Format:** Static JSON with `emu_paks` and `tool_paks` arrays. Each entry has:
- `name`, `version`, `repository`, `pak_name`
- Optional: `description`, `checksum`, `device[]`, `download_url`
- Emu paks require `rom_folder`

**Caching:** 5-minute TTL via `RegistryCache` class in `package.ts`. Falls back to bundled `src/types/store.json` when the remote fetch fails.

## No External Services

This application does not integrate with:

| Category | Status |
|----------|--------|
| Authentication providers | None |
| Databases | None |
| Payment processors | None |
| Email services | None |
| Analytics / Telemetry | None |
| Error tracking (Sentry, etc.) | None |
| CDN | None |
| Cloud storage | None |

The application operates entirely locally — all data lives on the user's SD card.
