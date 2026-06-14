import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";
import { useMountEffect } from "./hooks/useMountEffect";

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
  const [scannedNetworks, setScannedNetworks] = useState<string[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [scanFailed, setScanFailed] = useState(false);

  const scanNetworks = useCallback(async () => {
    setIsScanning(true);
    setScanFailed(false);

    try {
      const networks = await invoke<string[]>("scan_wifi_networks");
      setScannedNetworks(networks);

      // If scanning returned nothing, try to grab the currently connected SSID
      if (networks.length === 0) {
        const currentSsid = await invoke<string | null>(
          "get_current_wifi_ssid",
        );
        if (currentSsid && typeof currentSsid === "string") {
          setSsid(currentSsid);
        }
      }
    } catch {
      // Scanning is optional - fall back to manual entry
      setScanFailed(true);
    } finally {
      setIsScanning(false);
    }
  }, []);

  useMountEffect(() => {
    scanNetworks();
  });

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
      <h1>WiFi Setup</h1>
      <p className="subtitle">
        Configure WiFi credentials for your device. The password will be stored
        in wifi.txt on your SD card.
      </p>

      {error && <p className="error">{error}</p>}

      <div className="wifi-form">
        <div className="form-group">
          <label htmlFor="ssid">Network Name (SSID)</label>

          {scannedNetworks.length > 0 ? (
            <div className="ssid-selector">
              <select
                id="ssid"
                value={ssid}
                onChange={(e) => setSsid(e.target.value)}
                disabled={isSaving}
              >
                <option value="">Select a network...</option>
                {scannedNetworks.map((network) => (
                  <option key={network} value={network}>
                    {network}
                  </option>
                ))}
              </select>
              <span className="manual-or">or</span>
              <input
                type="text"
                value={ssid}
                onChange={(e) => setSsid(e.target.value)}
                placeholder="Enter manually"
                disabled={isSaving}
              />
            </div>
          ) : (
            <div>
              <input
                id="ssid"
                type="text"
                value={ssid}
                onChange={(e) => setSsid(e.target.value)}
                placeholder="Enter WiFi network name"
                disabled={isSaving}
              />
              {isScanning && (
                <div className="scanning-progress">
                  <div className="install-spinner" />
                  <p className="scanning-hint">Scanning for networks...</p>
                </div>
              )}
              {scanFailed && (
                <p className="scan-failed-hint">
                  Could not scan for networks. Enter SSID manually.
                </p>
              )}
              <button
                type="button"
                className="rescan-btn"
                onClick={scanNetworks}
                disabled={isScanning}
              >
                {isScanning ? "Scanning..." : "Scan for networks"}
              </button>
            </div>
          )}
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
          <p className="wifi-warning">
            Note: Your WiFi password will be stored in plain text on the SD card
            (wifi.txt). This is required for MinUI WiFi functionality.
          </p>
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
