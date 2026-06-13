# MinUI Easy Installer & Package Store

![MinUI Easy Installer](assets/banner.svg)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](#)
[![Twitter: jellydn](https://img.shields.io/twitter/follow/jellydn.svg?style=social)](https://twitter.com/jellydn)

> The easiest way to install and manage MinUI on retro handheld devices.

Think "Balena Etcher for MinUI" — insert an SD card, select your device, click install.

## ✨ Features

- ⚡️ One-click MinUI installation and updates
- 📦 Built-in package store (Wifi.pak, SSH.pak, and more)
- 💾 SD card detection and validation
- 📶 WiFi configuration wizard
- 🎮 Supports: TrimUI Brick, TrimUI Smart Pro, Miyoo Mini+, Miyoo A30, Miyoo Flip, RG35XX Plus, RG35XX H, RG35XX SP

## Status

🚧 Early development. PRD is defined, implementation not yet started.

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
