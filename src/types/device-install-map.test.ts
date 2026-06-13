import { describe, expect, it } from "vitest";
import {
	getAllDeviceIds,
	getBasePlatform,
	getDeviceInstallRules,
	getDevicePaks,
	getExtrasPlatform,
	SHARED_BIOS,
	validateDeviceInstallMap,
} from "./device-install-map";

describe("device-install-map", () => {
	it("has valid structure", () => {
		const result = validateDeviceInstallMap();
		expect(result.valid).toBe(true);
		expect(result.errors).toHaveLength(0);
	});

	it("returns all device ids", () => {
		const ids = getAllDeviceIds();
		expect(ids.length).toBeGreaterThan(0);
		expect(ids).toContain("trimui-smart-pro");
		expect(ids).toContain("rgb30");
		expect(ids).toContain("miyoo-a30");
	});

	it("returns install rules for valid device", () => {
		const rules = getDeviceInstallRules("trimui-smart-pro");
		expect(rules).toBeDefined();
		expect(rules?.name).toBe("TrimUI Smart Pro");
		expect(rules?.basePlatform).toBe("trimui");
		expect(rules?.extrasPlatform).toBe("tg5040");
	});

	it("returns undefined for unknown device", () => {
		expect(getDeviceInstallRules("unknown")).toBeUndefined();
	});

	it("returns extras platform for TrimUI Smart Pro as tg5040", () => {
		expect(getExtrasPlatform("trimui-smart-pro")).toBe("tg5040");
	});

	it("returns extras platform for TrimUI Brick as tg5040", () => {
		expect(getExtrasPlatform("trimui-brick")).toBe("tg5040");
	});

	it("returns extras platform for Miyoo A30 as my282", () => {
		expect(getExtrasPlatform("miyoo-a30")).toBe("my282");
	});

	it("returns extras platform for Miyoo 355 as my355", () => {
		expect(getExtrasPlatform("miyoo355")).toBe("my355");
	});

	it("returns extras platform for MagicX as magicmini", () => {
		expect(getExtrasPlatform("magicx")).toBe("magicmini");
	});

	it("returns base platform for device", () => {
		expect(getBasePlatform("miyoo-a30")).toBe("miyoo285");
		expect(getBasePlatform("rg35xx-plus")).toBe("rg35xxplus");
	});

	it("returns device paks for RGB30", () => {
		const paks = getDevicePaks("rgb30");
		expect(paks).toHaveLength(3);
		expect(paks.map((p) => p.name)).toContain("Wi-Fi.pak");
		expect(paks.map((p) => p.name)).toContain("Splore.pak");
		expect(paks.map((p) => p.name)).toContain("P8-NATIVE.pak");
	});

	it("returns empty paks for devices without special paks", () => {
		expect(getDevicePaks("rg35xx-plus")).toHaveLength(0);
	});

	it("defines shared BIOS files", () => {
		expect(SHARED_BIOS).toContain("gba_bios.bin");
		expect(SHARED_BIOS).toContain("syscard3.pce");
		expect(SHARED_BIOS).toContain("bios.min");
		expect(SHARED_BIOS).toContain("sgb.bios");
	});
});
