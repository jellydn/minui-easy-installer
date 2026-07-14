import type {
  PackageRegistry,
  PackageRegistryEntry,
  PackageRegistryFetchResult,
} from "./package";

/// Length of a SHA-256 hex digest string.
const SHA256_HEX_LENGTH = 64;

// ---- Store JSON format interfaces ----

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

export interface StoreRegistry {
  emu_paks: StoreEmuPak[];
  tool_paks: StoreToolPak[];
}

// ---- Validation ----

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
    if (
      typeof pak.checksum !== "string" ||
      pak.checksum.length !== SHA256_HEX_LENGTH
    ) {
      return {
        entryName: pak.name,
        field: "checksum",
        reason: "must be a 64-character hex string",
      };
    }
  }
  return null;
}

// ---- URL resolution ----

function resolveArtifactUrl(
  repository: string,
  version: string,
  pakName: string,
): string {
  const repo = repository.replace("https://github.com/", "");
  const fileName = `${pakName.replace(/\s+/g, ".")}.pak.zip`;
  return `https://github.com/${repo}/releases/download/${version}/${fileName}`;
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

// ---- Conversion ----

export function convertStoreRegistry(data: StoreRegistry): PackageRegistry {
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

// ---- JSON parsing ----

export function isStoreRegistry(data: unknown): data is StoreRegistry {
  if (!data || typeof data !== "object") return false;
  const d = data as Record<string, unknown>;
  return Array.isArray(d.emu_paks) && Array.isArray(d.tool_paks);
}

export function parseRegistryFromJson(
  data: unknown,
): PackageRegistryFetchResult {
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
