import { describe, expect, it, vi } from "vitest";
import type { MinUIRelease, ReleaseFetchError } from "./release";
import {
	fetchMinUIRelease,
	GITHUB_API_URL,
	parseGitHubRelease,
} from "./release";

describe("parseGitHubRelease", () => {
	it("parses a valid GitHub release with base and extras", () => {
		const input = {
			tag_name: "v25.06.12",
			assets: [
				{
					browser_download_url:
						"https://github.com/shauninman/MinUI/releases/download/v25.06.12/MinUI-25.06.12-base.zip",
				},
				{
					browser_download_url:
						"https://github.com/shauninman/MinUI/releases/download/v25.06.12/MinUI-25.06.12-extras.zip",
				},
			],
		};

		const result = parseGitHubRelease(input) as MinUIRelease;

		expect(result).toEqual({
			version: "25.06.12",
			baseArchiveUrl: expect.stringContaining("base.zip"),
			extrasArchiveUrl: expect.stringContaining("extras.zip"),
			checksums: null,
		});
	});

	it("parses a release with only base archive", () => {
		const input = {
			tag_name: "v25.06.12",
			assets: [
				{
					browser_download_url:
						"https://github.com/shauninman/MinUI/releases/download/v25.06.12/MinUI-25.06.12-base.zip",
				},
			],
		};

		const result = parseGitHubRelease(input) as MinUIRelease;

		expect(result.version).toBe("25.06.12");
		expect(result.baseArchiveUrl).toContain("base.zip");
		expect(result.extrasArchiveUrl).toBeNull();
	});

	it("strips v prefix from tag_name", () => {
		const input = {
			tag_name: "v1.2.3",
			assets: [
				{
					browser_download_url: "https://example.com/MinUI-1.2.3-base.zip",
				},
			],
		};

		const result = parseGitHubRelease(input) as MinUIRelease;
		expect(result.version).toBe("1.2.3");
	});

	it("returns error for null input", () => {
		const result = parseGitHubRelease(null) as ReleaseFetchError;
		expect(result.code).toBe("PARSE_ERROR");
	});

	it("returns error for missing tag_name", () => {
		const input = { assets: [] };
		const result = parseGitHubRelease(input) as ReleaseFetchError;
		expect(result.code).toBe("PARSE_ERROR");
		expect(result.message).toContain("tag_name");
	});

	it("returns error when no base archive found", () => {
		const input = {
			tag_name: "v1.0.0",
			assets: [
				{
					browser_download_url: "https://example.com/MinUI-1.0.0-something.zip",
				},
			],
		};

		const result = parseGitHubRelease(input) as ReleaseFetchError;
		expect(result.code).toBe("NOT_FOUND");
		expect(result.message).toContain("base archive");
	});

	it("handles empty assets array", () => {
		const input = { tag_name: "v1.0.0", assets: [] };
		const result = parseGitHubRelease(input) as ReleaseFetchError;
		expect(result.code).toBe("NOT_FOUND");
	});
});

describe("fetchMinUIRelease", () => {
	it("fetches and parses release successfully", async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () =>
				Promise.resolve({
					tag_name: "v25.06.12",
					assets: [
						{
							browser_download_url:
								"https://github.com/shauninman/MinUI/releases/download/v25.06.12/MinUI-25.06.12-base.zip",
						},
						{
							browser_download_url:
								"https://github.com/shauninman/MinUI/releases/download/v25.06.12/MinUI-25.06.12-extras.zip",
						},
					],
				}),
		});

		const result = await fetchMinUIRelease(mockFetch);

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.version).toBe("25.06.12");
			expect(result.data.baseArchiveUrl).toContain("base.zip");
			expect(result.data.extrasArchiveUrl).toContain("extras.zip");
		}
		expect(mockFetch).toHaveBeenCalledWith(GITHUB_API_URL, {
			headers: { Accept: "application/vnd.github+json" },
		});
	});

	it("handles 404 not found", async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 404,
		});

		const result = await fetchMinUIRelease(mockFetch);

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("NOT_FOUND");
		}
	});

	it("handles other HTTP errors", async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: false,
			status: 500,
		});

		const result = await fetchMinUIRelease(mockFetch);

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("NETWORK_ERROR");
			expect(result.error.message).toContain("500");
		}
	});

	it("handles network errors", async () => {
		const mockFetch = vi.fn().mockRejectedValue(new Error("Network failure"));

		const result = await fetchMinUIRelease(mockFetch);

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("NETWORK_ERROR");
			expect(result.error.message).toBe("Network failure");
		}
	});

	it("handles parse errors from invalid response", async () => {
		const mockFetch = vi.fn().mockResolvedValue({
			ok: true,
			json: () => Promise.resolve({ invalid: "data" }),
		});

		const result = await fetchMinUIRelease(mockFetch);

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("PARSE_ERROR");
		}
	});
});
