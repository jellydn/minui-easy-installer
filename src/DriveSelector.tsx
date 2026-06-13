import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { RemovableDrive } from "./types/drive";
import { formatSize, getDriveDisplayName } from "./types/drive";

interface DriveSelectorProps {
	selectedDrive: RemovableDrive | null;
	onSelectDrive: (drive: RemovableDrive) => void;
}

function DriveSelector({ selectedDrive, onSelectDrive }: DriveSelectorProps) {
	const [drives, setDrives] = useState<RemovableDrive[]>([]);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [showFormatConfirm, setShowFormatConfirm] = useState(false);
	const [isFormatting, setIsFormatting] = useState(false);
	const [formatError, setFormatError] = useState<string | null>(null);
	const [formatSuccess, setFormatSuccess] = useState(false);

	const fetchDrives = async () => {
		setLoading(true);
		setError(null);
		try {
			const result = await invoke<RemovableDrive[]>("get_removable_drives");
			setDrives(result);
		} catch (err) {
			setError(String(err));
		} finally {
			setLoading(false);
		}
	};

	useEffect(() => {
		fetchDrives();
	}, []);

	const handleFormat = async () => {
		if (!selectedDrive) return;

		setIsFormatting(true);
		setFormatError(null);

		try {
			await invoke("format_drive", {
				mountPath: selectedDrive.mount_path,
				volumeName: selectedDrive.name,
			});
			setFormatSuccess(true);
			setShowFormatConfirm(false);

			// Refresh drives after format and propagate updated info to parent
			const updatedDrives = await invoke<RemovableDrive[]>(
				"get_removable_drives",
			);
			setDrives(updatedDrives);

			// Find the updated drive and notify parent so Status Summary refreshes
			const updatedDrive = updatedDrives.find(
				(d) => d.mount_path === selectedDrive.mount_path,
			);
			if (updatedDrive) {
				onSelectDrive(updatedDrive);
			}
		} catch (err) {
			const message = err instanceof Error ? err.message : String(err);
			setFormatError(message);
		} finally {
			setIsFormatting(false);
		}
	};

	const isNotFat32 =
		selectedDrive?.filesystem &&
		!selectedDrive.filesystem.toUpperCase().includes("FAT32") &&
		!selectedDrive.filesystem.toUpperCase().includes("MS-DOS");

	return (
		<div className="drive-selector">
			<h2>Select SD Card</h2>
			<button type="button" onClick={fetchDrives} disabled={loading}>
				{loading ? "Scanning..." : "Refresh Drives"}
			</button>

			{error && <p className="error">Error: {error}</p>}

			{!loading && drives.length === 0 && !error && (
				<p className="empty-state">
					No removable drives detected. Insert an SD card and click Refresh.
				</p>
			)}

			{drives.length > 0 && (
				<ul className="drive-list">
					{drives.map((drive) => (
						<li
							key={drive.mount_path}
							className={`drive-item ${selectedDrive?.mount_path === drive.mount_path ? "selected" : ""}`}
						>
							<button type="button" onClick={() => onSelectDrive(drive)}>
								<span className="drive-name">{drive.name}</span>
								<span className="drive-details">
									{formatSize(drive.size_bytes)} |{" "}
									{drive.filesystem || "Unknown FS"} | {drive.mount_path}
								</span>
							</button>
						</li>
					))}
				</ul>
			)}

			{selectedDrive && (
				<div className="selected-drive">
					<p>Selected: {getDriveDisplayName(selectedDrive)}</p>
					<p>Mount: {selectedDrive.mount_path}</p>
					<p>Free: {formatSize(selectedDrive.available_bytes)}</p>

					{loading ? (
						<div className="drive-refreshing">
							<div className="install-spinner" />
							<p>Refreshing drive info...</p>
						</div>
					) : (
						<div className="format-section">
							{isNotFat32 && (
								<p className="format-warning">
									This drive is not formatted as FAT32. MinUI requires a FAT32
									filesystem.
								</p>
							)}
							<button
								type="button"
								className="format-btn"
								onClick={() => setShowFormatConfirm(true)}
								disabled={isFormatting}
							>
								{isFormatting ? "Formatting..." : "Format to FAT32"}
							</button>
							{formatSuccess && (
								<p className="success-message">Drive formatted successfully!</p>
							)}
						</div>
					)}
				</div>
			)}

			{showFormatConfirm && selectedDrive && (
				<div className="confirm-overlay">
					<div className="confirm-dialog">
						<h2>Format Drive?</h2>
						<p className="confirm-warning">
							This will <strong>erase all data</strong> on{" "}
							<strong>{selectedDrive.name}</strong> ({selectedDrive.mount_path})
							and format it as FAT32. This cannot be undone.
						</p>

						{formatError && <p className="error">{formatError}</p>}

						{isFormatting && (
							<div className="format-progress">
								<div className="install-spinner" />
								<p className="formatting-hint">
									Formatting drive, please wait...
								</p>
							</div>
						)}

						<div className="confirm-actions">
							<button
								type="button"
								className="confirm-cancel"
								onClick={() => {
									setShowFormatConfirm(false);
									setFormatError(null);
								}}
								disabled={isFormatting || loading}
							>
								Cancel
							</button>
							<button
								type="button"
								className="confirm-proceed danger"
								onClick={handleFormat}
								disabled={isFormatting}
							>
								{isFormatting ? "Formatting..." : "Format to FAT32"}
							</button>
						</div>
					</div>
				</div>
			)}
		</div>
	);
}

export default DriveSelector;
