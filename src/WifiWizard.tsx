import { useState } from "react";

interface WifiWizardProps {
	sdMount: string | null;
	onComplete: () => void;
	onCancel: () => void;
}

function WifiWizard({ sdMount, onComplete, onCancel }: WifiWizardProps) {
	const [ssid, setSsid] = useState("");
	const [password, setPassword] = useState("");
	const [isSaving, setIsSaving] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [success, setSuccess] = useState(false);

	const handleSave = async () => {
		if (!ssid.trim()) {
			setError("Please enter a network name (SSID)");
			return;
		}

		if (!sdMount) {
			setError("No SD card selected");
			return;
		}

		setIsSaving(true);
		setError(null);

		try {
			const { invoke } = await import("@tauri-apps/api/core");
			await invoke("write_wifi_config", {
				sdMount,
				ssid: ssid.trim(),
				password,
			});

			setSuccess(true);
			setTimeout(onComplete, 1500);
		} catch (err) {
			const message =
				err instanceof Error ? err.message : "Failed to save WiFi config";
			setError(message);
		} finally {
			setIsSaving(false);
		}
	};

	if (success) {
		return (
			<div className="wifi-wizard">
				<h2>WiFi Configuration Saved</h2>
				<p className="success-message">
					wifi.txt has been created on your SD card.
				</p>
			</div>
		);
	}

	return (
		<div className="wifi-wizard">
			<h2>WiFi Setup</h2>
			<p className="wifi-description">
				Configure WiFi credentials for your device. The password will be stored
				in wifi.txt on your SD card.
			</p>

			{error && <p className="error">{error}</p>}

			<div className="wifi-form">
				<div className="form-group">
					<label htmlFor="ssid">Network Name (SSID)</label>
					<input
						id="ssid"
						type="text"
						value={ssid}
						onChange={(e) => setSsid(e.target.value)}
						placeholder="Enter WiFi network name"
						disabled={isSaving}
					/>
				</div>

				<div className="form-group">
					<label htmlFor="password">Password</label>
					<input
						id="password"
						type="password"
						value={password}
						onChange={(e) => setPassword(e.target.value)}
						placeholder="Enter WiFi password"
						disabled={isSaving}
					/>
				</div>
			</div>

			<div className="wifi-actions">
				<button
					type="button"
					className="wifi-cancel"
					onClick={onCancel}
					disabled={isSaving}
				>
					Cancel
				</button>
				<button
					type="button"
					className="wifi-save"
					onClick={handleSave}
					disabled={isSaving || !ssid.trim()}
				>
					{isSaving ? "Saving..." : "Save WiFi Config"}
				</button>
			</div>
		</div>
	);
}

export default WifiWizard;
