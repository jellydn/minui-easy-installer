# Plan: Custom Fork Support (e.g. MinUI-Zero)

This plan outlines the design and implementation steps for allowing the MinUI Easy Installer to fetch and install base releases from custom community forks (e.g., [MinUI-Zero](https://github.com/danklammer/MinUI-Zero)) instead of only the official `shauninman/MinUI`.

---

## 1. Compatibility & Findings

Community forks like MinUI-Zero share a highly compatible structure with the official MinUI:
- **Archive Format**: Utilises `-base.zip` and `-extras.zip` release assets.
- **Internal Structure**: Folder structure contains directories named after supported devices (e.g., `trimui-brick/`, `miyoo-mini-plus/`).
- **Path Rules**: Emus and Tools are installed in matching directories on the SD card.
- **Key differences**:
  1. Asset name prefix (e.g. `MinUI-Zero-20250525-1-base.zip` vs `MinUI-25.06.12-base.zip`).
  2. GitHub Owner/Repository endpoint.
  3. Version prefix metadata written/parsed (e.g. `MinUI-Zero v20250525` vs `MinUI v2025.01.01`).

---

## 2. Impact Analysis

### Frontend (TypeScript / React)

1. **`src/types/fork.ts` (New File)**
   - Define `ForkConfig` type (label, owner, repo, versionPrefix).
   - Define presets: Official MinUI (`shauninman/MinUI`), MinUI-Zero (`danklammer/MinUI-Zero`), and a Custom option.
   - Implement `buildReleaseUrl(fork: ForkConfig)` utility.

2. **`src/types/release.ts`**
   - Update `fetchMinUIRelease` to accept `fork: ForkConfig`.
   - Dynamic API url fetching instead of using the hardcoded `GITHUB_API_URL` constant.
   - Scope the cache key per-fork (e.g. `owner/repo`) so switching forks invalidates cache.

3. **`src/hooks/useVersionCheck.ts`**
   - Update `useVersionCheck` state/hook signature to accept `fork: ForkConfig` and query the selected fork.

4. **`src/App.tsx` & `src/Settings.tsx` (New UI)**
   - Add a Settings tab/screen.
   - Lift selected fork state to `App.tsx` (loaded from `localStorage` on mount).
   - Let users select a preset or configure a custom GitHub `owner/repo`.

5. **`src/Home.tsx`**
   - Pass the selected `forkConfig` to `fetchMinUIRelease` and `installMinui`.
   - Update UI headers and labels to display the fork name (e.g. "Install MinUI-Zero").

---

## 3. Backend (Rust)

1. **`src-tauri/src/install.rs`**
   - Pass an optional `fork_name: Option<String>` in `InstallOptions` to backend.
   - Use `fork_name` when writing the `minui.txt` version label on the SD card root (e.g., `MinUI-Zero 20250525` instead of `MinUI 20250525`).

2. **`src-tauri/src/version.rs`**
   - Update version pattern detection logic. Currently, it strips the hardcoded `"MinUI "` prefix. We will update `parse_minui_version()` to look for any prefix followed by a valid version sequence (e.g. using a flexible prefix pattern matching or splitting and parsing).

---

## 4. Execution Phases

### Phase 1: Models & Logic
- Create [fork.ts](file:///Users/huynhdung/src/tries/2026-06-13-minui-installer/src/types/fork.ts) containing types and presets.
- Update [release.ts](file:///Users/huynhdung/src/tries/2026-06-13-minui-installer/src/types/release.ts) to parameterise the fetch logic.
- Update tests: [release.test.ts](file:///Users/huynhdung/src/tries/2026-06-13-minui-installer/src/types/release.test.ts) and [useVersionCheck.test.ts](file:///Users/huynhdung/src/tries/2026-06-13-minui-installer/src/hooks/useVersionCheck.test.ts).

### Phase 2: Rust Backend Modifications
- Update `InstallOptions` in [install.rs](file:///Users/huynhdung/src/tries/2026-06-13-minui-installer/src-tauri/src/install.rs) to accept `fork_name`.
- Generalise prefix detection in [version.rs](file:///Users/huynhdung/src/tries/2026-06-13-minui-installer/src-tauri/src/version.rs) to handle arbitrary prefixes (e.g., `"MinUI-Zero"`, `"MinUI"`).
- Run cargo tests.

### Phase 3: Settings Screen & App Integration
- Implement the Settings UI component [Settings.tsx](file:///Users/huynhdung/src/tries/2026-06-13-minui-installer/src/Settings.tsx).
- Add fork selection state to [App.tsx](file:///Users/huynhdung/src/tries/2026-06-13-minui-installer/src/App.tsx) and persist it to localStorage.
- Integrate the active fork configuration into [Home.tsx](file:///Users/huynhdung/src/tries/2026-06-13-minui-installer/src/Home.tsx) installation flow.
- Verify UI visuals, update typechecking, and ensure tests pass.
