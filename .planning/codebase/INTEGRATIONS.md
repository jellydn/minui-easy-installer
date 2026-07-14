# External Integrations

## Overview

The MinUI Easy Installer connects to external services for release discovery, package management, and artifact downloads. All external connections are governed by a strict Content Security Policy.

## GitHub API

**Purpose**: Discover the latest MinUI release.

- **Endpoint**: `https://api.github.com/repos/shauninman/MinUI/releases/latest`
- **Used by**: `src/types/release.ts` (ts) / `src-tauri/src/download.rs` (rs)
- **Data**: Release metadata, version tag, asset URLs, SHA-256 checksums
- **Auth**: None (public repository, rate-limited by GitHub)

## GitHub Releases (Artifact Downloads)

**Purpose**: Download MinUI base and extras archives, plus package artifacts.

- **Domains**: `github.com`, `*.githubusercontent.com`
- **Used by**: `src-tauri/src/download.rs` — streaming HTTP downloads
- **Features**: Streaming downloads with progress callbacks, SHA-256 checksum verification, cancellation support via `CancellationToken`

## Package Registry

**Purpose**: Catalog of community-contributed emulator and utility packages.

- **Remote URL**: `https://packages.minui.dev/registry/index.json`
- **Bundled fallback**: `src/types/store.json`
- **Used by**: `src/types/package.ts` — `fetchPackageRegistry()`
- **Caching**: Session-scoped (cleared on app restart via `clearRegistryCache()`)
- **Schema**: Validated before use (treat registry data as untrusted)
  - `emu_paks[]` — Emulator packages with `name`, `repository`, `version`, `pak_name`, `rom_folder`
  - `tool_paks[]` — Utility packages with `name`, `repository`, `version`, `pak_name`, optional `device[]` filter
- **Error handling**: Falls back to bundled `store.json` on network failure

## Package Artifact Downloads

**Purpose**: Download individual `.pak.zip` files for community packages.

- **Pattern**: `https://github.com/{owner}/{repo}/releases/download/{version}/{pak_name}.pak.zip`
- **Override**: Individual tool packages can specify a custom `download_url`
- **Used by**: `src-tauri/src/package.rs` — `install_package` Tauri command

## CSP Whitelist

All external domains are explicitly allowed in `src-tauri/tauri.conf.json`:

| Domain | Purpose |
|--------|---------|
| `packages.minui.dev` | Package registry JSON |
| `api.github.com` | GitHub REST API (release metadata) |
| `github.com` | Release asset downloads |
| `*.githubusercontent.com` | Raw content downloads |

No other external domains are accessible from the frontend.

## No External Auth, Databases, or Payments

The application is fully local-first with no user accounts, authentication providers, databases, or payment integrations.
