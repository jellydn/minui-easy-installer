import type { AppError } from "./errors";
import { classifyError } from "./errors";
import storeData from "./store.json";

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
class RegistryCache {
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
      artifactUrl: options.artifactUrl,
      checksum: options.checksum || null,
      sdMount: options.sdMount,
      targetDir: options.targetDir,
      extractToRoot: options.extractToRoot,
      pakName: options.pakName,
      platform: options.platform,
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
      sdMount,
      registryPackages,
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

function resolveArtifactUrl(
  repository: string,
  version: string,
  pakName: string,
): string {
  const repo = repository.replace("https://github.com/", "");
  const fileName = `${pakName.replace(/\s+/g, ".")}.pak.zip`;
  return `https://github.com/${repo}/releases/download/${version}/${fileName}`;
}

interface StoreEmuPak {
  name: string;
  repository: string;
  version: string;
  pak_name: string;
  rom_folder: string;
  description?: string;
  checksum?: string;
}

interface StoreToolPak {
  name: string;
  repository: string;
  version: string;
  pak_name: string;
  device?: string[];
  download_url?: string;
  description?: string;
  checksum?: string;
}

interface StoreRegistry {
  emu_paks: StoreEmuPak[];
  tool_paks: StoreToolPak[];
}

function isStoreRegistry(data: unknown): data is StoreRegistry {
  if (!data || typeof data !== "object") return false;
  const d = data as Record<string, unknown>;
  return Array.isArray(d.emu_paks) && Array.isArray(d.tool_paks);
}

interface ValidationError {
  entryName: string;
  field: string;
  reason: string;
}

function validateStoreEntry(
  pak: StoreEmuPak | StoreToolPak,
  type: "emu" | "tool",
): ValidationError | null {
  if (
    !pak.name ||
    typeof pak.name !== "string" ||
    pak.name.trim().length === 0
  ) {
    return {
      entryName: "(unnamed)",
      field: "name",
      reason: "missing or empty",
    };
  }
  if (!pak.repository || typeof pak.repository !== "string") {
    return {
      entryName: pak.name,
      field: "repository",
      reason: "missing or not a string",
    };
  }
  if (!pak.repository.startsWith("https://github.com/")) {
    return {
      entryName: pak.name,
      field: "repository",
      reason: "must be https://github.com/...",
    };
  }
  if (
    !pak.version ||
    typeof pak.version !== "string" ||
    pak.version.trim().length === 0
  ) {
    return {
      entryName: pak.name,
      field: "version",
      reason: "missing or empty",
    };
  }
  if (
    !pak.pak_name ||
    typeof pak.pak_name !== "string" ||
    pak.pak_name.trim().length === 0
  ) {
    return {
      entryName: pak.name,
      field: "pak_name",
      reason: "missing or empty",
    };
  }
  if (type === "emu") {
    const emu = pak as StoreEmuPak;
    if (!emu.rom_folder || typeof emu.rom_folder !== "string") {
      return {
        entryName: pak.name,
        field: "rom_folder",
        reason: "missing or not a string",
      };
    }
  }
  if (pak.checksum !== undefined) {
    if (typeof pak.checksum !== "string" || pak.checksum.length !== 64) {
      return {
        entryName: pak.name,
        field: "checksum",
        reason: "must be a 64-character hex string",
      };
    }
  }
  return null;
}

function resolveDownloadUrl(
  repository: string,
  version: string,
  pak_name: string,
  download_url?: string,
): string {
  if (download_url) return download_url;
  return resolveArtifactUrl(repository, version, pak_name);
}

function convertStoreRegistry(data: StoreRegistry): PackageRegistry {
  const allErrors: string[] = [];
  const packages: PackageRegistryEntry[] = [];

  for (const pak of data.emu_paks) {
    const err = validateStoreEntry(pak, "emu");
    if (err) {
      allErrors.push(`[emu_paks] ${err.entryName}.${err.field}: ${err.reason}`);
      continue;
    }
    packages.push({
      name: pak.name,
      version: pak.version,
      category: "Emulators",
      description:
        pak.description ||
        `Emulates ${pak.rom_folder.replace("Roms/", "")} games`,
      repository: pak.repository,
      downloads: null,
      rating: null,
      artifactUrl: resolveDownloadUrl(
        pak.repository,
        pak.version,
        pak.pak_name,
      ),
      checksum: null,
      supportedDevices: [],
      installPathRules: {
        targetDir: "/Emus",
        extractToRoot: false,
        pakName: pak.pak_name,
      },
    });
  }

  for (const pak of data.tool_paks) {
    const err = validateStoreEntry(pak, "tool");
    if (err) {
      allErrors.push(
        `[tool_paks] ${err.entryName}.${err.field}: ${err.reason}`,
      );
      continue;
    }
    packages.push({
      name: pak.name,
      version: pak.version,
      category: "Utilities",
      description: pak.description || `${pak.name} utility`,
      repository: pak.repository,
      downloads: null,
      rating: null,
      artifactUrl: resolveDownloadUrl(
        pak.repository,
        pak.version,
        pak.pak_name,
        pak.download_url,
      ),
      checksum: pak.checksum || null,
      supportedDevices: pak.device || [],
      installPathRules: {
        targetDir: "/Tools",
        extractToRoot: false,
        pakName: pak.pak_name,
      },
    });
  }

  return { version: "1.0.0", packages };
}

function parseRegistryFromJson(data: unknown): PackageRegistryFetchResult {
  if (!isStoreRegistry(data)) {
    return {
      success: false,
      error: {
        message: "Invalid store format: missing emu_paks/tool_paks arrays",
        code: "PARSE_ERROR",
      },
    };
  }

  const registry = convertStoreRegistry(data);

  if (registry.packages.length === 0) {
    return {
      success: false,
      error: {
        message: "Registry has no valid entries",
        code: "VALIDATION_ERROR",
      },
    };
  }

  return { success: true, data: registry };
}

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
