import type { AppError } from "./errors";
import { classifyError } from "./errors";
import storeData from "./store.json";
import { parseRegistryFromJson } from "./registry-convert";

export interface PackageRegistryEntry {
  name: string;
  version: string;
  category: PackageCategory;
  description: string;
  repository: string;
  downloads: number | null;
  rating: number | null;
  artifactUrl: string;
  checksum: string | null;
  supportedDevices: string[];
  installPathRules: PackageInstallPathRules;
}

export type PackageCategory = "Utilities" | "Emulators";

export interface PackageInstallPathRules {
  targetDir: string;
  extractToRoot: boolean;
  pakName?: string;
}

export interface PackageRegistry {
  version: string;
  packages: PackageRegistryEntry[];
}

export interface PackageRegistryError {
  message: string;
  code:
    | "INVALID_ENTRY"
    | "VALIDATION_ERROR"
    | "PARSE_ERROR"
    | "NETWORK_ERROR"
    | "NOT_FOUND";
}

const REGISTRY_URL = "https://packages.minui.dev/registry/index.json";

/// How long the registry cache is valid before a re-fetch (5 minutes).
const CACHE_TTL_MS = 5 * 60_000;

/// Encapsulates the session-scoped package registry cache.
/// Each instance owns its cached data and TTL independently —
/// the module-level singleton is the only instance used in practice,
/// but the class keeps the cache logic testable and explicit.
export class RegistryCache {
  private registry: PackageRegistry | null = null;
  private fetchedAt: number = 0;
  private readonly ttlMs: number;

  constructor(ttlMs: number = CACHE_TTL_MS) {
    this.ttlMs = ttlMs;
  }

  /// Returns the cached registry if still valid, otherwise null.
  get(): PackageRegistry | null {
    if (this.registry && Date.now() - this.fetchedAt < this.ttlMs) {
      return this.registry;
    }
    return null;
  }

  /// Store a freshly fetched registry with a new timestamp.
  set(registry: PackageRegistry): void {
    this.registry = registry;
    this.fetchedAt = Date.now();
  }

  /// Invalidate the cache so the next fetch will re-request.
  clear(): void {
    this.registry = null;
    this.fetchedAt = 0;
  }
}

const registryCache = new RegistryCache();

export function clearRegistryCache(): void {
  registryCache.clear();
}

export async function installPackage(options: {
  artifactUrl: string;
  checksum?: string;
  sdMount: string;
  targetDir: string;
  extractToRoot: boolean;
  pakName?: string;
  platform: string;
}): Promise<PackageInstallResultEither> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<PackageInstallResult>("install_package", {
      opts: {
        artifactUrl: options.artifactUrl,
        checksum: options.checksum || null,
        sdMount: options.sdMount,
        targetDir: options.targetDir,
        extractToRoot: options.extractToRoot,
        pakName: options.pakName,
        platform: options.platform,
      },
    });

    if (result.success) {
      return { success: true, data: result };
    }

    const errorMsg = result.error || "Package installation failed";
    const code = classifyError(errorMsg);

    return {
      success: false,
      error: { message: errorMsg, code },
    };
  } catch (err) {
    const message = err instanceof Error ? err.message : "Unknown error";
    return {
      success: false,
      error: { message, code: "UNKNOWN_ERROR" },
    };
  }
}

export async function detectInstalledPackages(
  sdMount: string,
): Promise<InstalledPackage[]> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<InstalledPackage[]>("detect_installed_packages", {
      sdMount,
    });
  } catch {
    return [];
  }
}

export async function checkPackageUpdates(
  sdMount: string,
  registryPackages: [string, string][],
): Promise<PackageUpdateInfo[]> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<PackageUpdateInfo[]>("check_package_updates", {
      opts: {
        sdMount,
        registryPackages,
      },
    });
  } catch {
    return [];
  }
}

export type PackageRegistryFetchResult =
  | { success: true; data: PackageRegistry }
  | { success: false; error: PackageRegistryError };

export interface PackageInstallResult {
  success: boolean;
  error: string | null;
  files_copied: number;
}

export type PackageInstallError = AppError;

export type PackageInstallResultEither =
  | { success: true; data: PackageInstallResult }
  | { success: false; error: PackageInstallError };

export interface InstalledPackage {
  name: string;
  version: string | null;
  source: string;
}

export interface PackageUpdateInfo {
  name: string;
  installed_version: string | null;
  latest_version: string;
  update_available: boolean;
}

/// Registry conversion and validation lives in registry-convert.ts.
/// See: parseRegistryFromJson, convertStoreRegistry, isStoreRegistry, StoreRegistry.

export async function fetchPackageRegistry(): Promise<PackageRegistryFetchResult> {
  // Return cache if still valid
  const cached = registryCache.get();
  if (cached) {
    return { success: true, data: cached };
  }

  // Try fetching remote registry via Tauri backend
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const text = await invoke<string>("fetch_url", { url: REGISTRY_URL });
    const json = JSON.parse(text);
    const result = parseRegistryFromJson(json);

    if (result.success) {
      registryCache.set(result.data);
      return result;
    }
  } catch {
    // Remote fetch failed — fall through to bundled store
  }

  // Fall back to bundled store.json
  try {
    const result = parseRegistryFromJson(storeData);
    if (result.success) {
      return result;
    }

    return {
      success: false,
      error: {
        message: `Failed to load package data: ${result.error.message}`,
        code: "NETWORK_ERROR",
      },
    };
  } catch (err) {
    const message = err instanceof Error ? err.message : "Unknown error";
    return {
      success: false,
      error: { message, code: "NETWORK_ERROR" },
    };
  }
}
