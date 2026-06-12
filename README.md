# MinUI Easy Installer & Package Store

The easiest way to install and manage MinUI on retro handheld devices. Think "Balena Etcher for MinUI" — insert an SD card, select your device, click install.

## Features

- One-click MinUI installation and updates
- Built-in package store (Wifi.pak, SSH.pak, and more)
- SD card detection and validation
- WiFi configuration wizard
- Supports: TrimUI Brick, TrimUI Smart Pro, Miyoo Mini+, Miyoo A30, Miyoo Flip, RG35XX Plus, RG35XX H, RG35XX SP

## Status

Early development. PRD is defined, implementation not yet started.

## Tech Stack

- [Tauri v2](https://v2.tauri.app/) — Rust backend + React frontend
- Windows + macOS (MVP)

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/)

### Development

```bash
npm install
cargo tauri dev
```

### Build

```bash
cargo tauri build
```

## Project Structure

```
.
├── tasks/              # PRDs and task definitions
│   └── prd-minui-easy-installer-package-store.md
├── scripts/
│   └── ralph/          # Autonomous coding agent loop
│       ├── ralph.sh    # Run autonomous iterations
│       └── prd.json    # PRD in Ralph format
└── AGENTS.md           # Agent instructions
```

## Contributing

See [PRD](tasks/prd-minui-easy-installer-package-store.md) for full requirements and user stories.

## License

MIT
