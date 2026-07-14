import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
// @vitest-environment jsdom
import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  type Mock,
  vi,
} from "vitest";
import BiosInstaller from "./BiosInstaller";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const CATALOG = [
  {
    id: "gb_bios",
    subdir: "GB",
    filename: "gb_bios.bin",
    description: "Game Boy boot logo",
    system: "Game Boy",
  },
  {
    id: "sgb_bios",
    subdir: "",
    filename: "sgb.bios",
    description: "Super Game Boy",
    system: "Super Game Boy",
  },
];

function mockInvoke(
  invoke: Mock,
  overrides: {
    catalog?: unknown;
    status?: unknown;
  } = {},
) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd === "list_bios_catalog") {
      return Promise.resolve(overrides.catalog ?? CATALOG);
    }
    if (cmd === "get_bios_status") {
      const catalog = (overrides.catalog ?? CATALOG) as Array<{
        id: string;
        subdir: string;
        filename: string;
        description: string;
        system: string;
      }>;
      return Promise.resolve(
        overrides.status ?? {
          entries: catalog.map((entry) => ({
            entry,
            present: false,
          })),
          installed_count: 0,
        },
      );
    }
    if (cmd === "install_bios_file")
      return Promise.resolve("/sd/Bios/GB/gb_bios.bin");
    return Promise.resolve(null);
  });
}

describe("BiosInstaller", () => {
  afterEach(() => {
    cleanup();
  });

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads and renders the catalog on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mockInvoke(invoke as Mock);

    render(<BiosInstaller sdMount="/Volumes/SD" onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText(/Game Boy boot logo/)).toBeInTheDocument();
    });
    expect(screen.getAllByText("Bios/GB/gb_bios.bin").length).toBeGreaterThan(
      0,
    );
    expect(screen.getAllByText("Bios/sgb.bios").length).toBeGreaterThan(0);
  });

  it("shows installed count from the status payload", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mockInvoke(invoke as Mock, {
      status: {
        entries: [
          {
            entry: {
              id: "gb_bios",
              subdir: "GB",
              filename: "gb_bios.bin",
              description: "Game Boy boot logo",
              system: "Game Boy",
            },
            present: true,
          },
          {
            entry: {
              id: "sgb_bios",
              subdir: "",
              filename: "sgb.bios",
              description: "Super Game Boy",
              system: "Super Game Boy",
            },
            present: false,
          },
        ],
        installed_count: 1,
      },
    });

    render(<BiosInstaller sdMount="/Volumes/SD" onClose={vi.fn()} />);

    await waitFor(() => {
      expect(
        screen.getByText(/1 of 2 BIOS files installed/),
      ).toBeInTheDocument();
    });
  });

  it("renders an error with a Retry button when load fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockRejectedValue(new Error("SD card missing"));

    render(<BiosInstaller sdMount="/Volumes/SD" onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("SD card missing")).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
  });

  it("calls onClose when Back is clicked", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mockInvoke(invoke as Mock);
    const onClose = vi.fn();

    render(<BiosInstaller sdMount="/Volumes/SD" onClose={onClose} />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Back" })).toBeInTheDocument();
    });

    await userEvent.click(screen.getByRole("button", { name: "Back" }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("invokes install_bios_file when a file is chosen", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mockInvoke(invoke as Mock);

    render(<BiosInstaller sdMount="/Volumes/SD" onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText(/Game Boy boot logo/)).toBeInTheDocument();
    });

    const file = new File(["gb boot rom"], "gb_bios.bin", {
      type: "application/octet-stream",
    });
    const input = document.querySelector(
      '[data-testid="bios-file-input-gb_bios"]',
    ) as HTMLInputElement;
    expect(input).toBeTruthy();

    await userEvent.upload(input, file);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith(
        "install_bios_file",
        expect.objectContaining({
          opts: expect.objectContaining({
            sdMount: "/Volumes/SD",
            entryId: "gb_bios",
            base64Payload: expect.any(String),
          }),
        }),
      );
    });
  });

  it("shows the source filename after a successful install", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mockInvoke(invoke as Mock);

    render(<BiosInstaller sdMount="/Volumes/SD" onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText(/Game Boy boot logo/)).toBeInTheDocument();
    });

    const file = new File(["x"], "my_gb.bin", {
      type: "application/octet-stream",
    });
    const input = document.querySelector(
      '[data-testid="bios-file-input-gb_bios"]',
    ) as HTMLInputElement;
    await userEvent.upload(input, file);

    await waitFor(() => {
      expect(screen.getByText(/Copied my_gb\.bin/)).toBeInTheDocument();
    });
  });

  it("shows an error message if install fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockImplementation((cmd: string) => {
      if (cmd === "list_bios_catalog") return Promise.resolve(CATALOG);
      if (cmd === "get_bios_status") {
        return Promise.resolve({
          entries: CATALOG.map((entry) => ({ entry, present: false })),
          installed_count: 0,
        });
      }
      if (cmd === "install_bios_file") {
        return Promise.reject(new Error("permission denied"));
      }
      return Promise.resolve(null);
    });

    render(<BiosInstaller sdMount="/Volumes/SD" onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText(/Game Boy boot logo/)).toBeInTheDocument();
    });

    const file = new File(["x"], "broken.bin", {
      type: "application/octet-stream",
    });
    const input = document.querySelector(
      '[data-testid="bios-file-input-gb_bios"]',
    ) as HTMLInputElement;
    await userEvent.upload(input, file);

    await waitFor(() => {
      expect(screen.getByText("permission denied")).toBeInTheDocument();
    });
  });
});
