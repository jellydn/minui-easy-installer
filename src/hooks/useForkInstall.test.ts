// @vitest-environment jsdom
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, type Mock, vi } from "vitest";
import { ForkProvider } from "../contexts/ForkContext";
import { useForkInstall } from "./useForkInstall";
import { FORK_PRESETS } from "../types/fork";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("../types/release", () => ({
  fetchMinUIRelease: vi.fn(),
}));

vi.mock("../types/install", () => ({
  startInstallAndWait: vi.fn(),
  cancelInstall: vi.fn(),
}));

vi.mock("../types/validate", () => ({
  validateInstallation: vi.fn(),
}));

vi.mock("../types/package", () => ({
  fetchPackageRegistry: vi.fn(),
  installPackage: vi.fn(),
}));

/** Render the hook inside a ForkProvider so useFork() resolves. */
function renderUseForkInstall(opts: Parameters<typeof useForkInstall>[0]) {
  return renderHook(() => useForkInstall(opts), { wrapper: ForkProvider });
}

describe("useForkInstall", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("installMinUI surfaces an error when startInstallAndWait throws", async () => {
    const { fetchMinUIRelease } = await import("../types/release");
    const { startInstallAndWait } = await import("../types/install");

    (fetchMinUIRelease as Mock).mockResolvedValue({
      success: true,
      data: {
        version: "2025.01.01",
        baseArchiveUrl: "https://example.com/base.zip",
        extrasArchiveUrl: null,
        checksums: null,
        fork: FORK_PRESETS.official,
      },
    });
    (startInstallAndWait as Mock).mockRejectedValue(new Error("SD card full"));

    const { result } = renderUseForkInstall({
      selectedDevice: "miyoo-mini-plus",
      selectedDriveMount: "/sd",
      versionCheck: null,
      packageUpdates: [],
      onAfterUpdate: () => {},
    });

    await act(async () => {
      await result.current.installMinUI();
    });

    expect(result.current.install.phase).toBe("error");
    expect(result.current.install.error).toBe(
      "MinUI (Official) install failed: SD card full",
    );
  });

  it("installMinUI extracts the message when startInstallAndWait rejects with a plain string (Tauri v2 invoke)", async () => {
    const { fetchMinUIRelease } = await import("../types/release");
    const { startInstallAndWait } = await import("../types/install");

    (fetchMinUIRelease as Mock).mockResolvedValue({
      success: true,
      data: {
        version: "2025.01.01",
        baseArchiveUrl: "https://example.com/base.zip",
        extrasArchiveUrl: null,
        checksums: null,
        fork: FORK_PRESETS.minuitsp,
      },
    });
    // Tauri v2 invoke() rejects with a plain string when a Rust command returns Err(String)
    (startInstallAndWait as Mock).mockRejectedValue(
      "Download failed with status: 404 Not Found",
    );

    const { result } = renderUseForkInstall({
      selectedDevice: "trimui-smart-pro",
      selectedDriveMount: "/sd",
      versionCheck: null,
      packageUpdates: [],
      onAfterUpdate: () => {},
    });

    await act(async () => {
      await result.current.installMinUI();
    });

    expect(result.current.install.phase).toBe("error");
    // Must surface the real error, NOT "Unknown error"
    expect(result.current.install.error).toContain("404");
    expect(result.current.install.error).not.toContain("Unknown error");
  });

  it("installMinUI surfaces the version-metadata warning via extrasWarning", async () => {
    const { fetchMinUIRelease } = await import("../types/release");
    const { startInstallAndWait } = await import("../types/install");
    const { validateInstallation } = await import("../types/validate");

    (fetchMinUIRelease as Mock).mockResolvedValue({
      success: true,
      data: {
        version: "2025.01.01",
        baseArchiveUrl: "https://example.com/base.zip",
        extrasArchiveUrl: null,
        checksums: null,
        fork: FORK_PRESETS.official,
      },
    });
    (startInstallAndWait as Mock).mockResolvedValue({
      success: true,
      error: null,
      base_files_copied: 5,
      extras_files_copied: 0,
      extras_warning: "Failed to write version metadata: permission denied",
      rom_dirs_created: 0,
    });
    (validateInstallation as Mock).mockResolvedValue({
      success: true,
      data: {
        success: true,
        checks: [],
        passed_count: 0,
        failed_count: 0,
        free_space_bytes: null,
        device_path: "miyoo354",
        multiple_device_folders_warning: null,
      },
    });

    const { result } = renderUseForkInstall({
      selectedDevice: "miyoo-mini-plus",
      selectedDriveMount: "/sd",
      versionCheck: null,
      packageUpdates: [],
      onAfterUpdate: () => {},
    });

    await act(async () => {
      await result.current.installMinUI();
    });

    expect(result.current.install.phase).toBe("complete");
    expect(result.current.install.extrasWarning).toMatch(/version metadata/);
  });

  it("dismissInstall resets to the initial state", async () => {
    const { result } = renderUseForkInstall({
      selectedDevice: null,
      selectedDriveMount: null,
      versionCheck: null,
      packageUpdates: [],
      onAfterUpdate: () => {},
    });

    act(() => {
      result.current.dismissInstall();
    });
    expect(result.current.install.phase).toBe("idle");
  });
});
