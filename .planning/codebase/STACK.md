# Technology Stack

**Analysis Date:** 2026-06-13

## Languages

**Primary:**
- TypeScript 5.6.3 - Frontend logic and components
- Rust (Edition 2021) - Backend logic, file system operations, and Tauri shell

**Secondary:**
- HTML/CSS - Frontend structure and styling
- JavaScript - Configuration files (e.g., .eslintrc.cjs)

## Runtime

**Environment:**
- Node.js (for frontend build/dev tools)
- Tauri v2 (Desktop application runtime combining Rust backend and web frontend)

**Package Manager:**
- npm (Node Package Manager)
- Cargo (Rust Package Manager)
- Lockfile: present (`package-lock.json` and likely `Cargo.lock`)

## Frameworks

**Core:**
- React 18.3.1 - Frontend UI framework
- Tauri 2.0.0 - Desktop application framework

**Testing:**
- Vitest 4.1.8 - Frontend unit testing

**Build/Dev:**
- Vite 6.0.0 - Frontend build tool and dev server
- tsc - TypeScript compiler (for typechecking)
- oxlint 1.69.0 / oxfmt - Linting and formatting

## Key Dependencies

**Critical:**
- @tauri-apps/api 2.0.0 - Frontend API to communicate with the Rust backend
- reqwest 0.12 - Rust HTTP client for downloading archives and registry data
- tokio 1.0 - Rust async runtime for handling network and file operations
- sha2 0.10 - Rust library for checksum verification of downloaded archives
- zip 0.6 - Rust library for extracting `.zip` archives
- tempfile 3.0 - Rust library for creating temporary directories during extraction

**Infrastructure:**
- libc 0.2 / windows-sys 0.59 - OS-specific APIs for drive detection and file system operations

## Configuration

**Environment:**
- Environment variables accessed via Vite config (e.g., `TAURI_DEV_HOST`)
- Key configs required: standard Tauri environment setup (Rust toolchain, Node.js)

**Build:**
- `tauri.conf.json` - Tauri application configuration
- `vite.config.ts` - Vite build configuration
- `tsconfig.json` - TypeScript compiler configuration
- `.eslintrc.cjs` - ESLint configuration
- `prek.toml` - Pre-commit hooks configuration

## Platform Requirements

**Development:**
- Rust toolchain (cargo)
- Node.js and npm
- OS-specific build tools for Tauri (e.g., Xcode build tools on macOS, Visual Studio C++ build tools on Windows)

**Production:**
- Deployment target: Windows + macOS only (MVP)

---

*Stack analysis: 2026-06-13*
