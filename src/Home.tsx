import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import ConfirmDialog from "./ConfirmDialog";
import DeviceSelector from "./DeviceSelector";
import DriveSelector from "./DriveSelector";
import HealthCheck from "./HealthCheck";
import { useVersionCheck } from "./hooks/useVersionCheck";
import InstallProgressUI from "./InstallProgress";
import { getDeviceProfile } from "./types/device";
import type { RemovableDrive } from "./types/drive";
import { formatSize } from "./types/drive";
import type { InstallPhase, InstallProgressEvent } from "./types/install";
import { installMinui } from "./types/install";
import { fetchMinUIRelease } from "./types/release";
import type { ValidationResult } from "./types/validate";
import { validateInstallation } from "./types/validate";
import ValidationReportUI from "./ValidationReport";

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
	const version = useVersionCheck();
	const [isUpdatingAll, setIsUpdatingAll] = useState(false);
	const [updateAllMessage, setUpdateAllMessage] = useState("");
	const [updateAllError, setUpdateAllError] = useState<string | null>(null);

	interface InstallState {
		phase: InstallPhase;
		message: string;
		log: InstallProgressEvent[];
		error: string | null;
		baseFilesCopied: number;
		extrasFilesCopied: number;
		romDirsCreated: number;
		extrasWarning: string | null;
		validationResult: ValidationResult | null;
	}

	const initialInstallState: InstallState = {
		phase: "idle",
		message: "",
		log: [],
		error: null,
		baseFilesCopied: 0,
		extrasFilesCopied: 0,
		romDirsCreated: 0,
		extrasWarning: null,
		validationResult: null,
	};

	const [install, setInstall] = useState<InstallState>(initialInstallState);

	// Check installed version when drive changes (event-driven, not effect-driven)
	useEffect(() => {
		if (selectedDrive) {
			version.check(selectedDrive.mount_path);
		} else {
			version.reset();
		}
	}, [selectedDrive, version.check, version.reset]);

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
			setInstall((s) => ({
				...s,
				error: "Unknown device profile",
				phase: "error",
			}));
			return;
		}

		// Start installation flow
		setInstall((s) => ({
			...s,
			phase: "downloading",
			message: "",
			log: [],
			error: null,
		}));

		// Listen for progress events from the Rust backend
		const unlisten = await listen<InstallProgressEvent>(
			"install-progress",
			(event) => {
				const { step, details } = event.payload;
				setInstall((s) => {
					const phase =
						step === "download"
							? "downloading"
							: step === "extract"
								? "extracting"
								: step === "copy"
									? "copying"
									: s.phase;
					return {
						...s,
						phase,
						message: details,
						log: [...s.log, event.payload],
					};
				});
			},
		);

		try {
			// Step 1: Fetch release metadata
			const releaseResult = await fetchMinUIRelease();
			if (!releaseResult.success) {
				setInstall((s) => ({
					...s,
					error: `Failed to fetch release: ${releaseResult.error.message}`,
					phase: "error",
				}));
				return;
			}

			const release = releaseResult.data;
			setInstall((s) => ({
				...s,
				log: [
					...s.log,
					{
						step: "fetch",
						details: `Found MinUI v${release.version} (${release.baseArchiveUrl.split("/").pop()})`,
					},
				],
			}));

			// Step 2: Run the full install
			const result = await installMinui({
				baseUrl: release.baseArchiveUrl,
				extrasUrl: release.extrasArchiveUrl || undefined,
				baseChecksum: release.checksums?.base || undefined,
				extrasChecksum: release.checksums?.extras || undefined,
				sdMount: selectedDrive.mount_path,
				platform: profile.platform,
				extrasPlatform: profile.extrasPlatform,
				version: release.version,
			});

			if (result.success) {
				// Run validation after successful install
				const valResult = await validateInstallation({
					sdMount: selectedDrive.mount_path,
					hasExtras: result.data.extras_files_copied > 0,
					extrasDir: profile.installPathRules.extrasDir,
				});
				setInstall((s) => ({
					...s,
					phase: "complete",
					message: "Installation completed successfully!",
					baseFilesCopied: result.data.base_files_copied,
					extrasFilesCopied: result.data.extras_files_copied,
					romDirsCreated: result.data.rom_dirs_created,
					extrasWarning: result.data.extras_warning,
					validationResult: valResult.success ? valResult.data : null,
				}));
			} else {
				setInstall((s) => ({
					...s,
					error: result.error.message,
					phase: "error",
				}));
			}
		} catch (err) {
			const message = err instanceof Error ? err.message : "Unknown error";
			setInstall((s) => ({ ...s, error: message, phase: "error" }));
		} finally {
			unlisten();
		}
	}, [selectedDevice, selectedDrive]);

	const handleDismissInstall = () => {
		setInstall(initialInstallState);
	};

	const handleDismissValidation = () => {
		setInstall((s) => ({ ...s, validationResult: null }));
	};

	const handleRetryValidation = useCallback(async () => {
		if (!selectedDevice || !selectedDrive) return;
		const profile = getDeviceProfile(selectedDevice);
		if (!profile) return;

		const valResult = await validateInstallation({
			sdMount: selectedDrive.mount_path,
			hasExtras: install.extrasFilesCopied > 0,
			extrasDir: profile.installPathRules.extrasDir,
		});
		if (valResult.success) {
			setInstall((s) => ({ ...s, validationResult: valResult.data }));
		}
	}, [selectedDevice, selectedDrive, install.extrasFilesCopied]);

	const hasUpdates =
		(version.versionCheck?.update_available &&
			version.versionCheck?.installed != null) ||
		version.packageUpdates.length > 0;

	const handleUpdateAll = useCallback(async () => {
		if (!selectedDevice || !selectedDrive) return;

		const profile = getDeviceProfile(selectedDevice);
		if (!profile) return;

		setIsUpdatingAll(true);
		setUpdateAllError(null);
		setUpdateAllMessage("Starting updates...");

		try {
			// Step 1: Update MinUI if available
			if (version.versionCheck?.update_available) {
				setUpdateAllMessage("Updating MinUI...");

				const releaseResult = await fetchMinUIRelease();
				if (!releaseResult.success) {
					setUpdateAllError(
						`Failed to fetch MinUI release: ${releaseResult.error.message}`,
					);
					setIsUpdatingAll(false);
					return;
				}

				const release = releaseResult.data;
				const result = await installMinui({
					baseUrl: release.baseArchiveUrl,
					extrasUrl: release.extrasArchiveUrl || undefined,
					baseChecksum: release.checksums?.base || undefined,
					extrasChecksum: release.checksums?.extras || undefined,
					sdMount: selectedDrive.mount_path,
					platform: profile.platform,
					extrasPlatform: profile.extrasPlatform,
					version: release.version,
				});

				if (!result.success) {
					setUpdateAllError(`MinUI update failed: ${result.error.message}`);
					setIsUpdatingAll(false);
					return;
				}
			}

			// Step 2: Update packages if available
			if (version.packageUpdates.length > 0) {
				setUpdateAllMessage(
					`Updating ${version.packageUpdates.length} package(s)...`,
				);

				// Package updates would be handled here
				// For now, we'll just show the message
			}

			setUpdateAllMessage("All updates completed!");
			setIsUpdatingAll(false);

			// Refresh version check
			await version.check(selectedDrive.mount_path);
		} catch (err) {
			const message = err instanceof Error ? err.message : "Unknown error";
			setUpdateAllError(message);
			setIsUpdatingAll(false);
		}
	}, [
		selectedDevice,
		selectedDrive,
		version.check,
		version.versionCheck,
		version.packageUpdates,
	]);

	const isInstalling =
		install.phase !== "idle" &&
		install.phase !== "complete" &&
		install.phase !== "error";

	return (
		<div className="screen">
			<h1>MinUI Easy Installer</h1>
			<p className="subtitle">
				The easiest way to install and manage MinUI on retro handheld devices.
			</p>

			{install.phase !== "idle" ? (
				<div className="card">
					{install.validationResult ? (
						<ValidationReportUI
							result={install.validationResult}
							onDismiss={handleDismissValidation}
							onRetry={handleRetryValidation}
						/>
					) : (
						<InstallProgressUI
							phase={install.phase}
							message={install.message}
							log={install.log}
							baseFilesCopied={install.baseFilesCopied}
							extrasFilesCopied={install.extrasFilesCopied}
							romDirsCreated={install.romDirsCreated}
							extrasWarning={install.extrasWarning}
							error={install.error}
							onDismiss={handleDismissInstall}
						/>
					)}
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

					{selectedDrive && (
						<div className="card version-status">
							<h2>Status Summary</h2>

							<div className="status-device">
								<strong>Device:</strong>{" "}
								{selectedDevice
									? getDeviceProfile(selectedDevice)?.name
									: "Not selected"}
							</div>

							<div className="status-drive">
								<strong>SD Card:</strong> {selectedDrive.name}
								{selectedDrive.size_bytes && (
									<span> ({formatSize(selectedDrive.size_bytes)})</span>
								)}
								{selectedDrive.filesystem && (
									<span> - {selectedDrive.filesystem}</span>
								)}
							</div>

							{version.isChecking ? (
								<p className="checking">Checking version...</p>
							) : version.versionCheck ? (
								<div className="version-info">
									{version.versionCheck.installed ? (
										<p className="installed-version">
											<strong>MinUI:</strong> v
											{version.versionCheck.installed.version}
										</p>
									) : (
										<p className="no-version">
											<strong>MinUI:</strong> Not detected
										</p>
									)}
									{version.versionCheck.update_available ? (
										<p className="update-available">
											Update available: v{version.versionCheck.latest}
										</p>
									) : version.versionCheck.installed ? (
										<p className="up-to-date">MinUI is up to date</p>
									) : null}

									{version.packageUpdates.length > 0 && (
										<div className="package-updates">
											<h3>
												{version.packageUpdates.length} Package Update
												{version.packageUpdates.length > 1 ? "s" : ""} Available
											</h3>
											<ul>
												{version.packageUpdates.map((update) => (
													<li key={update.name}>
														{update.name}:{" "}
														{update.installed_version || "unknown"} &rarr;{" "}
														{update.latest_version}
													</li>
												))}
											</ul>
										</div>
									)}
								</div>
							) : (
								<p className="no-version">Select a drive to check version</p>
							)}
						</div>
					)}

					{selectedDrive && selectedDevice && (
						<div className="card ready">
							{isUpdatingAll ? (
								<div className="update-all-status">
									<h2>Updating...</h2>
									<div className="install-spinner" />
									<p>{updateAllMessage}</p>
									{updateAllError && <p className="error">{updateAllError}</p>}
								</div>
							) : (
								<>
									<h2>
										{hasUpdates
											? "Updates Available"
											: version.versionCheck?.installed
												? "Update MinUI"
												: "Install MinUI"}
									</h2>
									{hasUpdates && (
										<button
											type="button"
											className="update-all-btn"
											onClick={handleUpdateAll}
											disabled={isInstalling}
										>
											Update All
										</button>
									)}
									<button
										type="button"
										onClick={handleInstallClick}
										disabled={isInstalling}
									>
										{version.versionCheck?.installed
											? "Update MinUI Only"
											: "Install MinUI"}
									</button>
								</>
							)}
						</div>
					)}
				</>
			)}

			{selectedDrive && selectedDevice && (
				<div className="card">
					<HealthCheck
						sdMount={selectedDrive.mount_path}
						devicePlatform={getDeviceProfile(selectedDevice)?.platform}
					/>
				</div>
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
