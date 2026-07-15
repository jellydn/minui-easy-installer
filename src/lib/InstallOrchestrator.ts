import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { type DeviceProfile, getDeviceProfile } from "../types/device";
import { errorMessage } from "../types/errors";
import type { ForkConfig } from "../types/fork";
import type {
  InstallPhase,
  InstallProgressEvent,
  InstallResult,
} from "../types/install";
import { cancelInstall, startInstallAndWait } from "../types/install";
import {
  fetchPackageRegistry,
  installPackage,
} from "../types/package";
import {
  fetchMinUIRelease,
  type MinUIRelease,
} from "../types/release";
import type { ValidationResult } from "../types/validate";
import { validateInstallation } from "../types/validate";
import type { VersionCheckResult } from "../types/version";

// ── State types (re-exported so the hook can use them) ────────

export interface OrchestratorInstallState {
  phase: InstallPhase;
  message: string;
  log: InstallProgressEvent[];
  error: string | null;
  baseFilesCopied: number;
  extrasFilesCopied: number;
  romDirsCreated: number;
  extrasWarning: string | null;
  validationResult: ValidationResult | null;
}

export interface OrchestratorUpdateAllState {
  isUpdatingAll: boolean;
  message: string;
  error: string | null;
}

export interface OrchestratorState {
  install: OrchestratorInstallState;
  updateAll: OrchestratorUpdateAllState;
}

export const INITIAL_INSTALL_STATE: OrchestratorInstallState = {
  phase: "idle",
  message: "",
  log: [],
  error: null,
  baseFilesCopied: 0,
  extrasFilesCopied: 0,
  romDirsCreated: 0,
  extrasWarning: null,
  validationResult: null,
};

const INITIAL_UPDATE_ALL_STATE: OrchestratorUpdateAllState = {
  isUpdatingAll: false,
  message: "",
  error: null,
};

export type OrchestratorChangeListener = (state: OrchestratorState) => void;

// ── Standalone helpers ────────────────────────────────────────

function generateLogId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function stepToInstallPhase(step: string, current: InstallPhase): InstallPhase {
  switch (step) {
    case "download":
      return "downloading";
    case "extract":
      return "extracting";
    case "copy":
      return "copying";
    default:
      return current;
  }
}

/**
 * Fetch the release for a fork and run the install IPC.
 * Stateless — takes ForkConfig directly, no ref needed.
 */
async function fetchAndInstallRelease(
  sdMount: string,
  profile: DeviceProfile,
  fork: ForkConfig,
): Promise<
  | { kind: "ok"; release: MinUIRelease; data: InstallResult }
  | { kind: "err"; message: string }
> {
  const releaseResult = await fetchMinUIRelease(fork);
  if (!releaseResult.success) {
    return {
      kind: "err",
      message: `Failed to fetch ${fork.label} release: ${releaseResult.error.message}`,
    };
  }
  const release = releaseResult.data;
  try {
    const data = await startInstallAndWait({
      baseUrl: release.baseArchiveUrl,
      extrasUrl: release.extrasArchiveUrl || undefined,
      baseChecksum: release.checksums?.base || undefined,
      extrasChecksum: release.checksums?.extras || undefined,
      sdMount,
      platform: profile.platform,
      extrasPlatform: profile.extrasPlatform,
      version: release.version,
      forkName: fork.minuiTxtPrefix,
    });
    return { kind: "ok", release, data };
  } catch (err) {
    return {
      kind: "err",
      message: `${fork.label} install failed: ${errorMessage(err)}`,
    };
  }
}

// ── Orchestrator ──────────────────────────────────────────────

/**
 * Owns the MinUI install + update-all state machine without React.
 *
 * All async flows mutate internal state and call `onChange()` so
 * the hook can sync to React. Tauri event listeners are managed
 * internally — attached during `start()`, cleaned up in `finally`.
 */
export class InstallOrchestrator {
  private installState: OrchestratorInstallState = {
    ...INITIAL_INSTALL_STATE,
  };
  private updateAllState: OrchestratorUpdateAllState = {
    ...INITIAL_UPDATE_ALL_STATE,
  };

  private listener: OrchestratorChangeListener | null = null;
  private cancelled = false;

  // ── Public subscription ─────────────────────────────────

  /** Subscribe to state changes. Returns an unsubscribe function. */
  subscribe(listener: OrchestratorChangeListener): () => void {
    this.listener = listener;
    // Emit initial state so subscribers can seed their React state.
    listener(this.snapshot());
    return () => {
      this.listener = null;
    };
  }

  // ── Getters ─────────────────────────────────────────────

  get install(): OrchestratorInstallState {
    return this.installState;
  }

  get updateAllStatus(): OrchestratorUpdateAllState {
    return this.updateAllState;
  }

  get isInstalling(): boolean {
    const { phase } = this.installState;
    return phase !== "idle" && phase !== "complete" && phase !== "error";
  }

  // ── Actions ─────────────────────────────────────────────

  /** Start a MinUI install for the selected device + drive. */
  async start(
    fork: ForkConfig,
    device: string,
    sdMount: string,
  ): Promise<void> {
    const profile = getDeviceProfile(device);
    if (!profile) {
      this.setInstall({ error: "Unknown device profile", phase: "error" });
      return;
    }

    this.setInstall({
      phase: "downloading",
      message: "",
      log: [],
      error: null,
    });

    // Attach the progress listener before the try so the unlisten
    // handle is always assigned when the finally runs.
    let unlisten: UnlistenFn | undefined;
    try {
      unlisten = await this.attachProgressListener();
    } catch (err) {
      this.setInstall({ error: errorMessage(err), phase: "error" });
      return;
    }

    try {
      const fetched = await fetchAndInstallRelease(sdMount, profile, fork);
      if (fetched.kind === "err") {
        this.setInstall({ error: fetched.message, phase: "error" });
        return;
      }

      const { release, data } = fetched;
      const fileName = release.baseArchiveUrl.split("/").pop();

      this.setInstall({
        log: [
          ...this.installState.log,
          {
            step: "fetch",
            details: `Found ${fork.label} v${release.version} (${fileName})`,
            id: generateLogId(),
          },
        ],
      });

      const valResult = await validateInstallation({
        sdMount,
        platform: profile.platform,
        hasExtras: data.extras_files_copied > 0,
        extrasDir: profile.installPathRules.extrasDir,
      });

      this.setInstall({
        phase: "complete",
        message: "Installation completed successfully!",
        baseFilesCopied: data.base_files_copied,
        extrasFilesCopied: data.extras_files_copied,
        romDirsCreated: data.rom_dirs_created,
        extrasWarning: data.extras_warning,
        validationResult: valResult.success ? valResult.data : null,
      });
    } catch (err) {
      // When explicitly cancelled, keep the cancellation message.
      if (this.cancelled) {
        this.setInstall({ phase: "error" });
      } else {
        this.setInstall({ error: errorMessage(err), phase: "error" });
      }
    } finally {
      this.cancelled = false;
      unlisten?.();
    }
  }

  /** Cancel the in-flight install. */
  cancel(): void {
    this.cancelled = true;
    void cancelInstall();
    this.setInstall({
      phase: "error",
      error: "Installation cancelled",
      message: "",
    });
  }

  /** Retry validation after an install completes. */
  async retryValidation(device: string, sdMount: string): Promise<void> {
    const profile = getDeviceProfile(device);
    if (!profile) return;
    const { extrasFilesCopied } = this.installState;
    const valResult = await validateInstallation({
      sdMount,
      platform: profile.platform,
      hasExtras: extrasFilesCopied > 0,
      extrasDir: profile.installPathRules.extrasDir,
    });
    if (valResult.success) {
      this.setInstall({ validationResult: valResult.data });
    }
  }

  /** Update MinUI + all pending packages. */
  async updateAll(
    fork: ForkConfig,
    device: string,
    sdMount: string,
    versionCheck: VersionCheckResult | null,
    packageUpdates: {
      name: string;
      installed_version: string | null;
      latest_version: string;
    }[],
    onAfterUpdate: (sdMount: string) => Promise<void> | void,
  ): Promise<void> {
    const profile = getDeviceProfile(device);
    if (!profile) return;

    this.setUpdateAll({
      isUpdatingAll: true,
      error: null,
      message: "Starting updates...",
    });

    const finish = (err: string | null, message: string) => {
      this.setUpdateAll({ error: err, message, isUpdatingAll: false });
    };

    try {
      if (versionCheck?.update_available) {
        this.setUpdateAll({
          message: `Updating ${fork.label}...`,
        });

        const fetched = await fetchAndInstallRelease(sdMount, profile, fork);
        if (fetched.kind === "err") {
          finish(fetched.message, "");
          return;
        }
      }

      if (packageUpdates.length > 0) {
        this.setUpdateAll({
          message: `Updating ${packageUpdates.length} package(s)...`,
        });

        const registryResult = await fetchPackageRegistry();
        if (!registryResult.success) {
          finish(
            `Failed to fetch package registry: ${registryResult.error.message}`,
            "",
          );
          return;
        }

        const packageByName = new Map(
          registryResult.data.packages.map((p) => [p.name, p]),
        );

        const installResults = await Promise.all(
          packageUpdates.map(async (update) => {
            const entry = packageByName.get(update.name);
            if (!entry) return `${update.name}: not found in registry`;
            const result = await installPackage({
              artifactUrl: entry.artifactUrl,
              checksum: entry.checksum || undefined,
              sdMount,
              targetDir: entry.installPathRules.targetDir,
              extractToRoot: entry.installPathRules.extractToRoot,
              pakName: entry.installPathRules.pakName || update.name,
              platform: profile.platform,
            });
            if (!result.success) {
              return `${update.name}: ${result.error?.message || "install failed"}`;
            }
            return null;
          }),
        );

        const errors = installResults.filter((e): e is string => e !== null);
        if (errors.length > 0) {
          finish(`Package update errors:\n${errors.join("\n")}`, "");
          return;
        }
      }

      finish(null, "All updates completed!");
      await onAfterUpdate(sdMount);
    } catch (err) {
      finish(errorMessage(err), "");
    }
  }

  /** Reset install state to idle. */
  dismissInstall(): void {
    this.installState = { ...INITIAL_INSTALL_STATE };
    this.emit();
  }

  /** Clear the validation result. */
  dismissValidation(): void {
    this.setInstall({ validationResult: null });
  }

  // ── Private helpers ──────────────────────────────────────

  private setInstall(patch: Partial<OrchestratorInstallState>): void {
    this.installState = { ...this.installState, ...patch };
    this.emit();
  }

  private setUpdateAll(patch: Partial<OrchestratorUpdateAllState>): void {
    this.updateAllState = { ...this.updateAllState, ...patch };
    this.emit();
  }

  private snapshot(): OrchestratorState {
    return {
      install: this.installState,
      updateAll: this.updateAllState,
    };
  }

  private emit(): void {
    this.listener?.(this.snapshot());
  }

  private async attachProgressListener(): Promise<UnlistenFn> {
    return listen<InstallProgressEvent>("install-progress", (event) => {
      const { step, details } = event.payload;
      const phase = stepToInstallPhase(step, this.installState.phase);
      const entry = { ...event.payload, id: generateLogId() };
      this.setInstall({
        phase,
        message: details,
        log: [...this.installState.log, entry],
      });
    });
  }
}
