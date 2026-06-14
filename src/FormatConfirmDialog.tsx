import type { RemovableDrive } from "./types/drive";

interface FormatConfirmDialogProps {
	drive: RemovableDrive;
	isFormatting: boolean;
	error: string | null;
	onConfirm: () => void;
	onCancel: () => void;
}

function FormatConfirmDialog({
	drive,
	isFormatting,
	error,
	onConfirm,
	onCancel,
}: FormatConfirmDialogProps) {
	return (
		<div className="confirm-overlay">
			<div className="confirm-dialog">
				<h2>Format Drive?</h2>
				<p className="confirm-warning">
					This will <strong>erase all data</strong> on{" "}
					<strong>{drive.name}</strong> ({drive.mount_path}) and format it as
					FAT32. This cannot be undone.
				</p>

				{error && <p className="error">{error}</p>}

				{isFormatting && (
					<div className="format-progress">
						<div className="install-spinner" />
						<p className="formatting-hint">Formatting drive, please wait...</p>
					</div>
				)}

				<div className="confirm-actions">
					<button
						type="button"
						className="confirm-cancel"
						onClick={onCancel}
						disabled={isFormatting}
					>
						Cancel
					</button>
					<button
						type="button"
						className="confirm-proceed danger"
						onClick={onConfirm}
						disabled={isFormatting}
					>
						{isFormatting ? "Formatting..." : "Format to FAT32"}
					</button>
				</div>
			</div>
		</div>
	);
}

export default FormatConfirmDialog;
