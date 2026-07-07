import { beforeEach, describe, expect, it, type Mock, vi } from "vitest";
import type { InstallResult } from "./install";
import { installMinui } from "./install";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

/** Expected IPC payload keys shared by all installMinui calls. */
const IPC_KEYS = {
  extrasUrl: null,
  baseChecksum: null,
  extrasChecksum: null,
  forkName: null,
};

describe("installMinui", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("returns success with file counts on successful install", async () => {
    const mockResult: InstallResult = {
      success: true,
      error: null,
      base_files_copied: 15,
      extras_files_copied: 3,
      extras_warning: null,
      rom_dirs_created: 0,
    };

    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockResolvedValue(mockResult);

    const result = await installMinui({
      baseUrl: "https://example.com/base.zip",
      sdMount: "/Volumes/SDCARD",
      platform: "miyoo-mini-plus",
      extrasPlatform: "/Tools",
      version: "2025.01.01",
    });

    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.base_files_copied).toBe(15);
      expect(result.data.extras_files_copied).toBe(3);
    }
    expect(invoke).toHaveBeenCalledWith("install_minui", {
      baseUrl: "https://example.com/base.zip",
      sdMount: "/Volumes/SDCARD",
      platform: "miyoo-mini-plus",
      extrasPlatform: "/Tools",
      version: "2025.01.01",
      ...IPC_KEYS,
    });
  });

  it("returns error with copy code on failed install", async () => {
    const mockResult: InstallResult = {
      success: false,
      error: "Failed to copy file to SD card",
      base_files_copied: 0,
      extras_files_copied: 0,
      extras_warning: null,
      rom_dirs_created: 0,
    };

    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockResolvedValue(mockResult);

    const result = await installMinui({
      baseUrl: "https://example.com/base.zip",
      sdMount: "/Volumes/SDCARD",
      platform: "miyoo-mini-plus",
      extrasPlatform: "/Tools",
      version: "2025.01.01",
    });

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.code).toBe("COPY_ERROR");
      expect(result.error.message).toContain("copy");
    }
  });

  it("returns download error code on download failure", async () => {
    const mockResult: InstallResult = {
      success: false,
      error: "Base download failed: timeout",
      base_files_copied: 0,
      extras_files_copied: 0,
      extras_warning: null,
      rom_dirs_created: 0,
    };

    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockResolvedValue(mockResult);

    const result = await installMinui({
      baseUrl: "https://example.com/base.zip",
      sdMount: "/Volumes/SDCARD",
      platform: "miyoo-mini-plus",
      extrasPlatform: "/Tools",
      version: "2025.01.01",
    });

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.code).toBe("DOWNLOAD_ERROR");
    }
  });

  it("returns extraction error code on extraction failure", async () => {
    const mockResult: InstallResult = {
      success: false,
      error: "Base extraction failed: invalid zip",
      base_files_copied: 0,
      extras_files_copied: 0,
      extras_warning: null,
      rom_dirs_created: 0,
    };

    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockResolvedValue(mockResult);

    const result = await installMinui({
      baseUrl: "https://example.com/base.zip",
      sdMount: "/Volumes/SDCARD",
      platform: "miyoo-mini-plus",
      extrasPlatform: "/Tools",
      version: "2025.01.01",
    });

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.code).toBe("EXTRACTION_ERROR");
    }
  });

  it("returns unknown error on invoke exception", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockRejectedValue(new Error("IPC error"));

    const result = await installMinui({
      baseUrl: "https://example.com/base.zip",
      sdMount: "/Volumes/SDCARD",
      platform: "miyoo-mini-plus",
      extrasPlatform: "/Tools",
      version: "2025.01.01",
    });

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.code).toBe("UNKNOWN_ERROR");
      expect(result.error.message).toBe("IPC error");
    }
  });

  it("returns success with zero extras when no extras URL provided", async () => {
    const mockResult: InstallResult = {
      success: true,
      error: null,
      base_files_copied: 12,
      extras_files_copied: 0,
      extras_warning: null,
      rom_dirs_created: 0,
    };

    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockResolvedValue(mockResult);

    const result = await installMinui({
      baseUrl: "https://example.com/base.zip",
      sdMount: "/Volumes/SDCARD",
      platform: "trimui-brick",
      extrasPlatform: "/Tools",
      version: "2025.01.01",
    });

    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.base_files_copied).toBe(12);
      expect(result.data.extras_files_copied).toBe(0);
    }
  });

  it("passes extras URL and checksums when provided", async () => {
    const mockResult: InstallResult = {
      success: true,
      error: null,
      base_files_copied: 10,
      extras_files_copied: 5,
      extras_warning: null,
      rom_dirs_created: 0,
    };

    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockResolvedValue(mockResult);

    const result = await installMinui({
      baseUrl: "https://example.com/base.zip",
      extrasUrl: "https://example.com/extras.zip",
      baseChecksum: "abc123",
      extrasChecksum: "def456",
      sdMount: "/Volumes/SDCARD",
      platform: "miyoo-mini-plus",
      extrasPlatform: "/Tools",
      version: "2025.01.01",
    });

    expect(result.success).toBe(true);
    expect(invoke).toHaveBeenCalledWith("install_minui", {
      baseUrl: "https://example.com/base.zip",
      extrasUrl: "https://example.com/extras.zip",
      baseChecksum: "abc123",
      extrasChecksum: "def456",
      sdMount: "/Volumes/SDCARD",
      platform: "miyoo-mini-plus",
      extrasPlatform: "/Tools",
      version: "2025.01.01",
      forkName: null,
    });
  });

  it("passes forkName when provided", async () => {
    const mockResult: InstallResult = {
      success: true,
      error: null,
      base_files_copied: 10,
      extras_files_copied: 5,
      extras_warning: null,
      rom_dirs_created: 0,
    };

    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockResolvedValue(mockResult);

    const result = await installMinui({
      baseUrl: "https://example.com/base.zip",
      sdMount: "/Volumes/SDCARD",
      platform: "miyoo-mini-plus",
      extrasPlatform: "/Tools",
      version: "20250525",
      forkName: "MinUI-Zero",
    });

    expect(result.success).toBe(true);
    expect(invoke).toHaveBeenCalledWith("install_minui", {
      baseUrl: "https://example.com/base.zip",
      extrasUrl: null,
      baseChecksum: null,
      extrasChecksum: null,
      sdMount: "/Volumes/SDCARD",
      platform: "miyoo-mini-plus",
      extrasPlatform: "/Tools",
      version: "20250525",
      forkName: "MinUI-Zero",
    });
  });
});
