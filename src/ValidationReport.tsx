import { useCallback, useState } from "react";
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
  const [reportText, setReportText] = useState<string | null>(null);

  const handleCopyReport = useCallback(async () => {
    try {
      const text = reportText || (await formatValidationReport(result));
      await navigator.clipboard.writeText(text);
      setReportText(text);
      setCopySuccess(true);
      setTimeout(() => setCopySuccess(false), 2000);
    } catch {
      // Fallback: create textarea for manual copy
      const textarea = document.createElement("textarea");
      textarea.value = reportText || "";
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
      setCopySuccess(true);
      setTimeout(() => setCopySuccess(false), 2000);
    }
  }, [result, reportText]);

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
          <p>{formatBytes(result.free_space_bytes)}</p>
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

function formatBytes(bytes: number): string {
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;

  if (bytes >= GB) {
    return `${(bytes / GB).toFixed(2)} GB`;
  }
  if (bytes >= MB) {
    return `${(bytes / MB).toFixed(2)} MB`;
  }
  if (bytes >= KB) {
    return `${(bytes / KB).toFixed(2)} KB`;
  }
  return `${bytes} bytes`;
}

export default ValidationReportUI;
