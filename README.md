# MinUI Easy Installer & Package Store

![MinUI Easy Installer](assets/banner.svg)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](#)
[![Twitter: jellydn](https://img.shields.io/twitter/follow/jellydn.svg?style=social)](https://twitter.com/jellydn)

> The easiest way to install and manage MinUI on retro handheld devices.

## ✨ Features

- ⚡️ One-click MinUI installation and updates with real-time progress log
- 📦 Built-in package store with per-device platform paths (e.g., `/Emus/tg5040/DC.pak/`)
- 💾 SD card detection and validation with health checks
- 📶 WiFi configuration wizard
- 🧬 BIOS file installer for emulators that need copyrighted BIOS files (you supply the BIOS files)
- 🎮 Supports TrimUI Brick/Smart Pro, Miyoo Mini+/A30/Flip, RG35XX Plus/H/SP, and more

## Download

The latest release is **[v0.1.2](https://github.com/jellydn/minui-easy-installer/releases/tag/v0.1.2)**.

| Platform | Download |
|----------|----------|
| **macOS** (Apple Silicon) | [MinUI.Easy.Installer_0.1.2_aarch64.dmg](https://github.com/jellydn/minui-easy-installer/releases/download/v0.1.2/MinUI.Easy.Installer_0.1.2_aarch64.dmg) |
| **Windows** (x64) | [MinUI.Easy.Installer_0.1.2_x64-setup.exe](https://github.com/jellydn/minui-easy-installer/releases/download/v0.1.2/MinUI.Easy.Installer_0.1.2_x64-setup.exe) |
| **Windows** (x64 MSI) | [MinUI.Easy.Installer_0.1.2_x64_en-US.msi](https://github.com/jellydn/minui-easy-installer/releases/download/v0.1.2/MinUI.Easy.Installer_0.1.2_x64_en-US.msi) |

📦 [All releases](https://github.com/jellydn/minui-easy-installer/releases)

> **macOS users:** After downloading the DMG and dragging the app to Applications, [right-click the app and select Open](https://support.apple.com/guide/mac-help/open-a-mac-app-from-an-unidentified-developer-mh40616/mac) on first launch. If you see "is damaged and can't be opened", run `xattr -cr /Applications/MinUI\ Easy\ Installer.app` in Terminal.

## Status

✅ Active development — install, update, package store, and WiFi flows are working.

## Tech Stack

- [Tauri v2](https://v2.tauri.app/) — Rust backend + React frontend
- [oxlint](https://oxc-project.github.io/) + [oxfmt](https://oxc-project.github.io/) — Fast Rust-based linting and formatting
- [Bun](https://bun.sh/) — JavaScript runtime & package manager
- Windows + macOS (MVP)

## Install

```sh
bun install
```

## Usage

### Development

```sh
bun run dev
```

### Full Tauri dev (Rust + React)

```sh
cargo tauri dev
```

### Build

```sh
cargo tauri build
```

## Pre-commit

This project uses [prek](https://prek.j178.dev/) to enforce code quality. To install pre-commit hooks:

```sh
prek install
```

## Commands

Run `just` to see all available commands:

```sh
just              # List all commands
just check        # Run all checks (lint + typecheck + Rust fmt/clippy)
just fmt          # Format all code
just lint         # Lint with oxlint
just tauri-dev    # Run Tauri dev
just sign         # Ad-hoc sign macOS .app bundle (fixes Gatekeeper)
```

## Project Structure

```
.
├── assets/             # Brand assets and images
├── tasks/              # PRDs and task definitions
│   └── prd-minui-easy-installer-package-store.md
├── scripts/
│   └── ralph/          # Autonomous coding agent loop
│       ├── ralph.sh    # Run autonomous iterations
│       └── prd.json    # PRD in Ralph format
├── src/                # React frontend
├── src-tauri/          # Rust backend
└── AGENTS.md           # Agent instructions
```

## Contributing

See [PRD](tasks/prd-minui-easy-installer-package-store.md) for full requirements and user stories.

## Author

👤 **Dung Huynh**

- Website: https://productsway.com/
- Twitter: [@jellydn](https://twitter.com/jellydn)
- Github: [@jellydn](https://github.com/jellydn)

## Show your support

[![kofi](https://img.shields.io/badge/Ko--fi-F16061?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/dunghd)
[![paypal](https://img.shields.io/badge/PayPal-00457C?style=for-the-badge&logo=paypal&logoColor=white)](https://paypal.me/dunghd)
[![buymeacoffee](https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black)](https://www.buymeacoffee.com/dunghd)

Give a ⭐️ if this project helped you!

## License

MIT
