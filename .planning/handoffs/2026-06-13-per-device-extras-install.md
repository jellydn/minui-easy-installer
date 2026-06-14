# Session Handoff Plan

## 1. Primary Request and Intent

Continue fixing the per-device MinUI installation on the Home screen to match the official install guide at `/Volumes/NEXT28/README.txt`. The key insight from the README: **MinUI installs one device per SD card** — the base archive has ALL platform folders at root level, and the device auto-selects the right one on first boot. But extras (PAK emulators/tools) must go into **device-specific subdirectories**: `/Emus/{platform}/` and `/Tools/{platform}/`, not just at root.

This is especially important for the **extras installation step in `install_minui`** — currently `copy_extras_files` copies the entire extras archive to root, which dumps all platform folders instead of just the relevant platform's files.

## 2. Key Technical Concepts

- **MinUI base archive**: contains ALL platform folders at root (`trimui/`, `miyoo/`, `rg35xxplus/`, etc.) + shared files (`MinUI.zip`, `README.txt`, `em_ui.sh`). Device picks right folder on first boot. Current `copy_base_files` correctly copies everything to SD root.
- **MinUI extras archive**: contains `Emus/{platform}/...`, `Tools/{platform}/...`, and `Bios/...` at root level, for ALL platforms. Install should only copy the matching platform's folders (e.g., `Emus/tg5040/`) plus shared `Bios/`.
- **PAK structure**: PAK zips contain `.pak` directory contents (launch.sh, bin/, lib/, config/). They go into `{Emus|Tools}/{platform}/{pakName}.pak/`.
- **`copy_dir_recursive` with skip predicate**: Shared helper in `fs_utils.rs` that accepts a closure to skip paths.
- **Platform folder names for extras**: From examining the actual extras archive: `gkdpixel`, `m17`, `magicmini`, `miyoomini`, `my282`, `my355`, `rg35xx`, `rg35xxplus`, `rgb30`, `tg5040`, `trimuismart`, `zero28`.
- **Device ID → Platform mapping**: Defined in `src/types/device.ts`. E.g., `trimui-brick` → platform `trimui`. The extras use DIFFERENT folder names (e.g., `tg5040`, `trimuismart`) than the base archive (e.g., `trimui`).

## 3. Files and Code Sections

### `src-tauri/src/install.rs`

- **Why important**: Core install flow for the Home screen. Contains `install_minui`, `copy_base_files`, `copy_extras_files`, `try_install_extras`.
- **Changes made**: `copy_base_files` was rewritten to copy EVERYTHING from archive to SD root (matching README). `try_install_extras` was extracted from inline match pyramid.
- **Current state**: `copy_extras_files` still copies the entire extras archive to `sd_root.join(extras_dir.trim_start_matches('/'))` where `extras_dir = "/"`. This puts ALL platforms' emus at root, not just the matching one.
- **Code snippet of `copy_extras_files`**:

```rust
pub fn copy_extras_files(
    extracted_extras_path: &str,
    sd_mount: &str,
    extras_dir: &str,
) -> Result<u32, String> {
    let extras_src = Path::new(extracted_extras_path);
    let sd_root = Path::new(sd_mount);
    let extras_dst = sd_root.join(extras_dir.trim_start_matches('/'));

    if !extras_src.exists() {
        return Err("Extras source directory does not exist".to_string());
    }

    fs::create_dir_all(&extras_dst)
        .map_err(|e| format!("Failed to create extras directory: {}", e))?;

    fs_utils::copy_dir_recursive(extras_src, &extras_dst, &|path| is_preserved_path(path, extras_src))
}
```

### `src-tauri/src/fs_utils.rs`

- **Why important**: Shared directory copy utility with predicate-based skip.
- **Changes made**: Created from scratch; extracted from the duplicated `copy_dir_recursive` in `install.rs` and `package.rs`.
- **Code snippet**:

```rust
pub fn copy_dir_recursive<F>(src: &Path, dst: &Path, skip: &F) -> Result<u32, String>
where
    F: Fn(&Path) -> bool,
```

### `src/types/device.ts`

- **Why important**: Maps device IDs to platform folder names. After the latest update, has 17 device profiles covering all supported platforms.
- **Key issue**: Device ID uses user-friendly keys (e.g., `trimui-brick`), `platform` maps to base archive folder names (e.g., `trimui`). BUT: extras archive uses DIFFERENT folder names. For example, TrimUI Smart Pro uses `trimuismart` in the extras archive, not `trimui`.

Device profiles (relevant subset for platform mapping gap):

```
trimui-brick → platform: "trimui" (but extras has no "trimui" — it uses "trimuismart")
trimui-smart-pro → platform: "trimui" (extras uses "trimuismart")
rg35xx-plus → platform: "rg35xxplus" (extras uses "rg35xxplus" ✓)
rg35xx-h → platform: "rg35xxplus" (extras uses "rg35xxplus" ✓)
rg35xx-sp → platform: "rg35xxplus" (extras uses "rg35xxplus" ✓)
miyoo-mini → platform: "miyoo" (extras uses "miyoomini")
miyoo-mini-plus → platform: "miyoo" (extras uses "miyoomini")
miyoo-a30 → platform: "miyoo285" (extras uses "my282" — wait that's different)
```

### `src-tauri/src/package.rs`

- **Why important**: Package store install. Updated to install into `{Emus|Tools}/{platform}/{pakName}.pak/`.
- **Current code**:

```rust
let pak_root = Path::new(sd_mount)
    .join(rules.target_dir.trim_start_matches('/'))
    .join(platform)
    .join(format!("{}.pak", rules.pak_name));
fs::create_dir_all(&pak_root)?;
let files_copied = fs_utils::copy_dir_contents(extracted, &pak_root)?;
```

### `src-tauri/src/validate.rs`

- **Why important**: Post-install validation and SD card health check. Updated to use `MinUI.zip` and `minui.txt` instead of `minui.pak`/`boot.sh`/`DMG.png`. PAK detection is now recursive (`count_pak_dirs`).

## 4. Problem Solving

**Solved:**

- Base archive install now copies ALL folders/contents to SD root (per README: "Copy all the folders from this zip file to the root of your primary card")
- Package store installs into per-platform paths (`Tools/tg5040/DC.pak/`)
- WiFi config format fixed to plain `SSID\nPASS\n` (no labels)
- Drive detection replaced with `df -k` + `diskutil info` (instead of broken `diskutil list external` parser)
- macOS WiFi detection uses `system_profiler SPAirPortDataType` (works on 14.4+ where `airport` is removed)
- Validation checks `MinUI.zip`/`minui.txt` instead of `minui.pak`/`boot.sh`/`DMG.png`
- All I/O Tauri commands made `async` to prevent UI freeze

**Ongoing:**

- **Extras install on Home screen (`install_minui`) needs per-platform filtering** — currently copies ALL platform folders from extras archive to root. Needs to:
  1. Copy `Bios/` to root (shared across devices)
  2. Copy `Emus/{platformExtrasName}/` → `Emus/` on SD
  3. Copy `Tools/{platformExtrasName}/` → `Tools/` on SD

## 5. Pending Tasks

1. **Fix extras install per-device in `install_minui`**: `copy_extras_files` should not copy all platform folders to root. It should only copy the matching platform's Emus/Tools folders plus shared Bios/. This is the **primary task**.

2. **Fix `try_install_extras`** to accept platform and filter by platform folder name.

3. **Add `extras_platform` field to `DeviceProfile`** in `src/types/device.ts` — platform folder names for extras differ from base archive folder names. For example:
   - `trimui-brick` → base `trimui`, extras `trimuismart`
   - `miyoo-mini` → base `miyoo`, extras `miyoomini`
   - `miyoo-a30` → base `miyoo285`, extras `my282`
   - Need to verify all 17 profiles' extras folder names

4. **Update `src/Home.tsx`** to pass platform through to the install flow.

5. **Update `src-tauri/src/lib.rs`** Tauri command `install_minui` to accept and forward platform info.

## 6. Current Work

Immediately before this handoff, the package store install flow was fixed to put PAKs into per-platform directories (`{Emus|Tools}/{platform}/{pakName}.pak/`). The `install_package` command now accepts `platform` and `pakName` parameters.

The Home screen's `install_minui` function was NOT yet updated with the same per-platform treatment — it still copies extras to root. The README.txt on the actual SD card says:

- "Copy all the folders from this zip file to the root of your primary card" (base install — done ✓)
- "copy just the desired updated pak(s) to the corresponding device folder in the Emus or Tools folders (eg. `/Emus/tg5040` or `/Tools/rgb30`)" (extras — NOT done ✗)

## 7. Next Step

Fix `copy_extras_files` and `install_minui` to only copy the matching platform's extras (Emus/{platform}/ + Tools/{platform}/ + Bios/) instead of copying all platform folders. This requires:

1. Adding an `extras_platform` field to `DeviceProfile` in `device.ts` (since extras use different folder names than base archives)
2. Plumbing platform through `install_minui` → `try_install_extras` → `copy_extras_files`
3. Rewriting `copy_extras_files` to filter the extras archive by platform folder
