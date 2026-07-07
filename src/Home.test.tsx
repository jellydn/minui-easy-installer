import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, type Mock, vi } from "vitest";
import Home from "./Home";
import type { RemovableDrive } from "./types/drive";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("./types/release", () => ({
  fetchMinUIRelease: vi.fn(),
}));

vi.mock("./types/version", () => ({
  checkMinuiVersion: vi.fn(),
}));

vi.mock("./types/package", async () => {
  const actual = await import("./types/package");
  return {
    ...actual,
    fetchPackageRegistry: vi.fn(),
    checkPackageUpdates: vi.fn(),
  };
});

vi.mock("./types/install", async () => {
  const actual = await import("./types/install");
  return {
    ...actual,
    installMinui: vi.fn(),
  };
});

vi.mock("./types/validate", () => ({
  validateInstallation: vi.fn(),
  checkSdCardHealth: vi.fn(),
}));

const mockDrive: RemovableDrive = {
  name: "SD_CARD",
  mount_path: "/Volumes/SD_CARD",
  size_bytes: 32_000_000_000,
  filesystem: "FAT32",
  available_bytes: 28_000_000_000,
};

describe("Home", () => {
  const defaultProps = {
    selectedDevice: null as string | null,
    onSelectDevice: vi.fn(),
    selectedDrive: null as RemovableDrive | null,
    onSelectDrive: vi.fn(),
  };

  afterEach(() => {
    cleanup();
  });

  beforeEach(async () => {
    vi.clearAllMocks();
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockResolvedValue([]);
  });

  it("renders the home screen title and device selector", () => {
    render(<Home {...defaultProps} />);

    expect(screen.getByText("MinUI (Official) Easy Installer")).toBeInTheDocument();
    expect(screen.getByText("Select Your Device")).toBeInTheDocument();
  });

  it("shows install button when device and drive are selected", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockResolvedValue([mockDrive]);

    const { fetchMinUIRelease } = await import("./types/release");
    (fetchMinUIRelease as Mock).mockResolvedValue({
      success: true,
      data: {
        version: "2025.01.01",
        baseArchiveUrl: "https://example.com/base.zip",
        extrasArchiveUrl: null,
        checksums: null,
      },
    });

    const { checkMinuiVersion } = await import("./types/version");
    (checkMinuiVersion as Mock).mockResolvedValue({
      success: true,
      data: {
        installed: null,
        latest: "2025.01.01",
        update_available: false,
      },
    });

    const { fetchPackageRegistry, checkPackageUpdates } =
      await import("./types/package");
    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: true,
      data: { version: "1.0", packages: [] },
    });
    (checkPackageUpdates as Mock).mockResolvedValue([]);

    render(
      <Home
        {...defaultProps}
        selectedDevice="miyoo-mini-plus"
        selectedDrive={mockDrive}
      />,
    );

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Install MinUI (Official)" }),
      ).toBeInTheDocument();
    });
  });

  it("shows status summary when drive is selected", async () => {
    const { fetchMinUIRelease } = await import("./types/release");
    (fetchMinUIRelease as Mock).mockResolvedValue({
      success: true,
      data: {
        version: "2025.01.01",
        baseArchiveUrl: "https://example.com/base.zip",
        extrasArchiveUrl: null,
        checksums: null,
      },
    });

    const { checkMinuiVersion } = await import("./types/version");
    (checkMinuiVersion as Mock).mockResolvedValue({
      success: true,
      data: {
        installed: { version: "2024.12.25", source: "minui.txt" },
        latest: "2025.01.01",
        update_available: true,
      },
    });

    const { fetchPackageRegistry, checkPackageUpdates } =
      await import("./types/package");
    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: true,
      data: { version: "1.0", packages: [] },
    });
    (checkPackageUpdates as Mock).mockResolvedValue([]);

    render(
      <Home
        {...defaultProps}
        selectedDevice="miyoo-mini-plus"
        selectedDrive={mockDrive}
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Status Summary")).toBeInTheDocument();
    });
    expect(screen.getByText("v2024.12.25")).toBeInTheDocument();
    expect(screen.getByText(/Update available/)).toBeInTheDocument();
  });

  it("shows confirmation dialog before install", async () => {
    const { fetchMinUIRelease } = await import("./types/release");
    (fetchMinUIRelease as Mock).mockResolvedValue({
      success: true,
      data: {
        version: "2025.01.01",
        baseArchiveUrl: "https://example.com/base.zip",
        extrasArchiveUrl: null,
        checksums: null,
      },
    });

    const { checkMinuiVersion } = await import("./types/version");
    (checkMinuiVersion as Mock).mockResolvedValue({
      success: true,
      data: {
        installed: null,
        latest: "2025.01.01",
        update_available: false,
      },
    });

    const { fetchPackageRegistry, checkPackageUpdates } =
      await import("./types/package");
    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: true,
      data: { version: "1.0", packages: [] },
    });
    (checkPackageUpdates as Mock).mockResolvedValue([]);

    render(
      <Home
        {...defaultProps}
        selectedDevice="miyoo-mini-plus"
        selectedDrive={mockDrive}
      />,
    );

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Install MinUI (Official)" }),
      ).toBeInTheDocument();
    });

    await userEvent.click(
      screen.getByRole("button", { name: "Install MinUI (Official)" }),
    );

    expect(screen.getByText(/Confirm Installation/)).toBeInTheDocument();
  });
});
