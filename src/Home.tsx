import { useState } from "react";
import DriveSelector from "./DriveSelector";
import { getAllDeviceProfiles } from "./types/device";
import type { RemovableDrive } from "./types/drive";

function Home() {
	const [selectedDrive, setSelectedDrive] = useState<RemovableDrive | null>(
		null,
	);
	const [selectedDevice, setSelectedDevice] = useState<string | null>(null);

	const devices = getAllDeviceProfiles();

	return (
		<div className="home">
			<h1>MinUI Easy Installer</h1>
			<p className="subtitle">
				The easiest way to install and manage MinUI on retro handheld devices.
			</p>

			<div className="card">
				<h2>Select Your Device</h2>
				<select
					value={selectedDevice || ""}
					onChange={(e) => setSelectedDevice(e.target.value || null)}
				>
					<option value="">Choose a device...</option>
					{devices.map((device) => (
						<option key={device.id} value={device.id}>
							{device.name}
						</option>
					))}
				</select>
			</div>

			<div className="card">
				<DriveSelector
					selectedDrive={selectedDrive}
					onSelectDrive={setSelectedDrive}
				/>
			</div>

			{selectedDrive && selectedDevice && (
				<div className="card ready">
					<h2>Ready to Install</h2>
					<p>Device: {devices.find((d) => d.id === selectedDevice)?.name}</p>
					<p>
						Drive: {selectedDrive.name} ({selectedDrive.mount_path})
					</p>
				</div>
			)}
		</div>
	);
}

export default Home;
