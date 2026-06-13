# Taste (Continuously Learned by [CommandCode][cmd])

[cmd]: https://commandcode.ai/

# reporting

- Document implementation results and plan execution findings in a `tweak.md` file for post-hoc reporting (not `fix.md`). Confidence: 0.70

# git

- Group changes into atomic commits with conventional commit prefixes (fix:, feat:, docs:) using the `/commit-atomic` workflow. Confidence: 0.65

# package-store

- Use the pakman registry at `https://raw.githubusercontent.com/josegonzalez/pakman/refs/heads/main/paks.json` as the package store source. Confidence: 0.70

# wifi-config

- Wifi config file format uses colon-separated `SSID:PASSWORD` entries; SSIDs can contain spaces; lines starting with `#` are comments and ignored. Confidence: 0.70

# ui

- Always show formatting/action buttons regardless of detected state; let the user decide rather than hiding based on preconditions. Confidence: 0.65

- For I/O operations that involve disk/network, use async with proper loading states (placeholders/spinners) to avoid freezing the UI. Confidence: 0.85

- Keep UI text concise and to the point; avoid verbose explanations in dialog text. Confidence: 0.75
