import { describe, expect, it } from "vitest";
import type { ValidationResult } from "./validate";

describe("ValidationResult types", () => {
	it("should define ValidationCheck interface correctly", () => {
		const check = {
			name: "Check: minui.pak",
			passed: true,
			message: "Found: minui.pak",
		};
		expect(check.name).toBe("Check: minui.pak");
		expect(check.passed).toBe(true);
		expect(check.message).toBe("Found: minui.pak");
	});

	it("should define ValidationResult with success true", () => {
		const result: ValidationResult = {
			success: true,
			checks: [
				{ name: "Check 1", passed: true, message: "OK" },
				{ name: "Check 2", passed: true, message: "OK" },
			],
			passed_count: 2,
			failed_count: 0,
			free_space_bytes: 1024 * 1024 * 500,
		};
		expect(result.success).toBe(true);
		expect(result.passed_count).toBe(2);
		expect(result.failed_count).toBe(0);
		expect(result.free_space_bytes).toBe(524288000);
	});

	it("should define ValidationResult with success false", () => {
		const result: ValidationResult = {
			success: false,
			checks: [
				{ name: "Check 1", passed: true, message: "OK" },
				{ name: "Check 2", passed: false, message: "Missing" },
			],
			passed_count: 1,
			failed_count: 1,
			free_space_bytes: null,
		};
		expect(result.success).toBe(false);
		expect(result.failed_count).toBe(1);
		expect(result.free_space_bytes).toBeNull();
	});
});

describe("formatReportLocally", () => {
	it("should format a passing result", async () => {
		const { formatValidationReport } = await import("./validate");

		// Mock invoke to throw so it falls back to local formatting
		const result: ValidationResult = {
			success: true,
			checks: [
				{
					name: "Check: minui.pak",
					passed: true,
					message: "Found: minui.pak",
				},
			],
			passed_count: 1,
			failed_count: 0,
			free_space_bytes: 1024 * 1024 * 100,
		};

		// formatValidationReport calls invoke which will fail in test env,
		// so it falls back to formatReportLocally
		const report = await formatValidationReport(result);
		expect(report).toContain("MinUI Installation Validation Report");
		expect(report).toContain("PASSED");
		expect(report).toContain("Found: minui.pak");
		expect(report).toContain("1 passed, 0 failed");
		expect(report).toContain("100.00 MB");
	});

	it("should format a failing result", async () => {
		const { formatValidationReport } = await import("./validate");

		const result: ValidationResult = {
			success: false,
			checks: [
				{
					name: "Check: boot.sh",
					passed: false,
					message: "Missing: boot.sh",
				},
			],
			passed_count: 0,
			failed_count: 1,
			free_space_bytes: null,
		};

		const report = await formatValidationReport(result);
		expect(report).toContain("FAILED");
		expect(report).toContain("Missing: boot.sh");
		expect(report).toContain("0 passed, 1 failed");
	});

	it("should format free space in GB", async () => {
		const { formatValidationReport } = await import("./validate");

		const result: ValidationResult = {
			success: true,
			checks: [],
			passed_count: 0,
			failed_count: 0,
			free_space_bytes: 1024 * 1024 * 1024 * 2.5,
		};

		const report = await formatValidationReport(result);
		expect(report).toContain("2.50 GB");
	});
});
