import { errorMessage } from "./errors";

export interface ValidationCheck {
  name: string;
  passed: boolean;
  message: string;
}

export interface ValidationResult {
  success: boolean;
  checks: ValidationCheck[];
  passed_count: number;
  failed_count: number;
  free_space_bytes: number | null;
  /** The device-specific path that was validated (e.g. "miyoo", "rg35xxplus", "em_ui.sh"). */
  device_path: string;
  /** Warning message when multiple device folders are present on the SD card. */
  multiple_device_folders_warning: string | null;
}

export interface HealthCheckResult {
  checks: ValidationCheck[];
  passed_count: number;
  failed_count: number;
  free_space_bytes: number | null;
  filesystem: string | null;
  support_report: string;
  /** Sequential read speed in MB/s, if the benchmark ran. */
  read_speed_mbs: number | null;
}

export type ValidationError = {
  message: string;
  code: "VALIDATION_ERROR" | "UNKNOWN_ERROR";
};

export type ValidationResultEither =
  | { success: true; data: ValidationResult }
  | { success: false; error: ValidationError };

export async function validateInstallation(options: {
  sdMount: string;
  platform: string;
  hasExtras: boolean;
  extrasDir: string;
}): Promise<ValidationResultEither> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<ValidationResult>("validate_installation", {
      opts: {
        sdMount: options.sdMount,
        platform: options.platform,
        hasExtras: options.hasExtras,
        extrasDir: options.extrasDir,
      },
    });

    return { success: true, data: result };
  } catch (err) {
    const message = errorMessage(err);
    return {
      success: false,
      error: { message, code: "VALIDATION_ERROR" },
    };
  }
}

export async function formatValidationReport(
  result: ValidationResult,
): Promise<string> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<string>("format_validation_report", { result });
  } catch {
    // Fallback to client-side formatting
    return formatReportLocally(result);
  }
}

function formatReportLocally(result: ValidationResult): string {
  const lines: string[] = [];
  lines.push("MinUI Installation Validation Report");
  lines.push("=====================================");
  lines.push("");
  lines.push(result.success ? "Status: PASSED" : "Status: FAILED");
  lines.push("");
  lines.push(
    `Checks: ${result.passed_count} passed, ${result.failed_count} failed`,
  );
  lines.push("");
  lines.push("Details:");

  for (const check of result.checks) {
    const status = check.passed ? "✓" : "✗";
    lines.push(`  ${status} ${check.message}`);
  }

  if (result.free_space_bytes !== null) {
    lines.push("");
    lines.push(`Free Space: ${formatBytes(result.free_space_bytes)}`);
  }

  return lines.join("\n");
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

export async function checkSdCardHealth(options: {
  sdMount: string;
  devicePlatform?: string;
}): Promise<
  | { success: true; data: HealthCheckResult }
  | { success: false; error: ValidationError }
> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<HealthCheckResult>("check_sd_card_health", {
      opts: {
        sdMount: options.sdMount,
        devicePlatform: options.devicePlatform || null,
      },
    });

    return { success: true, data: result };
  } catch (err) {
    const message = errorMessage(err);
    return {
      success: false,
      error: { message, code: "VALIDATION_ERROR" },
    };
  }
}
