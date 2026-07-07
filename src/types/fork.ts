export interface ForkConfig {
  label: string;
  owner: string;
  repo: string;
  /**
   * Prefix written into minui.txt on the SD card.
   * TODO(phase-2): Consume this in install.rs when writing version metadata.
   */
  versionPrefix: string;
}

export const FORK_PRESETS: Record<string, ForkConfig> = {
  official: {
    label: "MinUI (Official)",
    owner: "shauninman",
    repo: "MinUI",
    versionPrefix: "MinUI",
  },
  "minui-zero": {
    label: "MinUI-Zero",
    owner: "danklammer",
    repo: "MinUI-Zero",
    versionPrefix: "MinUI-Zero",
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
  if (!GITHUB_OWNER_REPO_RE.test(parts[0]) || !GITHUB_OWNER_REPO_RE.test(parts[1])) {
    return null;
  }

  return {
    label: trimmed,
    owner: parts[0],
    repo: parts[1],
    versionPrefix: parts[1],
  };
}
