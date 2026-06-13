# External Integrations

**Analysis Date:** 2026-06-13

## APIs & External Services

**MinUI Package Registry:**
- Service: Static JSON registry at `https://packages.minui.dev/registry/index.json`
- What it's used for: Fetching available MinUI releases and packages
- SDK/Client: `reqwest` (Rust HTTP client)
- Auth: None (Publicly accessible)

**Archive Downloads:**
- Service: External HTTP servers (e.g., GitHub Releases)
- What it's used for: Downloading MinUI base and extras `.zip` archives
- SDK/Client: `reqwest`

## Data Storage

**Databases:**
- None

**File Storage:**
- Local filesystem only (Target SD Card drives and temporary OS directories)
- Managed via Rust standard library (`std::fs`), `tempfile`, and `zip` crates

**Caching:**
- None explicit in the MVP

## Authentication & Identity

**Auth Provider:**
- None (The application does not require user authentication)

## Monitoring & Observability

**Error Tracking:**
- None explicitly configured. Errors are bubbled up to the frontend UI via Tauri commands.

**Logs:**
- Standard output/error via terminal running the Tauri app during development.

## CI/CD & Deployment

**Hosting:**
- Static registry hosted externally (GitHub Pages / Vercel assumed for `packages.minui.dev`)

**CI Pipeline:**
- None explicitly configured in the repository (e.g., GitHub Actions workflows are absent in the current snapshot).

## Environment Configuration

**Required env vars:**
- `TAURI_DEV_HOST` (Optional, used in `vite.config.ts` for network dev server)

**Secrets location:**
- No secrets required. The application explicitly avoids logging or managing WiFi passwords/secrets in plaintext (Constraint in AGENTS.md).

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

---

*Integration audit: 2026-06-13*
