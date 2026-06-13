import { describe, expect, test } from "vitest";
import type { PackageRegistry, PackageRegistryEntry } from "./package";
import {
	fetchPackageRegistry,
	parsePackageRegistry,
	validatePackageEntry,
	validatePackageRegistry,
} from "./package";

describe("Package registry validation", () => {
	const validEntry: PackageRegistryEntry = {
		name: "wifi.pak",
		version: "1.0.0",
		author: "MinUI",
		category: "Network",
		description: "WiFi support for MinUI",
		downloads: 1000,
		rating: 4.5,
		artifactUrl: "https://github.com/minui/wifi.pak.zip",
		checksum: "abc123",
		supportedDevices: ["miyoo-mini-plus", "trimui-brick"],
		installPathRules: {
			targetDir: "/Tools",
			extractToRoot: false,
		},
	};

	test("valid entry passes validation", () => {
		const errors = validatePackageEntry(validEntry, 0);
		expect(errors).toHaveLength(0);
	});

	test("missing name fails validation", () => {
		const entry = { ...validEntry, name: "" };
		const errors = validatePackageEntry(entry, 0);
		expect(errors.length).toBeGreaterThan(0);
		expect(errors[0].message).toContain("name");
	});

	test("invalid category fails validation", () => {
		const entry = { ...validEntry, category: "Invalid" };
		const errors = validatePackageEntry(entry, 0);
		expect(errors.length).toBeGreaterThan(0);
		expect(errors[0].message).toContain("category");
	});

	test("negative downloads fails validation", () => {
		const entry = { ...validEntry, downloads: -1 };
		const errors = validatePackageEntry(entry, 0);
		expect(errors.length).toBeGreaterThan(0);
		expect(errors[0].message).toContain("downloads");
	});

	test("rating out of range fails validation", () => {
		const entry = { ...validEntry, rating: 6 };
		const errors = validatePackageEntry(entry, 0);
		expect(errors.length).toBeGreaterThan(0);
		expect(errors[0].message).toContain("rating");
	});

	test("null optional fields pass validation", () => {
		const entry = {
			...validEntry,
			downloads: null,
			rating: null,
			checksum: null,
		};
		const errors = validatePackageEntry(entry, 0);
		expect(errors).toHaveLength(0);
	});

	test("non-object entry fails validation", () => {
		const errors = validatePackageEntry("not an object", 0);
		expect(errors.length).toBeGreaterThan(0);
		expect(errors[0].code).toBe("INVALID_ENTRY");
	});

	test("missing supportedDevices fails validation", () => {
		const entry = { ...validEntry, supportedDevices: undefined };
		const errors = validatePackageEntry(entry, 0);
		expect(errors.length).toBeGreaterThan(0);
		expect(errors[0].message).toContain("supportedDevices");
	});

	test("path traversal in targetDir fails validation", () => {
		const entry = {
			...validEntry,
			installPathRules: {
				targetDir: "/Tools/../../etc",
				extractToRoot: false,
			},
		};
		const errors = validatePackageEntry(entry, 0);
		expect(errors.length).toBeGreaterThan(0);
		expect(errors.some((e) => e.message.includes("traversal"))).toBe(true);
	});

	test("http artifactUrl fails validation", () => {
		const entry = {
			...validEntry,
			artifactUrl: "http://github.com/minui/wifi.pak.zip",
		};
		const errors = validatePackageEntry(entry, 0);
		expect(errors.length).toBeGreaterThan(0);
		expect(errors.some((e) => e.message.includes("https"))).toBe(true);
	});

	test("disallowed host for artifactUrl fails validation", () => {
		const entry = {
			...validEntry,
			artifactUrl: "https://evil.com/malicious.zip",
		};
		const errors = validatePackageEntry(entry, 0);
		expect(errors.length).toBeGreaterThan(0);
		expect(errors.some((e) => e.message.includes("allowed"))).toBe(true);
	});

	test("missing installPathRules fails validation", () => {
		const entry = { ...validEntry, installPathRules: undefined };
		const errors = validatePackageEntry(entry, 0);
		expect(errors.length).toBeGreaterThan(0);
		expect(errors[0].message).toContain("installPathRules");
	});
});

describe("Package registry validation", () => {
	const validRegistry: PackageRegistry = {
		version: "1.0.0",
		packages: [
			{
				name: "wifi.pak",
				version: "1.0.0",
				author: "MinUI",
				category: "Network",
				description: "WiFi support",
				downloads: 100,
				rating: 4.0,
				artifactUrl: "https://github.com/minui/wifi.pak.zip",
				checksum: null,
				supportedDevices: ["miyoo-mini-plus"],
				installPathRules: {
					targetDir: "/Tools",
					extractToRoot: false,
				},
			},
		],
	};

	test("valid registry passes validation", () => {
		const result = validatePackageRegistry(validRegistry);
		expect(result.valid).toBe(true);
		expect(result.errors).toHaveLength(0);
	});

	test("missing version fails validation", () => {
		const registry = { ...validRegistry, version: "" };
		const result = validatePackageRegistry(registry);
		expect(result.valid).toBe(false);
		expect(result.errors.length).toBeGreaterThan(0);
	});

	test("missing packages array fails validation", () => {
		const registry = { ...validRegistry, packages: undefined };
		const result = validatePackageRegistry(registry);
		expect(result.valid).toBe(false);
		expect(result.errors.length).toBeGreaterThan(0);
	});

	test("invalid package entry fails validation", () => {
		const registry = {
			...validRegistry,
			packages: [{ name: "" }],
		};
		const result = validatePackageRegistry(registry);
		expect(result.valid).toBe(false);
		expect(result.errors.length).toBeGreaterThan(0);
	});

	test("non-object data fails validation", () => {
		const result = validatePackageRegistry("not an object");
		expect(result.valid).toBe(false);
		expect(result.errors[0].code).toBe("PARSE_ERROR");
	});

	test("duplicate package names generate warning", () => {
		const registry = {
			...validRegistry,
			packages: [validRegistry.packages[0], validRegistry.packages[0]],
		};
		const result = validatePackageRegistry(registry);
		expect(result.warnings.length).toBeGreaterThan(0);
		expect(result.warnings[0]).toContain("Duplicate");
	});
});

describe("parsePackageRegistry", () => {
	const validRegistry = {
		version: "1.0.0",
		packages: [
			{
				name: "wifi.pak",
				version: "1.0.0",
				author: "MinUI",
				category: "Network",
				description: "WiFi support",
				downloads: 100,
				rating: 4.0,
				artifactUrl: "https://github.com/minui/wifi.pak.zip",
				checksum: null,
				supportedDevices: ["miyoo-mini-plus"],
				installPathRules: {
					targetDir: "/Tools",
					extractToRoot: false,
				},
			},
		],
	};

	test("parses valid registry", () => {
		const result = parsePackageRegistry(validRegistry);
		expect(result.registry).not.toBeNull();
		expect(result.registry?.version).toBe("1.0.0");
		expect(result.registry?.packages).toHaveLength(1);
		expect(result.registry?.packages[0].name).toBe("wifi.pak");
		expect(result.errors).toHaveLength(0);
	});

	test("returns null for invalid registry", () => {
		const result = parsePackageRegistry({ version: "1.0.0" });
		expect(result.registry).toBeNull();
		expect(result.errors.length).toBeGreaterThan(0);
	});

	test("filters out invalid entries", () => {
		const registry = {
			version: "1.0.0",
			packages: [
				validRegistry.packages[0],
				{ name: "" }, // Invalid entry
			],
		};
		const result = parsePackageRegistry(registry);
		expect(result.registry).not.toBeNull();
		expect(result.registry?.packages).toHaveLength(1);
		expect(result.registry?.packages[0].name).toBe("wifi.pak");
	});
});

describe("fetchPackageRegistry", () => {
	test("loads and converts local store.json", async () => {
		const result = await fetchPackageRegistry();

		expect(result.success).toBe(true);
		if (!result.success) return;

		expect(result.data.version).toBe("1.0.0");
		expect(result.data.packages.length).toBeGreaterThan(0);

		// Verify Grout is included
		const grout = result.data.packages.find((p) => p.name === "Grout");
		expect(grout).toBeDefined();
		expect(grout!.artifactUrl).toBe(
			"https://github.com/rommapp/grout/releases/download/v4.8.1.0/Grout-MinUI.zip",
		);
		expect(grout!.category).toBe("Utilities");

		// Verify dreamcast is in emulators
		const dc = result.data.packages.find((p) => p.name === "Dreamcast");
		expect(dc).toBeDefined();
		expect(dc!.category).toBe("Emulators");

		// Verify device-filtered tools have supportedDevices
		const mediaPlayer = result.data.packages.find(
			(p) => p.name === "Media Player",
		);
		expect(mediaPlayer).toBeDefined();
		expect(mediaPlayer!.supportedDevices).toEqual(["brick"]);
	});
});
