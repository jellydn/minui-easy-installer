import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  INITIAL_INSTALL_STATE,
  InstallOrchestrator,
  type OrchestratorState,
} from "../lib/InstallOrchestrator";
import { FORK_PRESETS } from "../types/fork";

// Mock Tauri event listener. The orchestrator calls `listen()` from
// @tauri-apps/api/event for install-progress events.
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

// Mock IPC functions — the orchestrator calls these but we don't
// want to actually trigger Tauri IPC in tests.
vi.mock("../types/install", () => ({
  startInstallAndWait: vi.fn(),
  cancelInstall: vi.fn(),
}));

vi.mock("../types/release", () => ({
  fetchMinUIRelease: vi.fn(),
}));

vi.mock("../types/validate", () => ({
  validateInstallation: vi.fn(),
}));

vi.mock("../types/package", () => ({
  fetchPackageRegistry: vi.fn(),
  installPackage: vi.fn(),
}));

function collect(orchestrator: InstallOrchestrator): OrchestratorState[] {
  const states: OrchestratorState[] = [];
  orchestrator.subscribe((s) => states.push({ ...s }));
  return states;
}

describe("InstallOrchestrator", () => {
  let orch: InstallOrchestrator;

  beforeEach(() => {
    vi.clearAllMocks();
    orch = new InstallOrchestrator();
  });

  // ── Initial state ──────────────────────────────────────

  it("starts in the idle state", () => {
    expect(orch.install.phase).toBe("idle");
    expect(orch.install.error).toBeNull();
    expect(orch.isInstalling).toBe(false);
  });

  it("subscribe emits initial state synchronously", () => {
    const states = collect(orch);
    expect(states).toHaveLength(1);
    expect(states[0].install.phase).toBe("idle");
  });

  // ── dismissInstall / dismissValidation ──────────────────

  it("dismissInstall resets to initial state", () => {
    // Set some non-idle state first
    (orch as any).installState = {
      ...INITIAL_INSTALL_STATE,
      phase: "error",
      error: "something went wrong",
    };

    orch.dismissInstall();

    expect(orch.install.phase).toBe("idle");
    expect(orch.install.error).toBeNull();
  });

  it("dismissInstall notifies subscribers", () => {
    const states = collect(orch);
    orch.dismissInstall();
    expect(states).toHaveLength(2);
    expect(states[1].install.phase).toBe("idle");
  });

  it("dismissValidation clears validationResult", () => {
    (orch as any).installState = {
      ...INITIAL_INSTALL_STATE,
      validationResult: { success: true } as any,
    };
    orch.dismissValidation();
    expect(orch.install.validationResult).toBeNull();
  });

  // ── cancel ─────────────────────────────────────────────

  it("cancel sets phase to error and calls cancelInstall", async () => {
    const { cancelInstall } = await import("../types/install");

    orch.cancel();

    expect(orch.install.phase).toBe("error");
    expect(orch.install.error).toBe("Installation cancelled");
    expect(cancelInstall).toHaveBeenCalled();
  });

  it("cancel notifies subscribers", () => {
    const states = collect(orch);
    orch.cancel();
    expect(states).toHaveLength(2);
    expect(states[1].install.phase).toBe("error");
  });

  // ── start (installMinUI) ───────────────────────────────

  it("start errors when device is unknown", async () => {
    await orch.start(FORK_PRESETS.official, "nonexistent-device", "/sd");
    expect(orch.install.phase).toBe("error");
    expect(orch.install.error).toBe("Unknown device profile");
  });

  it("start errors when install fails", async () => {
    const { fetchMinUIRelease } = await import("../types/release");
    const { startInstallAndWait } = await import("../types/install");

    (fetchMinUIRelease as any).mockResolvedValue({
      success: true,
      data: {
        version: "2025.01.01",
        baseArchiveUrl: "https://example.com/base.zip",
        extrasArchiveUrl: null,
        checksums: null,
        fork: FORK_PRESETS.official,
      },
    });
    (startInstallAndWait as any).mockRejectedValue(
      new Error("SD card full"),
    );

    await orch.start(
      FORK_PRESETS.official,
      "miyoo-mini-plus",
      "/sd",
    );

    expect(orch.install.phase).toBe("error");
    expect(orch.install.error).toMatch(/SD card full/);
  });

  it("start completes successfully with validation", async () => {
    const { fetchMinUIRelease } = await import("../types/release");
    const { startInstallAndWait } = await import("../types/install");
    const { validateInstallation } = await import("../types/validate");

    (fetchMinUIRelease as any).mockResolvedValue({
      success: true,
      data: {
        version: "2025.01.01",
        baseArchiveUrl: "https://example.com/base.zip",
        extrasArchiveUrl: null,
        checksums: null,
        fork: FORK_PRESETS.official,
      },
    });
    (startInstallAndWait as any).mockResolvedValue({
      success: true,
      error: null,
      base_files_copied: 3,
      extras_files_copied: 0,
      extras_warning: null,
      rom_dirs_created: 5,
    });
    (validateInstallation as any).mockResolvedValue({
      success: true,
      data: { success: true, checks: [], passed_count: 0, failed_count: 0 },
    });

    await orch.start(
      FORK_PRESETS.official,
      "miyoo-mini-plus",
      "/sd",
    );

    expect(orch.install.phase).toBe("complete");
    expect(orch.install.baseFilesCopied).toBe(3);
    expect(orch.install.romDirsCreated).toBe(5);
    expect(orch.install.validationResult).toBeDefined();
  });

  // ── isInstalling ───────────────────────────────────────

  it("isInstalling is true during downloading phase", () => {
    (orch as any).installState = {
      ...INITIAL_INSTALL_STATE,
      phase: "downloading",
    };
    expect(orch.isInstalling).toBe(true);
  });

  it("isInstalling is false when idle, complete, or error", () => {
    expect(orch.isInstalling).toBe(false);
    (orch as any).installState = {
      ...INITIAL_INSTALL_STATE,
      phase: "complete",
    };
    expect(orch.isInstalling).toBe(false);
    (orch as any).installState = { ...INITIAL_INSTALL_STATE, phase: "error" };
    expect(orch.isInstalling).toBe(false);
  });
});
