# Integrations

## External APIs

### GitHub REST API

- **Endpoint**: `https://api.github.com/repos/{owner}/{repo}/releases/latest`
- **Purpose**: Fetch the latest MinUI release (or community fork release) for version checking and download URLs
- **Code**: `src/types/release.ts` (`fetchMinUIRelease`), `src/types/fork.ts` (`buildReleaseUrl`)
- **Auth**: None (public repository, unauthenticated requests)
- **Rate limit**: 60 req/hour unauthenticated; sufficient for the install flow (one fetch per session)

### GitHub Releases (asset downloads)

- **Endpoint**: `https://github.com/{owner}/{repo}/releases/download/{tag}/{asset}`
- **Purpose**: Download base and extras zip archives for MinUI installation
- **Code**: `src-tauri/src/download.rs` (`download_archive_streaming`)
- **Flow**: Fetch release metadata → parse asset URLs → stream download with progress
- **Cancellation**: `CancellationToken` checked at phase boundaries

### Package Registry

- **Endpoint**: `https://packages.minui.dev/registry/index.json`
- **Purpose**: Fetch the community package catalog with per-device platform paths
- **Code**: `src/types/package.ts` (`fetchPackageRegistry`)
- **Cache**: Session-scoped (fetched once per app launch)
- **Validation**: Schema validated before use (see `src/types/validate.ts`)

### GitHub Content (raw)

- **Domain**: `*.githubusercontent.com`
- **Purpose**: CSP allowlisted for potential raw file access
- **Current use**: Not directly fetched; included in CSP for future package asset downloads

## Package Downloads

Community packages reference GitHub release assets:

- **Pattern**: `https://github.com/{repo}/releases/download/{version}/{fileName}`
- **Code**: `src/types/package.ts` (`buildDownloadUrlForVersion`)
- **Validation**: Only `https://github.com/` repositories are accepted

## No Integrations For

| Category | Status |
|----------|--------|
| Database | ❌ None (stateless app) |
| Authentication | ❌ None (local-only desktop app) |
| Payment processing | ❌ None |
| Analytics/telemetry | ❌ None |
| Error tracking | ❌ None |
| Email | ❌ None |
| CDN | ❌ None |
| Cloud storage | ❌ None |
