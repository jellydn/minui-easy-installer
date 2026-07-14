# List available commands
default:
    @just --list

# Run full Tauri dev (Rust + React)
run: tauri-dev

# Frontend dev server
dev:
    bun run dev

# Build frontend
build:
    bun run build

# Run Tauri dev (Rust + React)
tauri-dev:
    cargo tauri dev

# Run Tauri build
tauri-build:
    cargo tauri build

# Typecheck TypeScript
typecheck:
    bun run typecheck

# Lint TypeScript with oxlint
lint:
    bun run lint

# Format TypeScript with oxfmt
fmt-ts:
    bun run fmt

# Run all checks (lint + typecheck + Rust fmt/clippy)
check:
    bun run lint
    bun run typecheck
    cd src-tauri && cargo fmt --check
    cd src-tauri && cargo clippy -- -D warnings

# Format all code
fmt: fmt-ts
    cd src-tauri && cargo fmt

# Ad-hoc sign the macOS app bundle (fixes Gatekeeper "damaged" error)
sign:
    @APP=$$(ls -d src-tauri/target/release/bundle/macos/*.app 2>/dev/null | head -1); \
    if [ -n "$$APP" ]; then \
        codesign --force --deep --sign - "$$APP" && echo "Signed: $$APP"; \
    else \
        echo "No .app bundle found. Run 'cargo tauri build' first."; \
    fi

# Run pre-commit hooks
pre-commit:
    prek run --all-files
