import { useState } from "react";
import ConfirmDialog from "./ConfirmDialog";
import DeviceSelector from "./DeviceSelector";
import DriveSelector from "./DriveSelector";
import type { RemovableDrive } from "./types/drive";

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

	const handleInstallClick = () => {
		setShowConfirmDialog(true);
	};

	const handleConfirmInstall = () => {
		setShowConfirmDialog(false);
	};

	const handleCancelInstall = () => {
		setShowConfirmDialog(false);
	};

	return (
		<div className="home">
			<h1>MinUI Easy Installer</h1>
			<p className="subtitle">
				The easiest way to install and manage MinUI on retro handheld devices.
			</p>

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
					<button type="button" onClick={handleInstallClick}>
						Install MinUI
					</button>
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
