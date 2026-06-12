import { useCallback, useState } from "react";
import ConfirmDialog from "./ConfirmDialog";
import DeviceSelector from "./DeviceSelector";
import DriveSelector from "./DriveSelector";
import InstallProgressUI from "./InstallProgress";
import { getDeviceProfile } from "./types/device";
import type { RemovableDrive } from "./types/drive";
import type { InstallPhase } from "./types/install";
import { installMinui } from "./types/install";
import { fetchMinUIRelease } from "./types/release";

interface HomeProps {
	selectedDevice: string | null;
	onSelectDevice: (deviceId: string | null) => void;
	selectedDrive: RemovableDrive | null;
	onSelectDrive: (drive: RemovableDrive | null) => void;
}

function Home({
	selectedDevice,
	onSelectDevice,
	selectedDrive,
	onSelectDrive,
}: HomeProps) {
	const [showConfirmDialog, setShowConfirmDialog] = useState(false);
	const [installPhase, setInstallPhase] = useState<InstallPhase>("idle");
	const [installMessage, setInstallMessage] = useState("");
	const [installError, setInstallError] = useState<string | null>(null);
	const [baseFilesCopied, setBaseFilesCopied] = useState(0);
	const [extrasFilesCopied, setExtrasFilesCopied] = useState(0);

	const handleInstallClick = () => {
		setShowConfirmDialog(true);
	};

	const handleCancelInstall = () => {
		setShowConfirmDialog(false);
	};

	const handleConfirmInstall = useCallback(async () => {
		setShowConfirmDialog(false);

		if (!selectedDevice || !selectedDrive) return;

		const profile = getDeviceProfile(selectedDevice);
		if (!profile) {
			setInstallError("Unknown device profile");
			setInstallPhase("error");
			return;
		}

		// Start installation flow
		setInstallPhase("downloading");
		setInstallMessage("Fetching latest MinUI release...");
		setInstallError(null);

		try {
			// Step 1: Fetch release metadata
			const releaseResult = await fetchMinUIRelease();
			if (!releaseResult.success) {
				setInstallError(
					`Failed to fetch release: ${releaseResult.error.message}`,
				);
				setInstallPhase("error");
				return;
			}

			const release = releaseResult.data;
			setInstallMessage(`Downloading MinUI v${release.version}...`);

			// Step 2: Run the full install
			setInstallPhase("copying");
			setInstallMessage("Installing MinUI to SD card...");

			const result = await installMinui({
				baseUrl: release.baseArchiveUrl,
				extrasUrl: release.extrasArchiveUrl || undefined,
				baseChecksum: release.checksums?.base || undefined,
				extrasChecksum: release.checksums?.extras || undefined,
				sdMount: selectedDrive.mount_path,
				platform: profile.platform,
				extrasDir: profile.installPathRules.extrasDir,
			});

			if (result.success) {
				setBaseFilesCopied(result.data.base_files_copied);
				setExtrasFilesCopied(result.data.extras_files_copied);
				setInstallPhase("complete");
				setInstallMessage("Installation completed successfully!");
			} else {
				setInstallError(result.error.message);
				setInstallPhase("error");
			}
		} catch (err) {
			const message = err instanceof Error ? err.message : "Unknown error";
			setInstallError(message);
			setInstallPhase("error");
		}
	}, [selectedDevice, selectedDrive]);

	const handleDismissInstall = () => {
		setInstallPhase("idle");
		setInstallMessage("");
		setInstallError(null);
		setBaseFilesCopied(0);
		setExtrasFilesCopied(0);
	};

	const isInstalling =
		installPhase !== "idle" &&
		installPhase !== "complete" &&
		installPhase !== "error";

	return (
		<div className="home">
			<h1>MinUI Easy Installer</h1>
			<p className="subtitle">
				The easiest way to install and manage MinUI on retro handheld devices.
			</p>

			{installPhase !== "idle" ? (
				<div className="card">
					<InstallProgressUI
						phase={installPhase}
						message={installMessage}
						baseFilesCopied={baseFilesCopied}
						extrasFilesCopied={extrasFilesCopied}
						error={installError}
						onDismiss={handleDismissInstall}
					/>
				</div>
			) : (
				<>
					<div className="card">
						<DeviceSelector
							selectedDevice={selectedDevice}
							onSelectDevice={onSelectDevice}
						/>
					</div>

					<div className="card">
						<DriveSelector
							selectedDrive={selectedDrive}
							onSelectDrive={onSelectDrive}
						/>
					</div>

					{selectedDrive && selectedDevice && (
						<div className="card ready">
							<h2>Ready to Install</h2>
							<button
								type="button"
								onClick={handleInstallClick}
								disabled={isInstalling}
							>
								Install MinUI
							</button>
						</div>
					)}
				</>
			)}

			{showConfirmDialog && selectedDevice && selectedDrive && (
				<ConfirmDialog
					selectedDevice={selectedDevice}
					selectedDrive={selectedDrive}
					onConfirm={handleConfirmInstall}
					onCancel={handleCancelInstall}
				/>
			)}
		</div>
	);
}

export default Home;
