import { classifyError } from "./install";
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

export type PackageCategory =
	| "Utilities"
	| "Emulators"
	| "Network"
	| "Community";

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

export type PackageInstallError = {
	message: string;
	code:
		| "DOWNLOAD_ERROR"
		| "EXTRACTION_ERROR"
		| "COPY_ERROR"
		| "CHECKSUM_ERROR"
		| "UNKNOWN_ERROR";
};

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
}

interface StoreToolPak {
	name: string;
	repository: string;
	version: string;
	pak_name: string;
	device?: string[];
	download_url?: string;
	description?: string;
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
	const packages: PackageRegistryEntry[] = [];

	for (const pak of data.emu_paks) {
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
			checksum: null,
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

export async function fetchPackageRegistry(): Promise<PackageRegistryFetchResult> {
	try {
		if (isStoreRegistry(storeData)) {
			return { success: true, data: convertStoreRegistry(storeData) };
		}

		return {
			success: false,
			error: {
				message: "Invalid store data",
				code: "PARSE_ERROR",
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
