export interface PackageRegistryEntry {
	name: string;
	version: string;
	author: string;
	category: PackageCategory;
	description: string;
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
}

export interface PackageRegistry {
	version: string;
	packages: PackageRegistryEntry[];
}

export const REGISTRY_URL = "https://packages.minui.dev/registry/index.json";

export interface PackageRegistryError {
	message: string;
	code:
		| "INVALID_ENTRY"
		| "VALIDATION_ERROR"
		| "PARSE_ERROR"
		| "NETWORK_ERROR"
		| "NOT_FOUND";
}

export interface PackageRegistryValidationResult {
	valid: boolean;
	errors: PackageRegistryError[];
	warnings: string[];
}

const VALID_CATEGORIES: PackageCategory[] = [
	"Utilities",
	"Emulators",
	"Network",
	"Community",
];

export function validatePackageEntry(
	entry: unknown,
	index: number,
): PackageRegistryError[] {
	const errors: PackageRegistryError[] = [];

	if (!entry || typeof entry !== "object") {
		errors.push({
			message: `Package entry at index ${index} is not an object`,
			code: "INVALID_ENTRY",
		});
		return errors;
	}

	const pkg = entry as Record<string, unknown>;

	// Required string fields
	const requiredStrings = [
		"name",
		"version",
		"author",
		"description",
		"artifactUrl",
	];
	for (const field of requiredStrings) {
		if (
			typeof pkg[field] !== "string" ||
			(pkg[field] as string).trim() === ""
		) {
			errors.push({
				message: `Package at index ${index}: missing or empty required field '${field}'`,
				code: "INVALID_ENTRY",
			});
		}
	}

	// Category validation
	if (
		typeof pkg.category !== "string" ||
		!VALID_CATEGORIES.includes(pkg.category as PackageCategory)
	) {
		errors.push({
			message: `Package at index ${index}: invalid category '${pkg.category}'. Must be one of: ${VALID_CATEGORIES.join(", ")}`,
			code: "INVALID_ENTRY",
		});
	}

	// Optional number fields
	if (pkg.downloads !== null && pkg.downloads !== undefined) {
		if (typeof pkg.downloads !== "number" || pkg.downloads < 0) {
			errors.push({
				message: `Package at index ${index}: 'downloads' must be a non-negative number or null`,
				code: "INVALID_ENTRY",
			});
		}
	}

	if (pkg.rating !== null && pkg.rating !== undefined) {
		if (typeof pkg.rating !== "number" || pkg.rating < 0 || pkg.rating > 5) {
			errors.push({
				message: `Package at index ${index}: 'rating' must be a number between 0 and 5 or null`,
				code: "INVALID_ENTRY",
			});
		}
	}

	// Checksum is optional string
	if (pkg.checksum !== null && pkg.checksum !== undefined) {
		if (typeof pkg.checksum !== "string") {
			errors.push({
				message: `Package at index ${index}: 'checksum' must be a string or null`,
				code: "INVALID_ENTRY",
			});
		}
	}

	// Supported devices array
	if (!Array.isArray(pkg.supportedDevices)) {
		errors.push({
			message: `Package at index ${index}: 'supportedDevices' must be an array`,
			code: "INVALID_ENTRY",
		});
	} else {
		for (const device of pkg.supportedDevices) {
			if (typeof device !== "string") {
				errors.push({
					message: `Package at index ${index}: all entries in 'supportedDevices' must be strings`,
					code: "INVALID_ENTRY",
				});
				break;
			}
		}
	}

	// Install path rules
	if (!pkg.installPathRules || typeof pkg.installPathRules !== "object") {
		errors.push({
			message: `Package at index ${index}: 'installPathRules' must be an object`,
			code: "INVALID_ENTRY",
		});
	} else {
		const rules = pkg.installPathRules as Record<string, unknown>;
		if (typeof rules.targetDir !== "string" || rules.targetDir.trim() === "") {
			errors.push({
				message: `Package at index ${index}: 'installPathRules.targetDir' must be a non-empty string`,
				code: "INVALID_ENTRY",
			});
		}
		if (typeof rules.extractToRoot !== "boolean") {
			errors.push({
				message: `Package at index ${index}: 'installPathRules.extractToRoot' must be a boolean`,
				code: "INVALID_ENTRY",
			});
		}
	}

	return errors;
}

export function validatePackageRegistry(
	data: unknown,
): PackageRegistryValidationResult {
	const errors: PackageRegistryError[] = [];
	const warnings: string[] = [];

	if (!data || typeof data !== "object") {
		errors.push({
			message: "Registry data is not an object",
			code: "PARSE_ERROR",
		});
		return { valid: false, errors, warnings };
	}

	const registry = data as Record<string, unknown>;

	// Version is required
	if (typeof registry.version !== "string" || registry.version.trim() === "") {
		errors.push({
			message: "Registry missing or empty 'version' field",
			code: "VALIDATION_ERROR",
		});
	}

	// Packages array is required
	if (!Array.isArray(registry.packages)) {
		errors.push({
			message: "Registry 'packages' must be an array",
			code: "VALIDATION_ERROR",
		});
		return { valid: false, errors, warnings };
	}

	// Validate each package entry
	for (let i = 0; i < registry.packages.length; i++) {
		const entryErrors = validatePackageEntry(registry.packages[i], i);
		errors.push(...entryErrors);
	}

	// Check for duplicate package names
	const names = new Set<string>();
	for (const pkg of registry.packages) {
		if (
			pkg &&
			typeof pkg === "object" &&
			typeof (pkg as Record<string, unknown>).name === "string"
		) {
			const name = (pkg as Record<string, unknown>).name as string;
			if (names.has(name)) {
				warnings.push(`Duplicate package name: ${name}`);
			}
			names.add(name);
		}
	}

	return {
		valid: errors.length === 0,
		errors,
		warnings,
	};
}

export function parsePackageRegistry(data: unknown): {
	registry: PackageRegistry | null;
	errors: PackageRegistryError[];
} {
	if (!data || typeof data !== "object") {
		return {
			registry: null,
			errors: [
				{ message: "Registry data is not an object", code: "PARSE_ERROR" },
			],
		};
	}

	const raw = data as Record<string, unknown>;

	// Version is required
	if (typeof raw.version !== "string" || raw.version.trim() === "") {
		return {
			registry: null,
			errors: [
				{
					message: "Registry missing or empty 'version' field",
					code: "VALIDATION_ERROR",
				},
			],
		};
	}

	// Packages array is required
	if (!Array.isArray(raw.packages)) {
		return {
			registry: null,
			errors: [
				{
					message: "Registry 'packages' must be an array",
					code: "VALIDATION_ERROR",
				},
			],
		};
	}

	// Filter out invalid entries (log warnings but don't fail)
	const validPackages: PackageRegistryEntry[] = [];
	for (let i = 0; i < raw.packages.length; i++) {
		const entryErrors = validatePackageEntry(raw.packages[i], i);
		if (entryErrors.length === 0) {
			const pkg = raw.packages[i] as Record<string, unknown>;
			validPackages.push({
				name: pkg.name as string,
				version: pkg.version as string,
				author: pkg.author as string,
				category: pkg.category as PackageCategory,
				description: pkg.description as string,
				downloads: (pkg.downloads as number) ?? null,
				rating: (pkg.rating as number) ?? null,
				artifactUrl: pkg.artifactUrl as string,
				checksum: (pkg.checksum as string) ?? null,
				supportedDevices: (pkg.supportedDevices as string[]) || [],
				installPathRules: pkg.installPathRules as PackageInstallPathRules,
			});
		}
	}

	return {
		registry: {
			version: raw.version as string,
			packages: validPackages,
		},
		errors: [],
	};
}

export async function installPackage(options: {
	artifactUrl: string;
	checksum?: string;
	sdMount: string;
	targetDir: string;
	extractToRoot: boolean;
}): Promise<PackageInstallResultEither> {
	try {
		const { invoke } = await import("@tauri-apps/api/core");
		const result = await invoke<PackageInstallResult>("install_package", {
			artifactUrl: options.artifactUrl,
			checksum: options.checksum || null,
			sdMount: options.sdMount,
			targetDir: options.targetDir,
			extractToRoot: options.extractToRoot,
		});

		if (result.success) {
			return { success: true, data: result };
		}

		const errorMsg = result.error || "Package installation failed";
		let code: PackageInstallError["code"] = "COPY_ERROR";

		if (errorMsg.includes("download")) {
			code = "DOWNLOAD_ERROR";
		} else if (
			errorMsg.includes("extraction") ||
			errorMsg.includes("extract")
		) {
			code = "EXTRACTION_ERROR";
		} else if (errorMsg.includes("checksum")) {
			code = "CHECKSUM_ERROR";
		}

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

export async function fetchPackageRegistry(
	fetchFn: typeof globalThis.fetch = globalThis.fetch,
	registryUrl: string = REGISTRY_URL,
): Promise<PackageRegistryFetchResult> {
	try {
		const response = await fetchFn(registryUrl, {
			headers: { Accept: "application/json" },
		});

		if (!response.ok) {
			if (response.status === 404) {
				return {
					success: false,
					error: { message: "Registry not found", code: "NOT_FOUND" },
				};
			}
			return {
				success: false,
				error: {
					message: `Registry fetch error: ${response.status}`,
					code: "NETWORK_ERROR",
				},
			};
		}

		const data = await response.json();
		const result = parsePackageRegistry(data);

		if (!result.registry) {
			return {
				success: false,
				error: result.errors[0] || {
					message: "Failed to parse registry",
					code: "PARSE_ERROR",
				},
			};
		}

		return { success: true, data: result.registry };
	} catch (err) {
		const message =
			err instanceof Error ? err.message : "Unknown network error";
		return {
			success: false,
			error: { message, code: "NETWORK_ERROR" },
		};
	}
}
