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

export interface PackageRegistryError {
	message: string;
	code: "INVALID_ENTRY" | "VALIDATION_ERROR" | "PARSE_ERROR";
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
