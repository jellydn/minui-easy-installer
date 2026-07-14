import { useCallback, useEffect, useState } from "react";
import ConfirmDialog from "./ConfirmDialog";
import DeviceSelector from "./DeviceSelector";
import DriveSelector from "./DriveSelector";
import HealthCheck from "./HealthCheck";
import { useFork } from "./contexts/ForkContext";
import { useForkInstall } from "./hooks/useForkInstall";
import { useVersionCheck } from "./hooks/useVersionCheck";
import InstallProgressUI from "./InstallProgress";
import { getDeviceProfile } from "./types/device";
import type { RemovableDrive } from "./types/drive";
import { formatSize } from "./types/drive";
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
  const { fork } = useFork();
  const version = useVersionCheck(fork);
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);

  // Drive changes drive a version check; check() is stable (useRef-guarded).
  useEffect(() => {
    if (selectedDrive) {
      version.check(selectedDrive.mount_path);
    } else {
      version.reset();
    }
  }, [selectedDrive, version.check, version.reset]);

  const forkInstall = useForkInstall({
    selectedDevice,
    selectedDriveMount: selectedDrive?.mount_path ?? null,
    versionCheck: version.versionCheck,
    packageUpdates: version.packageUpdates,
    onAfterUpdate: (sdMount) => version.check(sdMount),
  });

  const handleInstallClick = () => setShowConfirmDialog(true);
  const handleCancelInstall = () => setShowConfirmDialog(false);
  const handleConfirmInstall = useCallback(() => {
    setShowConfirmDialog(false);
    void forkInstall.installMinUI();
  }, [forkInstall.installMinUI]);

  const hasUpdates =
    (version.versionCheck?.update_available &&
      version.versionCheck?.installed != null) ||
    version.packageUpdates.length > 0;

  return (
    <div className="screen">
      <h1>{fork.label} Easy Installer</h1>
      <p className="subtitle">
        The easiest way to install and manage {fork.label} on retro handheld
        devices.
      </p>

      {forkInstall.install.phase !== "idle" ? (
        <div className="card">
          {forkInstall.install.validationResult ? (
            <ValidationReportUI
              result={forkInstall.install.validationResult}
              onDismiss={forkInstall.dismissValidation}
              onRetry={forkInstall.retryValidation}
            />
          ) : (
            <InstallProgressUI
              phase={forkInstall.install.phase}
              message={forkInstall.install.message}
              log={forkInstall.install.log}
              baseFilesCopied={forkInstall.install.baseFilesCopied}
              extrasFilesCopied={forkInstall.install.extrasFilesCopied}
              romDirsCreated={forkInstall.install.romDirsCreated}
              extrasWarning={forkInstall.install.extrasWarning}
              error={forkInstall.install.error}
              onDismiss={forkInstall.dismissInstall}
              onCancel={forkInstall.cancelInstall}
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

              {version.isChecking ? (
                <p className="checking">Checking version...</p>
              ) : version.versionCheck ? (
                <div className="version-info">
                  {version.versionCheck.installed ? (
                    <p className="installed-version">
                      <strong>{fork.label}:</strong> v
                      {version.versionCheck.installed.version}
                    </p>
                  ) : (
                    <p className="no-version">
                      <strong>{fork.label}:</strong> Not detected
                    </p>
                  )}
                  {version.versionCheck.update_available ? (
                    <p className="update-available">
                      Update available: v{version.versionCheck.latest}
                    </p>
                  ) : version.versionCheck.installed ? (
                    <p className="up-to-date">{fork.label} is up to date</p>
                  ) : null}

                  {version.packageUpdates.length > 0 && (
                    <div className="package-updates">
                      <h3>
                        {version.packageUpdates.length} Package Update
                        {version.packageUpdates.length > 1 ? "s" : ""} Available
                      </h3>
                      <ul>
                        {version.packageUpdates.map((update) => (
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
              {forkInstall.isUpdatingAll ? (
                <div className="update-all-status">
                  <h2>Updating...</h2>
                  <div className="install-spinner" />
                  <p>{forkInstall.updateAllMessage}</p>
                  {forkInstall.updateAllError && (
                    <p className="error">{forkInstall.updateAllError}</p>
                  )}
                </div>
              ) : (
                <>
                  <h2>
                    {hasUpdates
                      ? "Updates Available"
                      : version.versionCheck?.installed
                        ? `Update ${fork.label}`
                        : `Install ${fork.label}`}
                  </h2>
                  {hasUpdates && (
                    <button
                      type="button"
                      className="update-all-btn"
                      onClick={() => void forkInstall.updateAll()}
                      disabled={forkInstall.isInstalling}
                    >
                      Update All
                    </button>
                  )}
                  <button
                    type="button"
                    onClick={handleInstallClick}
                    disabled={forkInstall.isInstalling}
                  >
                    {version.versionCheck?.installed
                      ? `Update ${fork.label} Only`
                      : `Install ${fork.label}`}
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
