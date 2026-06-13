# Session Handoff Plan

## 1. Primary Request and Intent

Fix the Package Store install path so Dreamcast (and all PAK packages) install to the correct per-platform subdirectory. Current bug: the store shows "Installs to: Emus/trimui-smart-pro/DC.pak/" but should show "Installs to: Emus/tg5040/DC.pak/".

The existing `DeviceProfile.extrasPlatform` field (already defined in `device.ts`) is the correct mapping — the Package Store currently ignores it and uses the device ID directly instead.

## 2. Key Technical Concepts

- **Device ID vs extrasPlatform**: `selectedDevice` is `trimui-smart-pro`, but the extras folder is `tg5040`. Mapping lives in `DeviceProfile.extrasPlatform` in `device.ts`
- **Package install path**: Rust `install_package()` in `package.rs` resolves path as `{targetDir}/{platform}/{pakName}.pak/` — it needs the extras platform name, not the device ID
- **Display vs install**: Both `installDestination()` function and `handleInstall()` in `PackageStore.tsx` use `selectedDevice` — both must be fixed
- **`getDeviceProfile()`**: Exists in `device.ts`, not currently imported in `PackageStore.tsx`

## 3. Files and Code Sections

### `src/PackageStore.tsx`

- **Why important**: Core package store UI — handles display paths AND actual install API calls
- **Bug location 1 — installDestination()** (line ~297): Uses `selectedDevice` for path display

```typescript
function installDestination(
  pkg: PackageRegistryEntry,
  selectedDevice: string | null,
): string {
  const baseDir = pkg.category === "Emulators" ? "Emus" : "Tools";
  const device = selectedDevice || "{platform}";
  const pakName = pkg.installPathRules.pakName || pkg.name.replace(/\s+/g, ".");
  return `${baseDir}/${device}/${pakName}.pak/`;
}
```

- **Bug location 2 — handleInstall()** (line ~108): Passes `platform: selectedDevice` to Rust backend

```typescript
const result = await installPackage({
  artifactUrl: pkg.artifactUrl,
  checksum: pkg.checksum || undefined,
  sdMount: selectedDrive,
  targetDir: pkg.installPathRules.targetDir,
  extractToRoot: pkg.installPathRules.extractToRoot,
  pakName: pkg.installPathRules.pakName || pkg.name.replace(/\s+/g, "."),
  platform: selectedDevice, // ← BUG: should be extrasPlatform
});
```

### `src/types/device.ts`

- **Why important**: Contains `getDeviceProfile()` and `DeviceProfile` with `extrasPlatform` mapping
- **Not currently imported** in `PackageStore.tsx` — needs to be added

### `src-tauri/src/package.rs`

- **Why important**: Backend install function. Already correct — just needs the right platform string passed from frontend
- **Path resolution** (line ~158): `.join(rules.target_dir).join(platform).join(format!("{}.pak", rules.pak_name))` — `platform` comes from frontend

### `src/App.tsx`

- **Why important**: Passes `selectedDevice` (device ID) to `PackageStore`. This is fine — PackageStore just needs to resolve it locally.

```typescript
<PackageStore
	selectedDevice={selectedDevice}
	selectedDrive={selectedDrive.mount_path}
/>
```

### `src-tauri/src/install.rs`

- **Why important**: Contains the extra files copy logic that was correctly plumbed with `extras_platform`. No changes needed here — just context for understanding the full pattern.

## 4. Problem Solving

**Solved this session:**

- Added `extrasPlatform` to `DeviceProfile` in `device.ts` with correct mappings
- Rewrote `copy_extras_files()` in `install.rs` to filter by extras platform
- Added real-time install progress events (`InstallProgressEvent`) via Tauri events
- Updated `ConfirmDialog` to show install plan with device-specific extras paths
- All 49 Rust tests pass, TypeScript typecheck and lint pass

**Ongoing:**

- Package Store install paths still use device ID instead of extras platform — the fix is straightforward but was interrupted

## 5. Pending Tasks

- **Fix `PackageStore.tsx`**: Import `getDeviceProfile`, resolve `extrasPlatform`, use it in both `installDestination()` and `handleInstall()`
- **Run typecheck + lint**: Verify no type errors after changes
- **Run cargo test**: Verify Rust tests still pass (no Rust changes expected)

## 6. Current Work

Was about to fix `PackageStore.tsx` to use `extrasPlatform` instead of `selectedDevice`. The key insight is:

1. At the top of the `PackageStore` function, derive the extras platform:

   ```typescript
   import { getDeviceProfile } from "./types/device";
   // Inside component:
   const profile = selectedDevice ? getDeviceProfile(selectedDevice) : null;
   const extrasPlatform =
     profile?.extrasPlatform || selectedDevice || "{platform}";
   ```

2. Use `extrasPlatform` in `installDestination()` instead of raw `selectedDevice`

3. Pass `extrasPlatform` in `handleInstall()`:

   ```typescript
   platform: extrasPlatform,
   ```

4. The user called `/handoffs` to create this handoff before the fix was implemented.

## 7. Next Step

Open `src/PackageStore.tsx` and:

1. Add `import { getDeviceProfile } from "./types/device";` at the top
2. Inside the `PackageStore` component, before the render, resolve the extras platform:
   ```typescript
   const profile = selectedDevice ? getDeviceProfile(selectedDevice) : null;
   const extrasPlatform =
     profile?.extrasPlatform || selectedDevice || "{platform}";
   ```
3. In `handleInstall()`, change `platform: selectedDevice` to `platform: extrasPlatform`
4. In `installDestination()`, change the function to accept and use the extras platform instead of the device ID. Either:
   - a) Add `extrasPlatform` parameter to the function, or
   - b) Call `getDeviceProfile` inside the function
5. Update the `PackageCard` render to use the resolved extras platform
6. Run `bun run typecheck && bun run lint && cd src-tauri && cargo test`
