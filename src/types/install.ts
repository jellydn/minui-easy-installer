import type { AppError, AppErrorCode } from "./errors";
import { classifyError } from "./errors";

export interface InstallResult {
  success: boolean;
  error: string | null;
  base_files_copied: number;
  extras_files_copied: number;
  extras_warning: string | null;
  rom_dirs_created: number;
}

export type InstallError = AppError;
export type InstallErrorCode = AppErrorCode;

export type InstallResultEither =
  | { success: true; data: InstallResult }
  | { success: false; error: InstallError };

export type InstallPhase =
  | "idle"
  | "downloading"
  | "extracting"
  | "copying"
  | "complete"
  | "error";

export interface InstallProgressEvent {
  step: string;
  details: string;
}

export async function installMinui(options: {
  baseUrl: string;
  extrasUrl?: string;
  baseChecksum?: string;
  extrasChecksum?: string;
  sdMount: string;
  platform: string;
  extrasPlatform: string;
  version: string;
}): Promise<InstallResultEither> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<InstallResult>("install_minui", {
      baseUrl: options.baseUrl,
      extrasUrl: options.extrasUrl || null,
      baseChecksum: options.baseChecksum || null,
      extrasChecksum: options.extrasChecksum || null,
      sdMount: options.sdMount,
      platform: options.platform,
      extrasPlatform: options.extrasPlatform,
      version: options.version,
    });

    if (result.success) {
      return { success: true, data: result };
    }

    const errorMsg = result.error || "Installation failed";
    const code = classifyError(errorMsg);

    return {
      success: false,
      error: { message: errorMsg, code },
    };
  } catch (err) {
    const message = err instanceof Error ? err.message : "Unknown error";
    return {
      success: false,
      error: { message, code: "UNKNOWN_ERROR" },
    };
  }
}
