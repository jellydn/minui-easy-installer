import { useScrollToBottom } from "./hooks/useScrollToBottom";
import type { InstallPhase, InstallProgressEvent } from "./types/install";

interface InstallProgressProps {
  phase: InstallPhase;
  message: string;
  log: InstallProgressEvent[];
  baseFilesCopied: number;
  extrasFilesCopied: number;
  romDirsCreated: number;
  extrasWarning: string | null;
  error: string | null;
  onDismiss: () => void;
  onCancel?: () => void;
}

const PHASE_LABELS: Record<InstallPhase, string> = {
  idle: "Preparing...",
  downloading: "Downloading",
  extracting: "Extracting",
  copying: "Copying to SD Card",
  complete: "Complete",
  error: "Failed",
};

function InstallProgressUI({
  phase,
  message,
  log,
  baseFilesCopied,
  extrasFilesCopied,
  romDirsCreated,
  extrasWarning,
  error,
  onDismiss,
  onCancel,
}: InstallProgressProps) {
  const { containerRef, sentinelRef } = useScrollToBottom(log);

  const cancellable =
    onCancel != null &&
    (phase === "downloading" || phase === "extracting" || phase === "copying");

  // Show a progress bar for the most recent download event that has a
  // known total size. Events are appended in order, so the last matching
  // entry is the current one.
  const latestDownload = [...log]
    .reverse()
    .find(
      (
        entry,
      ): entry is InstallProgressEvent & {
        currentBytes: number;
        totalBytes: number;
      } =>
        entry.step === "download" &&
        typeof entry.currentBytes === "number" &&
        typeof entry.totalBytes === "number",
    );

  return (
    <div className="install-progress">
      <h2>{PHASE_LABELS[phase]}</h2>

      {phase !== "complete" && phase !== "error" && (
        <div className="install-spinner" />
      )}

      {message && <p className="install-message">{message}</p>}

      {latestDownload && phase !== "complete" && phase !== "error" && (
        <div className="install-download-progress">
          <progress
            value={latestDownload.currentBytes}
            max={latestDownload.totalBytes}
            aria-label="Download progress"
          />
          <span className="install-download-progress-text">
            {formatBytes(latestDownload.currentBytes)} /{" "}
            {formatBytes(latestDownload.totalBytes)}
          </span>
        </div>
      )}

      {cancellable && (
        <div className="install-cancel-row">
          <button
            type="button"
            className="install-cancel-button"
            onClick={onCancel}
          >
            Cancel
          </button>
        </div>
      )}

      {log.length > 0 && (
        <div className="install-log" ref={containerRef}>
          {log.map((entry, i) => (
            // New log entries receive a stable id from the listener; the index
            // fallback only exists for entries created before that change.
            <div key={entry.id ?? i} className={`log-line log-${entry.step}`}>
              <span className="log-step">{STEP_ICON[entry.step] ?? "•"}</span>
              <span className="log-details">{entry.details}</span>
            </div>
          ))}
          <div ref={sentinelRef} />
        </div>
      )}

      {phase === "complete" && (
        <div className="install-summary">
          <p className="install-success">
            Installation completed successfully!
          </p>
          <p>Base files copied: {baseFilesCopied}</p>
          {extrasFilesCopied > 0 && (
            <p>Extras files copied: {extrasFilesCopied}</p>
          )}
          {romDirsCreated > 0 && (
            <p>ROM directories created: {romDirsCreated}</p>
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

const STEP_ICON: Record<string, string> = {
  fetch: "✓",
  download: "↓",
  extract: "▸",
  copy: "→",
  finish: "✓",
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(
    units.length - 1,
    Math.floor(Math.log(bytes) / Math.log(1024)),
  );
  const value = bytes / 1024 ** i;
  return `${value.toFixed(i === 0 ? 0 : 2)} ${units[i]}`;
}

export default InstallProgressUI;
