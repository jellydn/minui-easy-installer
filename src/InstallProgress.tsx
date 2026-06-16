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

  return (
    <div className="install-progress">
      <h2>{PHASE_LABELS[phase]}</h2>

      {phase !== "complete" && phase !== "error" && (
        <div className="install-spinner" />
      )}

      {message && <p className="install-message">{message}</p>}

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
            <div key={i} className={`log-line log-${entry.step}`}>
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

export default InstallProgressUI;
