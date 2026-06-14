# Taste (Continuously Learned by [CommandCode][cmd])

[cmd]: https://commandcode.ai/

# reporting

- Document implementation results and plan execution findings in a `tweak.md` file for post-hoc reporting (not `fix.md`). Confidence: 0.70

# git

- Group changes into atomic commits with conventional commit prefixes (fix:, feat:, docs:) using the `/commit-atomic` workflow. Confidence: 0.65

# package-store

- Store registry is served from `assets/store.json` with `emu_paks` and `tool_paks` arrays; entries can have `device` (array of device IDs, e.g. `["brick"]`) and/or `download_url` (custom artifact URL override). Confidence: 0.70

# wifi-config

- Wifi config file format uses colon-separated `SSID:PASSWORD` entries; SSIDs can contain spaces; lines starting with `#` are comments and ignored. Confidence: 0.70

# store

- Use actual descriptions from `store.json` data instead of auto-generated placeholder descriptions like `${pak.name} tool for MinUI`. Confidence: 0.72

- For package author attribution, use the actual author name with a link to their repo instead of hardcoding "Community"; keep it simple (KISS). Confidence: 0.70

# ui

- Always show formatting/action buttons regardless of detected state; let the user decide rather than hiding based on preconditions. Confidence: 0.65

- For I/O operations that involve disk/network, use async with proper loading states (placeholders/spinners) to avoid freezing the UI. Confidence: 0.85

- Keep UI text concise and to the point; avoid verbose explanations in dialog text. Confidence: 0.75
