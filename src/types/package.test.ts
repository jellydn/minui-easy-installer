import { describe, expect, test, vi } from "vitest";
import type { PackageRegistry, PackageRegistryEntry } from "./package";
import {
	fetchPackageRegistry,
	parsePackageRegistry,
	REGISTRY_URL,
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
	const validRegistryData = {
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

	test("fetches registry successfully", async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve(validRegistryData),
		});

		const result = await fetchPackageRegistry(mockFetch);
		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.version).toBe("1.0.0");
			expect(result.data.packages).toHaveLength(1);
		}
		expect(mockFetch).toHaveBeenCalledWith(REGISTRY_URL, expect.any(Object));
	});

	test("handles 404 response", async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 404,
		});

		const result = await fetchPackageRegistry(mockFetch);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("NOT_FOUND");
		}
	});

	test("handles network error", async () => {
		const mockFetch = vi.fn().mockRejectedValue(new Error("Network error"));

		const result = await fetchPackageRegistry(mockFetch);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("NETWORK_ERROR");
		}
	});

	test("handles invalid registry data", async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ invalid: true }),
		});

		const result = await fetchPackageRegistry(mockFetch);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("VALIDATION_ERROR");
		}
	});

	test("rejects non-allowlisted registry URL", async () => {
		const customUrl = "https://custom.registry.com/packages.json";
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve(validRegistryData),
		});

		const result = await fetchPackageRegistry(mockFetch, customUrl);
		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("INVALID_ENTRY");
		}
		expect(mockFetch).not.toHaveBeenCalled();
	});

	test("accepts allowlisted registry URL", async () => {
		const customUrl =
			"https://raw.githubusercontent.com/josegonzalez/pakman/main/paks.json";
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve(validRegistryData),
		});

		const result = await fetchPackageRegistry(mockFetch, customUrl);
		expect(result.success).toBe(true);
	});

	test("fetches and converts pakman registry format", async () => {
		const pakmanData = {
			emu_paks: [
				{
					name: "Dreamcast",
					repository: "https://github.com/josegonzalez/minui-dreamcast-pak",
					version: "0.5.0",
					pak_name: "DC",
					rom_folder: "Roms/Sega Dreamcast (DC)",
				},
			],
			tool_paks: [
				{
					name: "SSH Server",
					repository:
						"https://github.com/josegonzalez/minui-dropbear-server-pak",
					version: "0.9.0",
					pak_name: "SSH Server",
				},
			],
		};
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve(pakmanData),
		});

		const result = await fetchPackageRegistry(mockFetch);
		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.packages).toHaveLength(2);
			const emu = result.data.packages[0];
			expect(emu.name).toBe("Dreamcast");
			expect(emu.category).toBe("Emulators");
			expect(emu.artifactUrl).toBe(
				"https://github.com/josegonzalez/minui-dreamcast-pak/releases/download/0.5.0/DC.pak.zip",
			);
			expect(emu.checksum).toBeNull();
			const tool = result.data.packages[1];
			expect(tool.name).toBe("SSH Server");
			expect(tool.category).toBe("Utilities");
		}
	});
});
