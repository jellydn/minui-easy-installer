import type { InstallPhase } from "./types/install";

interface InstallProgressProps {
	phase: InstallPhase;
	message: string;
	baseFilesCopied: number;
	extrasFilesCopied: number;
	extrasWarning: string | null;
	error: string | null;
	onDismiss: () => void;
}

const PHASE_LABELS: Record<InstallPhase, string> = {
	idle: "Preparing...",
	downloading: "Downloading MinUI",
	extracting: "Extracting Archives",
	copying: "Copying Files to SD Card",
	complete: "Installation Complete",
	error: "Installation Failed",
};

function InstallProgressUI({
	phase,
	message,
	baseFilesCopied,
	extrasFilesCopied,
	extrasWarning,
	error,
	onDismiss,
}: InstallProgressProps) {
	return (
		<div className="install-progress">
			<h2>{PHASE_LABELS[phase]}</h2>

			{phase !== "complete" && phase !== "error" && (
				<div className="install-spinner" />
			)}

			<p className="install-message">{message || "Please wait..."}</p>

			{phase === "complete" && (
				<div className="install-summary">
					<p className="install-success">
						Installation completed successfully!
					</p>
					<p>Base files copied: {baseFilesCopied}</p>
					{extrasFilesCopied > 0 && (
						<p>Extras files copied: {extrasFilesCopied}</p>
					)}
					{extrasWarning && (
						<p className="extras-warning">Warning: {extrasWarning}</p>
					)}
				</div>
			)}

			{phase === "error" && error && (
				<div className="install-error">
					<p className="error">{error}</p>
					<p className="install-retry-hint">
						You can try again or check your connection and SD card.
					</p>
				</div>
			)}

			{(phase === "complete" || phase === "error") && (
				<div className="install-actions">
					<button type="button" onClick={onDismiss}>
						{phase === "complete" ? "Done" : "Dismiss"}
					</button>
				</div>
			)}
		</div>
	);
}

export default InstallProgressUI;
