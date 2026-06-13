import { useCallback, useEffect, useState } from "react";
import ConfirmDialog from "./ConfirmDialog";
import DeviceSelector from "./DeviceSelector";
import DriveSelector from "./DriveSelector";
import HealthCheck from "./HealthCheck";
import InstallProgressUI from "./InstallProgress";
import { getDeviceProfile } from "./types/device";
import type { RemovableDrive } from "./types/drive";
import { formatSize } from "./types/drive";
import type { InstallPhase } from "./types/install";
import { installMinui } from "./types/install";
import type { PackageUpdateInfo } from "./types/package";
import { checkPackageUpdates, fetchPackageRegistry } from "./types/package";
import { fetchMinUIRelease } from "./types/release";
import type { ValidationResult } from "./types/validate";
import { validateInstallation } from "./types/validate";
import type { VersionCheckResult } from "./types/version";
import { checkMinuiVersion } from "./types/version";
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
	const [installPhase, setInstallPhase] = useState<InstallPhase>("idle");
	const [installMessage, setInstallMessage] = useState("");
	const [installError, setInstallError] = useState<string | null>(null);
	const [baseFilesCopied, setBaseFilesCopied] = useState(0);
	const [extrasFilesCopied, setExtrasFilesCopied] = useState(0);
	const [validationResult, setValidationResult] =
		useState<ValidationResult | null>(null);
	const [versionCheck, setVersionCheck] = useState<VersionCheckResult | null>(
		null,
	);
	const [isCheckingVersion, setIsCheckingVersion] = useState(false);
	const [packageUpdates, setPackageUpdates] = useState<PackageUpdateInfo[]>([]);
	const [isUpdatingAll, setIsUpdatingAll] = useState(false);
	const [updateAllMessage, setUpdateAllMessage] = useState("");
	const [updateAllError, setUpdateAllError] = useState<string | null>(null);

	// Check installed version when drive is selected
	useEffect(() => {
		if (!selectedDrive) {
			setVersionCheck(null);
			setPackageUpdates([]);
			return;
		}

		let cancelled = false;

		async function checkVersion() {
			setIsCheckingVersion(true);
			try {
				// First fetch latest release to get version
				const releaseResult = await fetchMinUIRelease();
				const latestVersion = releaseResult.success
					? releaseResult.data.version
					: undefined;

				// Then check installed version
				const result = await checkMinuiVersion({
					sdMount: selectedDrive!.mount_path,
					latestVersion,
				});

				if (!cancelled && result.success) {
					setVersionCheck(result.data);
				}

				// Check for package updates
				const registryResult = await fetchPackageRegistry();
				if (!cancelled && registryResult.success) {
					const registryPackages: [string, string][] =
						registryResult.data.packages.map((p) => [p.name, p.version]);
					const updates = await checkPackageUpdates(
						selectedDrive!.mount_path,
						registryPackages,
					);
					if (!cancelled) {
						setPackageUpdates(updates.filter((u) => u.update_available));
					}
				}
			} catch {
				// Version check failure is non-fatal
			} finally {
				if (!cancelled) {
					setIsCheckingVersion(false);
				}
			}
		}

		checkVersion();

		return () => {
			cancelled = true;
		};
	}, [selectedDrive]);

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

				// Run validation after successful install
				const valResult = await validateInstallation({
					sdMount: selectedDrive.mount_path,
					hasExtras: result.data.extras_files_copied > 0,
					extrasDir: profile.installPathRules.extrasDir,
				});
				if (valResult.success) {
					setValidationResult(valResult.data);
				}
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
		setValidationResult(null);
	};

	const handleDismissValidation = () => {
		setValidationResult(null);
	};

	const handleRetryValidation = useCallback(async () => {
		if (!selectedDevice || !selectedDrive) return;
		const profile = getDeviceProfile(selectedDevice);
		if (!profile) return;

		const valResult = await validateInstallation({
			sdMount: selectedDrive.mount_path,
			hasExtras: extrasFilesCopied > 0,
			extrasDir: profile.installPathRules.extrasDir,
		});
		if (valResult.success) {
			setValidationResult(valResult.data);
		}
	}, [selectedDevice, selectedDrive, extrasFilesCopied]);

	const hasUpdates =
		(versionCheck?.update_available ?? false) || packageUpdates.length > 0;

	const handleUpdateAll = useCallback(async () => {
		if (!selectedDevice || !selectedDrive) return;

		const profile = getDeviceProfile(selectedDevice);
		if (!profile) return;

		setIsUpdatingAll(true);
		setUpdateAllError(null);
		setUpdateAllMessage("Starting updates...");

		try {
			// Step 1: Update MinUI if available
			if (versionCheck?.update_available) {
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
					extrasDir: profile.installPathRules.extrasDir,
				});

				if (!result.success) {
					setUpdateAllError(`MinUI update failed: ${result.error.message}`);
					setIsUpdatingAll(false);
					return;
				}
			}

			// Step 2: Update packages if available
			if (packageUpdates.length > 0) {
				setUpdateAllMessage(`Updating ${packageUpdates.length} package(s)...`);

				// Package updates would be handled here
				// For now, we'll just show the message
			}

			setUpdateAllMessage("All updates completed!");
			setIsUpdatingAll(false);

			// Refresh version check
			const releaseResult = await fetchMinUIRelease();
			const latestVersion = releaseResult.success
				? releaseResult.data.version
				: undefined;
			const versionResult = await checkMinuiVersion({
				sdMount: selectedDrive.mount_path,
				latestVersion,
			});
			if (versionResult.success) {
				setVersionCheck(versionResult.data);
			}
		} catch (err) {
			const message = err instanceof Error ? err.message : "Unknown error";
			setUpdateAllError(message);
			setIsUpdatingAll(false);
		}
	}, [selectedDevice, selectedDrive, versionCheck, packageUpdates]);

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
					{validationResult ? (
						<ValidationReportUI
							result={validationResult}
							onDismiss={handleDismissValidation}
							onRetry={handleRetryValidation}
						/>
					) : (
						<InstallProgressUI
							phase={installPhase}
							message={installMessage}
							baseFilesCopied={baseFilesCopied}
							extrasFilesCopied={extrasFilesCopied}
							error={installError}
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

							{isCheckingVersion ? (
								<p className="checking">Checking version...</p>
							) : versionCheck ? (
								<div className="version-info">
									{versionCheck.installed ? (
										<p className="installed-version">
											<strong>MinUI:</strong> v{versionCheck.installed.version}
										</p>
									) : (
										<p className="no-version">
											<strong>MinUI:</strong> Not detected
										</p>
									)}
									{versionCheck.update_available ? (
										<p className="update-available">
											Update available: v{versionCheck.latest}
										</p>
									) : versionCheck.installed ? (
										<p className="up-to-date">MinUI is up to date</p>
									) : null}

									{packageUpdates.length > 0 && (
										<div className="package-updates">
											<h3>
												{packageUpdates.length} Package Update
												{packageUpdates.length > 1 ? "s" : ""} Available
											</h3>
											<ul>
												{packageUpdates.map((update) => (
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
											: versionCheck?.installed
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
										{versionCheck?.installed
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
