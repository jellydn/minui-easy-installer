import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import DriveSelector from "./DriveSelector";
import type { RemovableDrive } from "./types/drive";

vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(),
}));

const mockDrive: RemovableDrive = {
	name: "SD_CARD",
	mount_path: "/Volumes/SD_CARD",
	size_bytes: 32_000_000_000,
	filesystem: "FAT32",
	available_bytes: 28_000_000_000,
};

describe("DriveSelector", () => {
	afterEach(() => {
		cleanup();
	});

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("shows empty state when no drives are detected", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue([]);

		render(<DriveSelector selectedDrive={null} onSelectDrive={vi.fn()} />);

		await waitFor(() => {
			expect(
				screen.getByText(/No removable drives detected/),
			).toBeInTheDocument();
		});
	});

	it("lists detected drives and calls onSelectDrive when clicked", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue([mockDrive]);
		const onSelectDrive = vi.fn();

		render(
			<DriveSelector selectedDrive={null} onSelectDrive={onSelectDrive} />,
		);

		await waitFor(() => {
			expect(screen.getByText("SD_CARD")).toBeInTheDocument();
		});

		await userEvent.click(screen.getByText("SD_CARD"));
		expect(onSelectDrive).toHaveBeenCalledWith(mockDrive);
	});

	it("shows selected drive details", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockResolvedValue([mockDrive]);

		render(<DriveSelector selectedDrive={mockDrive} onSelectDrive={vi.fn()} />);

		await waitFor(() => {
			expect(screen.getByText(/Selected:/)).toBeInTheDocument();
		});
		expect(screen.getByText(/Mount: \/Volumes\/SD_CARD/)).toBeInTheDocument();
	});

	it("shows error when drive detection fails", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockRejectedValue(new Error("diskutil failed"));

		render(<DriveSelector selectedDrive={null} onSelectDrive={vi.fn()} />);

		await waitFor(() => {
			expect(screen.getByText(/Error:/)).toBeInTheDocument();
		});
	});

	it("handles drives with missing filesystem", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		const driveNoFs: RemovableDrive = {
			...mockDrive,
			filesystem: null,
			size_bytes: null,
		};
		vi.mocked(invoke).mockResolvedValue([driveNoFs]);

		render(<DriveSelector selectedDrive={null} onSelectDrive={vi.fn()} />);

		await waitFor(() => {
			expect(screen.getByText("SD_CARD")).toBeInTheDocument();
		});
		expect(screen.getByText(/Unknown FS/)).toBeInTheDocument();
	});
});
