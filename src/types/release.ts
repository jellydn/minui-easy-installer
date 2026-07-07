import type { ForkConfig } from "./fork";
import { buildReleaseUrl, getForkCacheKey } from "./fork";

export interface MinUIRelease {
  version: string;
  baseArchiveUrl: string;
  extrasArchiveUrl: string | null;
  checksums: ReleaseChecksums | null;
  /** The fork that this release came from. Set by fetchMinUIRelease. */
  fork?: ForkConfig;
}

export interface ReleaseChecksums {
  base: string | null;
  extras: string | null;
}

export interface ReleaseFetchError {
  message: string;
  code: "NETWORK_ERROR" | "PARSE_ERROR" | "NOT_FOUND";
}

export type ReleaseFetchResult =
  | { success: true; data: MinUIRelease }
  | { success: false; error: ReleaseFetchError };

export function parseGitHubRelease(
  data: unknown,
): MinUIRelease | ReleaseFetchError {
  if (!data || typeof data !== "object") {
    return { message: "Invalid release data", code: "PARSE_ERROR" };
  }

  const release = data as Record<string, unknown>;

  if (typeof release.tag_name !== "string") {
    return { message: "Missing tag_name in release", code: "PARSE_ERROR" };
  }

  const version = release.tag_name.replace(/^v/, "");

  const assets = Array.isArray(release.assets) ? release.assets : [];

  function getDownloadUrl(asset: unknown): string | null {
    if (
      typeof asset === "object" &&
      asset !== null &&
      "browser_download_url" in asset
    ) {
      const url = (asset as Record<string, unknown>).browser_download_url;
      if (typeof url === "string") return url;
    }
    return null;
  }

  function findAssetByName(keyword: string): string | null {
    for (const a of assets) {
      const url = getDownloadUrl(a);
      if (url && url.toLowerCase().includes(keyword)) return url;
    }
    return null;
  }

  const baseArchiveUrl = findAssetByName("base");
  if (!baseArchiveUrl) {
    return { message: "No base archive found in release", code: "NOT_FOUND" };
  }

  const extrasArchiveUrl = findAssetByName("extras");

  return {
    version,
    baseArchiveUrl,
    extrasArchiveUrl,
    checksums: null,
  };
}

// Session-scoped cache — keyed by fork, so switching forks invalidates the cache.
const releaseCache = new Map<string, MinUIRelease>();

export function clearReleaseCache(key?: string): void {
  if (key) {
    releaseCache.delete(key);
  } else {
    releaseCache.clear();
  }
}

export async function fetchMinUIRelease(
  fork: ForkConfig,
  fetchFn: typeof globalThis.fetch = globalThis.fetch,
): Promise<ReleaseFetchResult> {
  const cacheKey = getForkCacheKey(fork);
  if (fetchFn === globalThis.fetch) {
    const cached = releaseCache.get(cacheKey);
    if (cached) {
      return { success: true, data: cached };
    }
  }

  const apiUrl = buildReleaseUrl(fork);

  try {
    const response = await fetchFn(apiUrl, {
      headers: { Accept: "application/vnd.github+json" },
    });

    if (!response.ok) {
      if (response.status === 404) {
        return {
          success: false,
          error: { message: "Release not found", code: "NOT_FOUND" },
        };
      }
      return {
        success: false,
        error: {
          message: `GitHub API error: ${response.status}`,
          code: "NETWORK_ERROR",
        },
      };
    }

    const data = await response.json();
    const result = parseGitHubRelease(data);

    if ("code" in result) {
      return { success: false, error: result };
    }

    // Attach the fork that produced this result
    result.fork = fork;

    releaseCache.set(cacheKey, result);
    return { success: true, data: result };
  } catch (err) {
    const message =
      err instanceof Error ? err.message : "Unknown network error";
    return {
      success: false,
      error: { message, code: "NETWORK_ERROR" },
    };
  }
}
