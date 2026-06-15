---
'minui-easy-installer': patch
---

fix(version): restrict raw fallback to strict version grammar

The `parse_minui_version` raw fallback now rejects free-form text (e.g. "Created by MinUI Team 2024") via a new `looks_like_version` helper that accepts only 2-3 dot-separated numeric segments with an optional 'v'/'V' prefix. This prevents lexicographically nonsensical version comparisons and causes `detect_installed_version` to return `None` (the safe default) for unrecognized version strings.
