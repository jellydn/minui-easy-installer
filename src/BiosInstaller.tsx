import { useCallback, useEffect, useRef, useState } from "react";
import {
  type BiosEntry,
  type BiosInstallState,
  type BiosStatus,
  bufferToBase64,
  getBiosStatus,
  installBiosFile,
  listBiosCatalog,
} from "./types/bios";

interface BiosInstallerProps {
  sdMount: string;
  onClose: () => void;
}

const initialInstallState: BiosInstallState = { status: "idle" };

function BiosInstaller({ sdMount, onClose }: BiosInstallerProps) {
  const [catalog, setCatalog] = useState<BiosEntry[]>([]);
  const [status, setStatus] = useState<BiosStatus | null>(null);
  const [installStates, setInstallStates] = useState<
    Record<string, BiosInstallState>
  >({});
  const [loadError, setLoadError] = useState<string | null>(null);
  // Map entryId -> the hidden <input type="file"> we trigger. Created on
  // demand so we don't render 11 inputs into the DOM.
  const fileInputsRef = useRef<Record<string, HTMLInputElement | null>>({});

  const refresh = useCallback(async () => {
    try {
      const [entries, biosStatus] = await Promise.all([
        listBiosCatalog(),
        getBiosStatus(sdMount),
      ]);
      setCatalog(entries);
      setStatus(biosStatus);
      setLoadError(null);
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to load";
      setLoadError(message);
    }
  }, [sdMount]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const handlePick = (entryId: string) => {
    const input = fileInputsRef.current[entryId];
    if (!input) return;
    input.value = "";
    input.click();
  };

  const handleFileChosen = async (
    entryId: string,
    file: File,
  ): Promise<void> => {
    setInstallStates((prev) => ({
      ...prev,
      [entryId]: { status: "installing", sourceFilename: file.name },
    }));

    try {
      const buffer = await file.arrayBuffer();
      const base64 = bufferToBase64(buffer);
      await installBiosFile(sdMount, entryId, base64);
      setInstallStates((prev) => ({
        ...prev,
        [entryId]: { status: "done", sourceFilename: file.name },
      }));
      await refresh();
    } catch (err) {
      const message = err instanceof Error ? err.message : "Install failed";
      setInstallStates((prev) => ({
        ...prev,
        [entryId]: {
          status: "error",
          error: message,
          sourceFilename: file.name,
        },
      }));
    }
  };

  const isLoading = catalog.length === 0 && loadError === null;

  return (
    <div className="bios-installer">
      <div className="bios-installer-header">
        <h1>BIOS Files</h1>
        <p className="subtitle">
          MinUI does not bundle BIOS files (they are copyrighted). Drop in the
          ones you already own and we will copy them to the right place on the
          SD card.
        </p>
        <p className="bios-warn">
          You are on your own to source these files. Do not ask us where to
          download them.
        </p>
      </div>

      {isLoading && (
        <div className="card">
          <div className="scanning-progress">
            <div className="install-spinner" />
            <p>Loading BIOS catalog...</p>
          </div>
        </div>
      )}

      {loadError && (
        <div className="card">
          <p className="error">{loadError}</p>
          <button type="button" onClick={() => void refresh()}>
            Retry
          </button>
        </div>
      )}

      {status && (
        <div className="card">
          <h2>Status</h2>
          <p className="bios-summary">
            {status.installed_count} of {status.entries.length} BIOS files
            installed
          </p>
        </div>
      )}

      {catalog.length > 0 && (
        <div className="card">
          <h2>Catalog</h2>
          <ul className="bios-list">
            {catalog.map((entry) => {
              const statusEntry = status?.entries.find(
                (s) => s.entry.id === entry.id,
              );
              const present = statusEntry?.present ?? false;
              const installState =
                installStates[entry.id] ?? initialInstallState;
              const targetPath = entry.subdir
                ? `Bios/${entry.subdir}/${entry.filename}`
                : `Bios/${entry.filename}`;
              const isBusy = installState.status === "installing";
              return (
                <li key={entry.id} className="bios-item">
                  <div className="bios-item-info">
                    <div className="bios-item-title">
                      <strong>{entry.system}</strong>
                      <span className="bios-item-desc">
                        {" "}
                        — {entry.description}
                      </span>
                    </div>
                    <div className="bios-item-path">{targetPath}</div>
                    <div className="bios-item-status">
                      {present ? (
                        <span className="bios-status-present">Installed</span>
                      ) : (
                        <span className="bios-status-missing">
                          Not installed
                        </span>
                      )}
                    </div>
                    {installState.status === "done" && (
                      <p className="bios-message success-message">
                        Copied {installState.sourceFilename}
                      </p>
                    )}
                    {installState.status === "error" && (
                      <p className="bios-message error">{installState.error}</p>
                    )}
                  </div>
                  <div className="bios-item-actions">
                    <input
                      ref={(el) => {
                        fileInputsRef.current[entry.id] = el;
                      }}
                      type="file"
                      style={{ display: "none" }}
                      onChange={(e) => {
                        const file = e.target.files?.[0];
                        if (file) {
                          void handleFileChosen(entry.id, file);
                        }
                      }}
                      data-testid={`bios-file-input-${entry.id}`}
                    />
                    <button
                      type="button"
                      onClick={() => handlePick(entry.id)}
                      disabled={isBusy}
                    >
                      {present ? "Replace" : "Choose file"}
                    </button>
                  </div>
                </li>
              );
            })}
          </ul>
        </div>
      )}

      <div className="bios-actions">
        <button type="button" className="wifi-cancel" onClick={onClose}>
          Back
        </button>
      </div>
    </div>
  );
}

export default BiosInstaller;
