/**
 * Shared utilities for registry scripts (update-registry, discover-packages).
 */

/** Package entry stored in store.json. */
export interface StorePak {
  name: string;
  repository: string;
  version: string;
  pak_name: string;
  rom_folder?: string;
  download_url?: string;
  device?: string[];
  description?: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any;
}

export interface StoreRegistry {
  emu_paks: StorePak[];
  tool_paks: StorePak[];
}

/** Extract owner/repo from a GitHub URL. Handles trailing slashes and .git suffix. */
export function repoSlug(repository: string): { owner: string; repo: string } | null {
  const match = repository.match(/^https:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\.git)?\/?$/);
  if (!match) return null;
  return { owner: match[1], repo: match[2] };
}

/** Strip a leading 'v' from a version tag (e.g. "v1.2.3" → "1.2.3"). */
export function stripLeadingV(tag: string): string {
  return tag.startsWith("v") ? tag.slice(1) : tag;
}

/** Build GitHub API request headers. Includes auth if GITHUB_TOKEN is set. */
export function apiHeaders(): Record<string, string> {
  const headers: Record<string, string> = {
    "User-Agent": "minui-easy-installer-registry",
  };
  if (process.env.GITHUB_TOKEN) {
    headers["Authorization"] = `Bearer ${process.env.GITHUB_TOKEN}`;
  }
  return headers;
}

/** Fetch a URL with error handling. Returns the Response or null on failure. */
export async function fetchApi(url: string): Promise<Response | null> {
  try {
    return await fetch(url, { headers: apiHeaders() });
  } catch {
    console.warn(`  ⚠ Network error fetching ${url}`);
    return null;
  }
}

/** Polite delay between API requests (200ms). */
export function apiDelay(): Promise<void> {
  return new Promise((r) => setTimeout(r, 200));
}
