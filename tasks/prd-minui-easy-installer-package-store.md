# PRD: MinUI Easy Installer & Package Store

## Introduction

MinUI Easy Installer & Package Store is a cross-platform desktop application that simplifies installing, updating, and extending MinUI on supported retro handheld devices. The product should feel like “Balena Etcher for MinUI”: insert an SD card, select a device, click install, and finish with a validated MinUI setup.

The app targets beginners and handheld owners who want MinUI without manually downloading releases, extracting ZIP files, choosing platform folders, copying PAKs, or editing WiFi files.

## Goals

- Enable a new user to install MinUI on a supported SD card in under 60 seconds after downloads complete.
- Support Windows and macOS in the MVP.
- Support one-click install and update flows for MinUI Base and Extras.
- Provide a built-in package store backed by a static GitHub-hosted registry.
- Install and configure essential packages such as Wifi.pak and SSH.pak without manual folder copying.
- Detect common SD card and installation problems before and after installation.

## Target Users

- Retro handheld beginners.
- TrimUI Brick users.
- TrimUI Smart Pro users.
- Miyoo Mini Plus users.
- Miyoo A30 users.
- Miyoo Flip users.
- RG35XX Plus users.
- RG35XX H users.
- RG35XX SP users.
- Users who want MinUI without manual SD card operations.

## User Stories

### US-001: Detect removable SD card

**Description:** As a beginner, I want the app to detect my inserted SD card so that I do not need to locate the drive manually.

**Acceptance Criteria:**

- [ ] App lists removable storage devices with name, mount path, size, filesystem, and available space.
- [ ] App identifies whether each drive is FAT32 when the operating system exposes that information.
- [ ] App warns before writing to any selected drive.
- [ ] App does not write to a drive until the user explicitly confirms install or update.
- [ ] Typecheck, lint, and relevant Rust checks pass.

### US-002: Select target handheld device

**Description:** As a user, I want to select my handheld model so that the installer uses the correct MinUI platform files.

**Acceptance Criteria:**

- [ ] Device selector includes TrimUI Brick, TrimUI Smart Pro, Miyoo Mini+, Miyoo A30, Miyoo Flip, RG35XX Plus, RG35XX H, and RG35XX SP.
- [ ] Each supported device maps to the correct platform folder or installation profile.
- [ ] Unsupported devices are not shown as install targets in MVP.
- [ ] Selected device persists for the current session.
- [ ] Verify UI in browser/app preview.

### US-003: Install latest MinUI release

**Description:** As a user, I want to install the latest MinUI release with one click so that I do not need to download and extract files manually.

**Acceptance Criteria:**

- [ ] App fetches latest MinUI release metadata from the configured source.
- [ ] App downloads Base and Extras archives for the selected device when required.
- [ ] App verifies downloaded archive checksums when checksum data is available.
- [ ] App extracts archives to a temporary working directory.
- [ ] App copies the correct Base and Extras files to the selected SD card.
- [ ] App shows progress for download, extract, copy, and validation steps.
- [ ] App shows a success screen after validation passes.
- [ ] Failed installs show a recoverable error message and do not report success.

### US-004: Validate installation

**Description:** As a user, I want the app to validate the SD card after installation so that I know MinUI was installed correctly.

**Acceptance Criteria:**

- [ ] App checks expected MinUI folders and files for the selected device.
- [ ] App checks that essential PAK files from Extras were installed when selected.
- [ ] App checks available free space after installation.
- [ ] App reports missing or corrupted expected files.
- [ ] App generates a readable installation report.

### US-005: Update installed MinUI

**Description:** As an existing MinUI user, I want the app to detect and install MinUI updates so that I do not need to replace files manually.

**Acceptance Criteria:**

- [ ] App detects the installed MinUI version when version metadata is available on the SD card.
- [ ] App compares installed version with latest release metadata.
- [ ] App shows an update prompt when a newer version exists.
- [ ] App updates MinUI files without deleting user ROMs or saves.
- [ ] App shows update completion and validation results.

### US-006: Browse built-in package store

**Description:** As a user, I want to browse available packages so that I can extend MinUI without searching GitHub manually.

**Acceptance Criteria:**

- [ ] Store loads package metadata from a static registry JSON URL.
- [ ] Store supports categories: Utilities, Emulators, Network, and Community.
- [ ] Package cards show name, version, author, category, description, download count, and rating when present.
- [ ] Store supports search by package name and description.
- [ ] Store handles registry fetch failures with a clear retry state.
- [ ] Verify UI in browser/app preview.

### US-007: Install Wifi.pak

**Description:** As a user, I want to install Wifi.pak from the app so that I do not need to copy files into Tools folders manually.

**Acceptance Criteria:**

- [ ] Wifi.pak appears in the package store under Network or Utilities.
- [ ] Clicking Install downloads the configured package artifact.
- [ ] App verifies package checksum when available.
- [ ] App copies Wifi.pak into the correct Tools folder for the selected device profile.
- [ ] App marks Wifi.pak as installed after validation.

### US-008: Install SSH.pak

**Description:** As a user, I want to install SSH.pak from the app so that I can enable remote access tools more easily.

**Acceptance Criteria:**

- [ ] SSH.pak appears in the package store.
- [ ] Clicking Install downloads and installs SSH.pak to the correct folder.
- [ ] App verifies package checksum when available.
- [ ] App marks SSH.pak as installed after validation.

### US-009: Configure WiFi credentials

**Description:** As a user, I want a WiFi wizard to scan networks and save credentials so that I do not need to manually create wifi.txt.

**Acceptance Criteria:**

- [ ] App can install Wifi.pak before starting WiFi setup.
- [ ] App can list nearby WiFi networks when the host operating system allows scanning.
- [ ] User can manually enter SSID when scanning is unavailable.
- [ ] User can enter password securely without displaying it by default.
- [ ] App generates wifi.txt in the correct location and format for Wifi.pak.
- [ ] App can run a connection test when supported by the target package/device workflow.

### US-010: Update installed packages

**Description:** As a user, I want the app to detect package updates so that installed tools stay current.

**Acceptance Criteria:**

- [ ] App detects installed package versions when package metadata exists on the SD card.
- [ ] App compares installed versions with registry versions.
- [ ] Home screen lists available package updates.
- [ ] “Update All” updates MinUI and packages in a safe order.
- [ ] Updating packages does not delete user configuration unless the package explicitly requires migration.

### US-011: Check SD card health

**Description:** As a user, I want the app to check my SD card for common problems so that I can fix issues before booting my handheld.

**Acceptance Criteria:**

- [ ] App checks filesystem format and warns when the card is not FAT32.
- [ ] App checks available space.
- [ ] App checks missing BIOS folders or files where supported by metadata.
- [ ] App checks missing PAK files for installed packages.
- [ ] App detects missing or unexpected core MinUI folders.
- [ ] App generates a report that can be copied for support.

### US-012: Manage package registry

**Description:** As a maintainer, I want a static package registry so that packages can be distributed without operating a backend service.

**Acceptance Criteria:**

- [ ] Registry is a static JSON file served from a stable URL such as `https://packages.minui.dev/registry/index.json`.
- [ ] Registry entries include name, version, author, category, description, downloads, rating, artifact URL, checksum, supported devices, and install path rules.
- [ ] App validates registry schema before displaying packages.
- [ ] Invalid package entries are skipped and logged without breaking the whole store.

## Functional Requirements

- FR-1: The app must be a desktop application built with Tauri v2, a Rust backend, and a React frontend unless a later architecture decision replaces this stack.
- FR-2: The MVP must support Windows and macOS.
- FR-3: The app must detect removable drives and show enough information for the user to choose the correct SD card.
- FR-4: The app must require explicit user confirmation before writing to an SD card.
- FR-5: The app must support device profiles for TrimUI Brick, TrimUI Smart Pro, Miyoo Mini+, Miyoo A30, Miyoo Flip, RG35XX Plus, RG35XX H, and RG35XX SP.
- FR-6: The app must download the latest MinUI Base release from the configured release source.
- FR-7: The app must download MinUI Extras when required for selected packages or install profile.
- FR-8: The app must verify checksums for downloads when checksum metadata is available.
- FR-9: The app must extract archives safely into a temporary directory before copying files to the SD card.
- FR-10: The app must copy only files required by the selected device profile.
- FR-11: The app must validate installation after install and update operations.
- FR-12: The app must display install and update progress with clear current step labels.
- FR-13: The app must detect installed MinUI version when possible.
- FR-14: The app must detect available MinUI updates by comparing local and remote version metadata.
- FR-15: The app must preserve user ROMs, saves, and user configuration during MinUI updates.
- FR-16: The app must load package metadata from a static GitHub-backed registry.
- FR-17: The app must support package categories for Utilities, Emulators, Network, and Community.
- FR-18: The app must search packages by name and description.
- FR-19: The app must install packages according to registry-defined install rules.
- FR-20: The app must support installing Wifi.pak and SSH.pak in the MVP.
- FR-21: The app must generate Wifi.pak-compatible `wifi.txt` from user-provided credentials.
- FR-22: The app must never log WiFi passwords or other secrets in plaintext.
- FR-23: The app must check SD card format, available space, expected folders, and expected package files.
- FR-24: The app must generate a copyable health or installation report.
- FR-25: The app must provide a Home screen showing detected device, SD card, installed MinUI version, and available updates.
- FR-26: The app must provide a Store screen with package search, category filtering, package details, and install actions.

## Non-Goals

- No Linux desktop support in Phase 1.
- No mobile companion app in Phase 1 or Phase 2.
- No hosted backend service for the package registry in MVP.
- No paid marketplace, accounts, or payment processing.
- No package publishing UI in MVP.
- No community package ratings submission flow in Phase 1.
- No OTA update mechanism running directly on handhelds in Phase 1 or Phase 2.
- No cloud save sync in Phase 1 or Phase 2.
- No destructive SD card formatting in MVP unless added behind a separate explicit PRD.

## Design Considerations

### Product Positioning

Tagline options:

- “Install MinUI in 60 seconds.”
- “The easiest way to manage MinUI.”
- “App Store for MinUI handhelds.”

### Home Screen

The Home screen should summarize the current state:

```text
Detected Device:
TrimUI Brick

SD Card:
128GB FAT32

Installed:
MinUI 20251127-1

Updates:
2 available

[Install Package]
[Update All]
```

### Store Screen

The Store should support a simple search-first flow:

```text
Search: wifi

Results:
- Wifi
- SSH
- LED Manager
- Syncthing

[Install]
```

### UX Principles

- Use beginner-friendly language and avoid assuming SD card knowledge.
- Always show which drive will be modified before writing.
- Prefer guided flows over exposing raw folder paths.
- Show recoverable errors with clear next steps.
- Keep advanced diagnostics available but not required for normal use.

## Technical Considerations

### Desktop App

Preferred stack:

- Tauri v2.
- Rust backend for filesystem, drive detection, archive extraction, checksum verification, and package installation.
- React frontend for guided installer, Home, Store, WiFi wizard, and reports.

Reasons:

- Small binary size.
- Cross-platform support.
- Fast startup.
- Strong access to native filesystem operations through Rust.

### Package Registry

The package registry should be static JSON hosted on GitHub Pages or equivalent static hosting.

Example structure:

```text
https://packages.minui.dev/registry/index.json
https://packages.minui.dev/packages/wifi/
https://packages.minui.dev/packages/ssh/
https://packages.minui.dev/packages/bootlogo/
```

Example package metadata:

```json
{
  "name": "Wifi",
  "version": "1.2.0",
  "author": "josegonzalez",
  "category": "network",
  "description": "Manage WiFi connections",
  "downloads": 12345,
  "rating": 4.8
}
```

MVP registry entries should also include artifact URLs, checksums, supported devices, and install rules even if those fields are not shown on package cards.

### Installer Engine

The installer engine is responsible for:

- Detecting removable drives.
- Resolving selected device profiles.
- Downloading releases and packages.
- Verifying checksums.
- Extracting archives safely.
- Copying files to the selected SD card.
- Installing and updating packages.
- Validating final SD card state.
- Producing user-readable reports.

### Security and Safety

- Require explicit user confirmation before writing to an SD card.
- Never format drives in MVP.
- Avoid logging secrets such as WiFi passwords.
- Verify checksums when available.
- Do not execute package code on the host computer during installation.
- Treat package registry data as untrusted input and validate schema before use.

## Release Phases

### Phase 1: MVP

- Install MinUI.
- Update MinUI.
- Install Wifi.pak.
- Install SSH.pak.
- Static package registry.
- Windows support.
- macOS support.

### Phase 2

- Linux support.
- Package ratings display and submission model.
- Community packages.
- ROM collection manager.

### Phase 3

- OTA updates.
- Mobile companion app.
- RetroAchievements setup.
- Cloud save sync.

## Optional Future Scope: ROM Collection Manager

The ROM Collection Manager is not part of the MVP unless explicitly pulled into Phase 2. Potential features:

- Drag and drop ROMs onto the app.
- Auto-sort ROMs into correct platform folders.
- Rename ROMs using common naming rules.
- Detect duplicate ROM files.
- Report unsupported extensions.

## Success Metrics

### Month 1

- 500 app installs.
- 50 package installs per day.
- Median successful MinUI install flow completed without support documentation.

### Month 3

- 2,000 app installs.
- 20 community packages available in the registry.
- At least 80% of install attempts that pass SD card preflight complete successfully.

### Month 6

- Become the default recommended onboarding tool for MinUI beginners.
- Maintain low support burden by producing useful install and health reports.

## Open Source Strategy

- License: MIT.
- Repository: `jellydn/minui-store`.
- Registry: GitHub-backed static registry.
- Encourage community packages through pull requests to the registry repository.
- Keep package metadata human-reviewable and easy to validate in CI.

## Open Questions

1. What is the authoritative source for latest MinUI release metadata and checksums?
2. What exact platform folder mapping should each supported device use?
3. What is the canonical Wifi.pak `wifi.txt` location and file format for each supported device?
4. Should the MVP allow choosing whether to install Extras, or always install Extras by default?
5. How should installed package versions be recorded if existing PAKs do not include metadata?
6. Should package downloads be mirrored under `packages.minui.dev`, or should registry entries point directly to GitHub release assets?
7. What minimum Windows and macOS versions should be supported?
