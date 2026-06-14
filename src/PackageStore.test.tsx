import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, type Mock, vi } from "vitest";
import PackageStore from "./PackageStore";
import type { PackageRegistry } from "./types/package";

vi.mock("./types/package", async () => {
  const actual = await import("./types/package");
  return {
    ...actual,
    fetchPackageRegistry: vi.fn(),
    installPackage: vi.fn(),
  };
});

const mockRegistry: PackageRegistry = {
  version: "1.0",
  packages: [
    {
      name: "Wifi.pak",
      version: "1.0.0",
      category: "Emulators",
      description: "WiFi connectivity tool",
      repository: "https://github.com/example/wifi",
      downloads: 1000,
      rating: 4.5,
      artifactUrl: "https://example.com/wifi.pak",
      checksum: null,
      supportedDevices: ["miyoo-mini-plus"],
      installPathRules: { targetDir: "/Tools", extractToRoot: false },
    },
    {
      name: "SSH.pak",
      version: "2.0.0",
      category: "Utilities",
      description: "SSH remote access",
      repository: "https://github.com/example/ssh",
      downloads: 500,
      rating: null,
      artifactUrl: "https://example.com/ssh.pak",
      checksum: null,
      supportedDevices: [],
      installPathRules: { targetDir: "/Tools", extractToRoot: false },
    },
  ],
};

describe("PackageStore", () => {
  afterEach(() => {
    cleanup();
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows loading state while fetching registry", async () => {
    const { fetchPackageRegistry } = await import("./types/package");
    (fetchPackageRegistry as Mock).mockReturnValue(new Promise(() => {}));

    render(
      <PackageStore
        selectedDevice="miyoo-mini-plus"
        selectedDrive="/Volumes/SD"
      />,
    );

    expect(screen.getByText("Loading packages...")).toBeInTheDocument();
  });

  it("displays package cards after loading", async () => {
    const { fetchPackageRegistry } = await import("./types/package");
    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: true,
      data: mockRegistry,
    });

    render(
      <PackageStore
        selectedDevice="miyoo-mini-plus"
        selectedDrive="/Volumes/SD"
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Wifi.pak")).toBeInTheDocument();
    });
    expect(screen.getByText("SSH.pak")).toBeInTheDocument();
    expect(screen.getByText("WiFi connectivity tool")).toBeInTheDocument();
  });

  it("shows error state with retry on fetch failure", async () => {
    const { fetchPackageRegistry } = await import("./types/package");
    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: false,
      error: { message: "Network error", code: "NETWORK_ERROR" },
    });

    render(<PackageStore selectedDevice={null} selectedDrive={null} />);

    await waitFor(() => {
      expect(screen.getByText(/Failed to load packages/)).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
  });

  it("filters packages by search query", async () => {
    const { fetchPackageRegistry } = await import("./types/package");
    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: true,
      data: mockRegistry,
    });

    render(
      <PackageStore
        selectedDevice="miyoo-mini-plus"
        selectedDrive="/Volumes/SD"
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Wifi.pak")).toBeInTheDocument();
    });

    const searchInput = screen.getByPlaceholderText("Search packages...");
    await userEvent.type(searchInput, "SSH");

    expect(screen.queryByText("Wifi.pak")).not.toBeInTheDocument();
    expect(screen.getByText("SSH.pak")).toBeInTheDocument();
  });

  it("shows empty state when search has no results", async () => {
    const { fetchPackageRegistry } = await import("./types/package");
    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: true,
      data: mockRegistry,
    });

    render(
      <PackageStore
        selectedDevice="miyoo-mini-plus"
        selectedDrive="/Volumes/SD"
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Wifi.pak")).toBeInTheDocument();
    });

    const searchInput = screen.getByPlaceholderText("Search packages...");
    await userEvent.type(searchInput, "nonexistent");

    expect(
      screen.getByText(/No packages found matching "nonexistent"/),
    ).toBeInTheDocument();
  });

  it("filters by category", async () => {
    const { fetchPackageRegistry } = await import("./types/package");
    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: true,
      data: mockRegistry,
    });

    render(
      <PackageStore
        selectedDevice="miyoo-mini-plus"
        selectedDrive="/Volumes/SD"
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Wifi.pak")).toBeInTheDocument();
    });

    await userEvent.click(screen.getByRole("button", { name: "Network" }));

    expect(screen.getByText("Wifi.pak")).toBeInTheDocument();
    expect(screen.queryByText("SSH.pak")).not.toBeInTheDocument();
  });
});
