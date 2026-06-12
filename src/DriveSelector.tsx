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
				</div>
			)}
		</div>
	);
}

export default DriveSelector;
