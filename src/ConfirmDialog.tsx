import { getDeviceProfile } from "./types/device";
import type { RemovableDrive } from "./types/drive";
import { formatSize } from "./types/drive";

interface ConfirmDialogProps {
	selectedDevice: string;
	selectedDrive: RemovableDrive;
	onConfirm: () => void;
	onCancel: () => void;
}

function ConfirmDialog({
	selectedDevice,
	selectedDrive,
	onConfirm,
	onCancel,
}: ConfirmDialogProps) {
	const deviceProfile = getDeviceProfile(selectedDevice);

	return (
		<div className="confirm-overlay">
			<div className="confirm-dialog">
				<h2>Confirm Installation</h2>
				<p className="confirm-warning">
					This will write files to the following drive. Please confirm you want
					to proceed.
				</p>

				<div className="confirm-details">
					<div className="confirm-section">
						<h3>Target Drive</h3>
						<p>
							<strong>Name:</strong> {selectedDrive.name}
						</p>
						<p>
							<strong>Mount Path:</strong> {selectedDrive.mount_path}
						</p>
						<p>
							<strong>Size:</strong> {formatSize(selectedDrive.size_bytes)}
						</p>
						{selectedDrive.filesystem && (
							<p>
								<strong>Filesystem:</strong> {selectedDrive.filesystem}
							</p>
						)}
					</div>

					<div className="confirm-section">
						<h3>Target Device</h3>
						<p>
							<strong>Device:</strong> {deviceProfile?.name || "Unknown Device"}
						</p>
						{deviceProfile && (
							<p>
								<strong>Platform:</strong> {deviceProfile.platform}
							</p>
						)}
					</div>
				</div>

				<div className="confirm-actions">
					<button className="confirm-cancel" onClick={onCancel} type="button">
						Cancel
					</button>
					<button className="confirm-proceed" onClick={onConfirm} type="button">
						Proceed with Installation
					</button>
				</div>
			</div>
		</div>
	);
}

export default ConfirmDialog;
