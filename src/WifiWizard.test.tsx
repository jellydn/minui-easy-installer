import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import WifiWizard from "./WifiWizard";

vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(),
}));

function mockInvoke(
	invoke: ReturnType<typeof vi.fn>,
	overrides: Record<string, unknown> = {},
) {
	invoke.mockImplementation((cmd: string) => {
		if (cmd === "get_current_wifi_ssid") return Promise.resolve(null);
		if (cmd in overrides) return Promise.resolve(overrides[cmd]);
		return Promise.resolve([]);
	});
}

describe("WifiWizard", () => {
	afterEach(() => {
		cleanup();
	});

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("renders SSID and password fields", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		mockInvoke(vi.mocked(invoke));

		render(
			<WifiWizard
				sdMount="/Volumes/SD"
				onComplete={vi.fn()}
				onCancel={vi.fn()}
			/>,
		);

		expect(screen.getByLabelText("Network Name (SSID)")).toBeInTheDocument();
		expect(screen.getByLabelText("Password")).toBeInTheDocument();
		expect(screen.getByLabelText("Password")).toHaveAttribute(
			"type",
			"password",
		);
	});

	it("falls back to manual SSID entry when scan fails", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		vi.mocked(invoke).mockRejectedValue(new Error("scan unavailable"));

		render(
			<WifiWizard
				sdMount="/Volumes/SD"
				onComplete={vi.fn()}
				onCancel={vi.fn()}
			/>,
		);

		await waitFor(() => {
			expect(
				screen.getByText(/Could not scan for networks/),
			).toBeInTheDocument();
		});
	});

	it("shows scanned networks in dropdown when scan succeeds", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		mockInvoke(vi.mocked(invoke), {
			scan_wifi_networks: ["HomeNetwork", "GuestWiFi"],
		});

		render(
			<WifiWizard
				sdMount="/Volumes/SD"
				onComplete={vi.fn()}
				onCancel={vi.fn()}
			/>,
		);

		await waitFor(() => {
			expect(screen.getByText("HomeNetwork")).toBeInTheDocument();
		});
		expect(screen.getByText("GuestWiFi")).toBeInTheDocument();
	});

	it("disables save button when SSID is empty", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		mockInvoke(vi.mocked(invoke));

		render(
			<WifiWizard
				sdMount="/Volumes/SD"
				onComplete={vi.fn()}
				onCancel={vi.fn()}
			/>,
		);

		await waitFor(() => {
			expect(screen.getByLabelText("Password")).toBeInTheDocument();
		});

		const saveBtn = screen.getByRole("button", { name: "Save WiFi Config" });
		expect(saveBtn).toBeDisabled();

		await userEvent.type(
			screen.getByPlaceholderText("Enter WiFi network name"),
			"MyNetwork",
		);
		expect(saveBtn).not.toBeDisabled();
	});

	it("calls onCancel when cancel is clicked", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		mockInvoke(vi.mocked(invoke));
		const onCancel = vi.fn();

		render(
			<WifiWizard
				sdMount="/Volumes/SD"
				onComplete={vi.fn()}
				onCancel={onCancel}
			/>,
		);

		await waitFor(() => {
			expect(
				screen.getByRole("button", { name: "Cancel" }),
			).toBeInTheDocument();
		});

		await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
		expect(onCancel).toHaveBeenCalled();
	});

	it("writes wifi config on save", async () => {
		const { invoke } = await import("@tauri-apps/api/core");
		mockInvoke(vi.mocked(invoke));

		render(
			<WifiWizard
				sdMount="/Volumes/SD"
				onComplete={vi.fn()}
				onCancel={vi.fn()}
			/>,
		);

		await waitFor(() => {
			expect(screen.getByLabelText("Password")).toBeInTheDocument();
		});

		await userEvent.type(
			screen.getByPlaceholderText("Enter WiFi network name"),
			"MyNetwork",
		);
		await userEvent.type(screen.getByLabelText("Password"), "secret123");
		await userEvent.click(
			screen.getByRole("button", { name: "Save WiFi Config" }),
		);

		await waitFor(() => {
			expect(invoke).toHaveBeenCalledWith("write_wifi_config", {
				sdMount: "/Volumes/SD",
				ssid: "MyNetwork",
				password: "secret123",
			});
		});
	});
});
