import { useCallback, useEffect, useRef, useState } from "react";
import { formatSize } from "./types/drive";
import type { HealthCheckResult } from "./types/validate";
import { checkSdCardHealth } from "./types/validate";

interface HealthCheckProps {
  sdMount: string;
  devicePlatform?: string;
}

function HealthCheck({ sdMount, devicePlatform }: HealthCheckProps) {
  const [healthResult, setHealthResult] = useState<HealthCheckResult | null>(
    null,
  );
  const [isChecking, setIsChecking] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  // Track the last-checked mount so we don't re-check the same drive.
  const lastSdMount = useRef<string | null>(null);

  const runCheck = useCallback(async () => {
    if (!sdMount) return;

    setIsChecking(true);
    setError(null);
    setHealthResult(null);

    const result = await checkSdCardHealth({
      sdMount,
      devicePlatform,
    });

    if (result.success) {
      setHealthResult(result.data);
    } else {
      setError(result.error.message);
    }

    setIsChecking(false);
  }, [sdMount, devicePlatform]);

  // Auto-run when a new SD card is selected.
  useEffect(() => {
    if (sdMount && sdMount !== lastSdMount.current) {
      lastSdMount.current = sdMount;
      void runCheck();
    }
  }, [sdMount, runCheck]);

  const handleCopyReport = async () => {
    if (!healthResult) return;

    try {
      await navigator.clipboard.writeText(healthResult.support_report);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      const textarea = document.createElement("textarea");
      textarea.value = healthResult.support_report;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="health-check">
      <h3>SD Card Health Check</h3>

      <button
        type="button"
        onClick={runCheck}
        disabled={isChecking}
        className="health-check-btn"
      >
        {isChecking ? "Checking..." : "Re-check Health"}
      </button>

      {error && <p className="error">{error}</p>}

      {healthResult && (
        <div className="health-results">
          <div className="health-summary">
            <span className="health-passed">
              {healthResult.passed_count} passed
            </span>
            {healthResult.failed_count > 0 && (
              <span className="health-failed">
                {" "}
                / {healthResult.failed_count} warnings
              </span>
            )}
          </div>

          <div className="health-checks">
            {healthResult.checks.map((check) => (
              <div
                key={check.name}
                className={`health-check-item ${check.passed ? "passed" : "warning"}`}
              >
                <span className="check-icon">{check.passed ? "✓" : "⚠"}</span>
                <span>{check.message}</span>
              </div>
            ))}
          </div>

          {healthResult.filesystem && (
            <p className="health-filesystem">
              Filesystem: {healthResult.filesystem}
            </p>
          )}

          {healthResult.free_space_bytes !== null && (
            <p className="health-space">
              Free Space: {formatSize(healthResult.free_space_bytes)}
            </p>
          )}

          {healthResult.read_speed_mbs != null && (
            <p className="health-speed">
              Read Speed: {healthResult.read_speed_mbs.toFixed(1)} MB/s
            </p>
          )}

          <div className="health-actions">
            <button
              type="button"
              onClick={handleCopyReport}
              className="copy-report-btn"
            >
              {copied ? "Copied!" : "Copy Support Report"}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default HealthCheck;
