export interface ForkConfig {
  label: string;
  owner: string;
  repo: string;
  /**
   * String written into minui.txt on the SD card and matched against
   * when reading minui.txt back. Single source of truth for both the
   * write side (installer) and the read side (version detection).
   */
  minuiTxtPrefix: string;
}

export const FORK_PRESETS: Record<string, ForkConfig> = {
  official: {
    label: "MinUI (Official)",
    owner: "shauninman",
    repo: "MinUI",
    minuiTxtPrefix: "MinUI",
  },
  "minui-zero": {
    label: "MinUI-Zero",
    owner: "danklammer",
    repo: "MinUI-Zero",
    minuiTxtPrefix: "MinUI-Zero",
  },
};

/** Build the GitHub API releases URL for a given fork. */
export function buildReleaseUrl(fork: ForkConfig): string {
  return `https://api.github.com/repos/${fork.owner}/${fork.repo}/releases/latest`;
}

/**
 * Opaque cache key that scopes release cache per fork.
 * The slash in "owner/repo" is safe here — it's a Map key, not a
 * filesystem path or URL component, so no traversal risk.
 */
export function getForkCacheKey(fork: ForkConfig): string {
  return `${fork.owner}/${fork.repo}`;
}

/** GitHub owner/repo pattern: alphanumerics, hyphens, dots, underscores. */
const GITHUB_OWNER_REPO_RE = /^[\w.-]+$/;

/** Build a custom ForkConfig from a raw owner/repo string. */
export function buildCustomFork(raw: string): ForkConfig | null {
  const trimmed = raw.trim();
  if (!trimmed) return null;

  const parts = trimmed.split("/");
  if (parts.length !== 2 || !parts[0] || !parts[1]) return null;
  if (
    !GITHUB_OWNER_REPO_RE.test(parts[0]) ||
    !GITHUB_OWNER_REPO_RE.test(parts[1])
  ) {
    return null;
  }

  return {
    label: trimmed,
    owner: parts[0],
    repo: parts[1],
    minuiTxtPrefix: parts[1],
  };
}

/**
 * Rehydrate a ForkConfig from a raw stored value (e.g. from localStorage).
 * Returns the matching preset when possible, or the custom fork as-is.
 */
export function rehydrateFork(stored: unknown): ForkConfig | null {
  if (!stored || typeof stored !== "object") return null;
  const parsed = stored as Record<string, unknown>;
  const owner = typeof parsed.owner === "string" ? parsed.owner : "";
  const repo = typeof parsed.repo === "string" ? parsed.repo : "";
  if (!owner || !repo) return null;

  // Prefer preset lookup for consistency
  for (const preset of Object.values(FORK_PRESETS)) {
    if (preset.owner === owner && preset.repo === repo) {
      return preset;
    }
  }

  // Custom fork not in presets
  return buildCustomFork(`${owner}/${repo}`);
}
