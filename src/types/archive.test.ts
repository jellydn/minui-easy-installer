import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DownloadResult, ExtractionResult } from "./archive";
import { downloadArchive, extractArchive, verifyChecksum } from "./archive";

// Mock the Tauri invoke function
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(),
}));

describe("downloadArchive", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("downloads archive successfully", async () => {
		const mockResult: DownloadResult = {
			success: true,
			file_path: "/tmp/test-archive.zip",
			checksum_verified: null,
			error: null,
		};

		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(mockResult);

		const result = await downloadArchive("https://example.com/archive.zip");

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.file_path).toBe("/tmp/test-archive.zip");
			expect(result.data.checksum_verified).toBeNull();
		}
		expect(invoke).toHaveBeenCalledWith("download_and_verify_archive", {
			url: "https://example.com/archive.zip",
			checksum: null,
		});
	});

	it("downloads and verifies checksum successfully", async () => {
		const mockResult: DownloadResult = {
			success: true,
			file_path: "/tmp/test-archive.zip",
			checksum_verified: true,
			error: null,
		};

		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(mockResult);

		const result = await downloadArchive(
			"https://example.com/archive.zip",
			"abc123",
		);

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.checksum_verified).toBe(true);
		}
		expect(invoke).toHaveBeenCalledWith("download_and_verify_archive", {
			url: "https://example.com/archive.zip",
			checksum: "abc123",
		});
	});

	it("handles checksum mismatch", async () => {
		const mockResult: DownloadResult = {
			success: false,
			file_path: null,
			checksum_verified: false,
			error: "Checksum mismatch",
		};

		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(mockResult);

		const result = await downloadArchive(
			"https://example.com/archive.zip",
			"wrong_checksum",
		);

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("CHECKSUM_ERROR");
			expect(result.error.message).toBe("Checksum mismatch");
		}
	});

	it("handles network errors", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockRejectedValue(new Error("Network failure"));

		const result = await downloadArchive("https://example.com/archive.zip");

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("UNKNOWN_ERROR");
			expect(result.error.message).toBe("Network failure");
		}
	});

	it("handles download failure", async () => {
		const mockResult: DownloadResult = {
			success: false,
			file_path: null,
			checksum_verified: null,
			error: "Download failed with status: 404",
		};

		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(mockResult);

		const result = await downloadArchive("https://example.com/archive.zip");

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("NETWORK_ERROR");
			expect(result.error.message).toContain("404");
		}
	});
});

describe("verifyChecksum", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("verifies checksum successfully", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(true);

		const result = await verifyChecksum("/tmp/test.zip", "abc123");

		expect(result.success).toBe(true);
		expect(result.verified).toBe(true);
		expect(invoke).toHaveBeenCalledWith("verify_archive_checksum", {
			filePath: "/tmp/test.zip",
			expectedChecksum: "abc123",
		});
	});

	it("handles checksum mismatch", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(false);

		const result = await verifyChecksum("/tmp/test.zip", "abc123");

		expect(result.success).toBe(true);
		expect(result.verified).toBe(false);
	});

	it("handles verification errors", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockRejectedValue(new Error("File not found"));

		const result = await verifyChecksum("/tmp/test.zip", "abc123");

		expect(result.success).toBe(false);
		expect(result.verified).toBe(false);
		expect(result.error).toBe("File not found");
	});
});

describe("extractArchive", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("extracts archive successfully", async () => {
		const mockResult: ExtractionResult = {
			success: true,
			output_path: "/tmp/extracted",
			files_extracted: 5,
			error: null,
		};

		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(mockResult);

		const result = await extractArchive("/tmp/archive.zip", "/tmp/extracted");

		expect(result.success).toBe(true);
		if (result.success) {
			expect(result.data.output_path).toBe("/tmp/extracted");
			expect(result.data.files_extracted).toBe(5);
		}
		expect(invoke).toHaveBeenCalledWith("extract_archive_to_directory", {
			archivePath: "/tmp/archive.zip",
			destination: "/tmp/extracted",
		});
	});

	it("extracts archive without destination (uses temp dir)", async () => {
		const mockResult: ExtractionResult = {
			success: true,
			output_path: "/tmp/random-dir",
			files_extracted: 3,
			error: null,
		};

		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(mockResult);

		const result = await extractArchive("/tmp/archive.zip");

		expect(result.success).toBe(true);
		expect(invoke).toHaveBeenCalledWith("extract_archive_to_directory", {
			archivePath: "/tmp/archive.zip",
			destination: null,
		});
	});

	it("handles path traversal detection", async () => {
		const mockResult: ExtractionResult = {
			success: false,
			output_path: null,
			files_extracted: null,
			error: "Path traversal detected in archive entry: ../etc/passwd",
		};

		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(mockResult);

		const result = await extractArchive("/tmp/malicious.zip");

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("PATH_TRAVERSAL");
			expect(result.error.message).toContain("Path traversal");
		}
	});

	it("handles security violation", async () => {
		const mockResult: ExtractionResult = {
			success: false,
			output_path: null,
			files_extracted: null,
			error: "Security violation: entry path escapes destination directory",
		};

		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(mockResult);

		const result = await extractArchive("/tmp/malicious.zip");

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("PATH_TRAVERSAL");
		}
	});

	it("handles file errors", async () => {
		const mockResult: ExtractionResult = {
			success: false,
			output_path: null,
			files_extracted: null,
			error: "Failed to open archive: No such file or directory",
		};

		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(mockResult);

		const result = await extractArchive("/nonexistent/archive.zip");

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("FILE_ERROR");
		}
	});

	it("handles archive errors", async () => {
		const mockResult: ExtractionResult = {
			success: false,
			output_path: null,
			files_extracted: null,
			error: "Failed to read ZIP archive: invalid zip file",
		};

		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue(mockResult);

		const result = await extractArchive("/tmp/corrupt.zip");

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("ARCHIVE_ERROR");
		}
	});

	it("handles unknown errors", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockRejectedValue(new Error("Unexpected failure"));

		const result = await extractArchive("/tmp/archive.zip");

		expect(result.success).toBe(false);
		if (!result.success) {
			expect(result.error.code).toBe("UNKNOWN_ERROR");
			expect(result.error.message).toBe("Unexpected failure");
		}
	});
});
