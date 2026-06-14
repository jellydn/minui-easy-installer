export interface InstalledVersion {
  version: string;
  source: string;
}

export interface VersionCheckResult {
  installed: InstalledVersion | null;
  latest: string | null;
  update_available: boolean;
}

export type VersionError = {
  message: string;
  code: "VERSION_ERROR" | "UNKNOWN_ERROR";
};

export type VersionCheckResultEither =
  | { success: true; data: VersionCheckResult }
  | { success: false; error: VersionError };

export async function checkMinuiVersion(options: {
  sdMount: string;
  latestVersion?: string;
}): Promise<VersionCheckResultEither> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await invoke<VersionCheckResult>("check_minui_version", {
      sdMount: options.sdMount,
      latestVersion: options.latestVersion || null,
    });

    return { success: true, data: result };
  } catch (err) {
    const message = err instanceof Error ? err.message : "Unknown error";
    return {
      success: false,
      error: { message, code: "VERSION_ERROR" },
    };
  }
}
