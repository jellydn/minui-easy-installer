import { useCallback, useState } from "react";
import { formatSize } from "./types/drive";
import type { ValidationResult } from "./types/validate";
import { formatValidationReport } from "./types/validate";

interface ValidationReportProps {
  result: ValidationResult;
  onDismiss: () => void;
  onRetry: () => void;
}

function ValidationReportUI({
  result,
  onDismiss,
  onRetry,
}: ValidationReportProps) {
  const [copySuccess, setCopySuccess] = useState(false);

  const handleCopyReport = useCallback(async () => {
    try {
      const text = await formatValidationReport(result);
      await navigator.clipboard.writeText(text);
      setCopySuccess(true);
      setTimeout(() => setCopySuccess(false), 2000);
    } catch {
      // Clipboard API failed; the report is still visible below.
    }
  }, [result]);

  return (
    <div className="validation-report">
      <h2>Installation Validation</h2>

      <div
        className={`validation-status ${result.success ? "success" : "failed"}`}
      >
        {result.success ? (
          <p className="validation-success">
            All checks passed! MinUI is correctly installed.
          </p>
        ) : (
          <p className="validation-failed">
            Some checks failed. Please review the details below.
          </p>
        )}
      </div>

      <div className="validation-summary">
        <p>
          <strong>{result.passed_count}</strong> passed,{" "}
          <strong>{result.failed_count}</strong> failed
        </p>
      </div>

      <div className="validation-device-path">
        <h3>Device-Specific Path</h3>
        <p>
          Checked:{" "}
          <code className="device-path-code">
            {result.device_path || "unknown"}
          </code>
        </p>
      </div>

      {result.multiple_device_folders_warning && (
        <div className="validation-warning validation-warning-multiple-devices">
          <h3>⚠️ Multiple Device Folders Detected</h3>
          <p className="warning-message">
            {result.multiple_device_folders_warning}
          </p>
        </div>
      )}

      <div className="validation-checks">
        <h3>Check Details</h3>
        <ul>
          {result.checks.map((check) => (
            <li
              key={check.name}
              className={`check-item ${check.passed ? "passed" : "failed"}`}
            >
              <span className="check-icon">{check.passed ? "✓" : "✗"}</span>
              <span className="check-message">{check.message}</span>
            </li>
          ))}
        </ul>
      </div>

      {result.free_space_bytes !== null && (
        <div className="validation-space">
          <h3>Free Space</h3>
          <p>{formatSize(result.free_space_bytes)}</p>
        </div>
      )}

      <div className="validation-actions">
        <button type="button" onClick={handleCopyReport} className="copy-btn">
          {copySuccess ? "Copied!" : "Copy Report"}
        </button>
        {!result.success && (
          <button type="button" onClick={onRetry} className="retry-btn">
            Retry Validation
          </button>
        )}
        <button type="button" onClick={onDismiss} className="done-btn">
          Done
        </button>
      </div>
    </div>
  );
}

export default ValidationReportUI;
