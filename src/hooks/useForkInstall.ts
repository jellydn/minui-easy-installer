import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type MutableRefObject,
} from "react";
import { useFork } from "../contexts/ForkContext";
import { type DeviceProfile, getDeviceProfile } from "../types/device";
import type { ForkConfig } from "../types/fork";
import type {
  InstallPhase,
  InstallProgressEvent,
  InstallResult,
} from "../types/install";
import { cancelInstall, startInstallAndWait } from "../types/install";
import { fetchPackageRegistry, installPackage } from "../types/package";
import { fetchMinUIRelease, type MinUIRelease } from "../types/release";
import type { ValidationResult } from "../types/validate";
import { validateInstallation } from "../types/validate";
import type { VersionCheckResult } from "../types/version";

interface InstallState {
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

export interface UseForkInstallOptions {
  selectedDevice: string | null;
  selectedDriveMount: string | null;
  /** Version check result; needed to gate update-all. */
  versionCheck: VersionCheckResult | null;
  /** Pending package updates; needed for update-all. */
  packageUpdates: {
    name: string;
    installed_version: string | null;
    latest_version: string;
  }[];
  /** Refresh the version check after update-all completes. */
  onAfterUpdate: (sdMount: string) => Promise<void> | void;
}

export interface UseForkInstallResult {
  install: InstallState;
  isInstalling: boolean;
  /** Run a single MinUI install (called from the Confirm dialog). */
  installMinUI: () => Promise<void>;
  /** Cancel the in-flight install, if any. */
  cancelInstall: () => void;
  /** Update MinUI + all pending packages concurrently. */
  updateAll: () => Promise<void>;
  /** Status of the "update all" flow, surfaced separately from `install`. */
  isUpdatingAll: boolean;
  updateAllMessage: string;
  updateAllError: string | null;
  dismissInstall: () => void;
  dismissValidation: () => void;
  retryValidation: () => Promise<void>;
}

const INITIAL_STATE: InstallState = {
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

/**
 * Fetch the release for the current fork and run the install IPC.
 * Returns either the release + install result or a pre-formatted
 * error message. Shared by `installMinUI` and the version-update
 * phase of `updateAll` so the install contract is single-sourced.
 *
 * Lives at module level so it isn't recreated on every render.
 */
async function fetchAndInstallRelease(
  sdMount: string,
  profile: DeviceProfile,
  forkRef: MutableRefObject<ForkConfig>,
): Promise<
  | { kind: "ok"; release: MinUIRelease; data: InstallResult }
  | { kind: "err"; message: string }
> {
  const releaseResult = await fetchMinUIRelease(forkRef.current);
  if (!releaseResult.success) {
    return {
      kind: "err",
      message: `Failed to fetch ${forkRef.current.label} release: ${releaseResult.error.message}`,
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
      forkName: forkRef.current.minuiTxtPrefix,
    });
    return { kind: "ok", release, data };
  } catch (err) {
    return {
      kind: "err",
      message: `${forkRef.current.label} install failed: ${errorMessage(err)}`,
    };
  }
}

/**
 * Owns the MinUI install + update-all orchestration: release fetch,
 * install IPC, Tauri progress event listener, validation, and the
 * per-package update batch.
 *
 * Lives in a hook so Home.tsx stays presentational and so the install
 * pipeline is unit-testable without rendering the whole Home tree.
 */
export function useForkInstall(
  options: UseForkInstallOptions,
): UseForkInstallResult {
  const { fork } = useFork();
  const {
    selectedDevice,
    selectedDriveMount,
    versionCheck,
    packageUpdates,
    onAfterUpdate,
  } = options;

  const [install, setInstall] = useState<InstallState>(INITIAL_STATE);
  const [isUpdatingAll, setIsUpdatingAll] = useState(false);
  const [updateAllMessage, setUpdateAllMessage] = useState("");
  const [updateAllError, setUpdateAllError] = useState<string | null>(null);

  // Hold the latest fork in a ref so the install callback stays stable
  // and we don't capture a stale value across fork changes.
  const forkRef = useRef<ForkConfig>(fork);
  useEffect(() => {
    forkRef.current = fork;
  }, [fork]);

  const dismissInstall = useCallback(() => setInstall(INITIAL_STATE), []);
  const dismissValidation = useCallback(
    () => setInstall((s) => ({ ...s, validationResult: null })),
    [],
  );

  const installMinUI = useCallback(async () => {
    if (!selectedDevice || !selectedDriveMount) return;

    const profile = getDeviceProfile(selectedDevice);
    if (!profile) {
      setInstall((s) => ({
        ...s,
        error: "Unknown device profile",
        phase: "error",
      }));
      return;
    }

    setInstall((s) => ({
      ...s,
      phase: "downloading",
      message: "",
      log: [],
      error: null,
    }));

    // Attach the progress listener before the try so the unlisten
    // handle is always assigned when the finally runs, even if
    // listen() itself rejects. Otherwise a failing listen would leave
    // a dangling Tauri subscription.
    let unlisten: UnlistenFn | undefined;
    try {
      unlisten = await attachProgressListener(setInstall);
    } catch (err) {
      setInstall((s) => ({ ...s, error: errorMessage(err), phase: "error" }));
      return;
    }

    try {
      const fetched = await fetchAndInstallRelease(
        selectedDriveMount,
        profile,
        forkRef,
      );
      if (fetched.kind === "err") {
        setInstall((s) => ({ ...s, error: fetched.message, phase: "error" }));
        return;
      }

      const { release, data } = fetched;
      const fileName = release.baseArchiveUrl.split("/").pop();
      setInstall((s) => ({
        ...s,
        log: [
          ...s.log,
          {
            step: "fetch",
            details: `Found ${forkRef.current.label} v${release.version} (${fileName})`,
            id: generateLogId(),
          },
        ],
      }));

      const valResult = await validateInstallation({
        sdMount: selectedDriveMount,
        platform: profile.platform,
        hasExtras: data.extras_files_copied > 0,
        extrasDir: profile.installPathRules.extrasDir,
      });
      setInstall((s) => ({
        ...s,
        phase: "complete",
        message: "Installation completed successfully!",
        baseFilesCopied: data.base_files_copied,
        extrasFilesCopied: data.extras_files_copied,
        romDirsCreated: data.rom_dirs_created,
        extrasWarning: data.extras_warning,
        validationResult: valResult.success ? valResult.data : null,
      }));
    } catch (err) {
      // When the user explicitly cancelled, keep the cancellation message
      // instead of overwriting it with the backend's error string.
      if (cancelledRef.current) {
        setInstall((s) => ({ ...s, phase: "error" }));
      } else {
        setInstall((s) => ({ ...s, error: errorMessage(err), phase: "error" }));
      }
    } finally {
      cancelledRef.current = false;
      unlisten?.();
    }
  }, [selectedDevice, selectedDriveMount]);

  const retryValidation = useCallback(async () => {
    if (!selectedDevice || !selectedDriveMount) return;
    const profile = getDeviceProfile(selectedDevice);
    if (!profile) return;
    const { extrasFilesCopied } = install;
    const valResult = await validateInstallation({
      sdMount: selectedDriveMount,
      platform: profile.platform,
      hasExtras: extrasFilesCopied > 0,
      extrasDir: profile.installPathRules.extrasDir,
    });
    if (valResult.success) {
      setInstall((s) => ({ ...s, validationResult: valResult.data }));
    }
  }, [selectedDevice, selectedDriveMount, install.extrasFilesCopied]);

  const updateAll = useCallback(async () => {
    if (!selectedDevice || !selectedDriveMount) return;

    const profile = getDeviceProfile(selectedDevice);
    if (!profile) return;

    setIsUpdatingAll(true);
    setUpdateAllError(null);
    setUpdateAllMessage("Starting updates...");

    const finish = (err: string | null, message: string) => {
      if (err) setUpdateAllError(err);
      setUpdateAllMessage(message);
      setIsUpdatingAll(false);
    };

    try {
      if (versionCheck?.update_available) {
        setUpdateAllMessage(`Updating ${forkRef.current.label}...`);

        const fetched = await fetchAndInstallRelease(
          selectedDriveMount,
          profile,
          forkRef,
        );
        if (fetched.kind === "err") {
          finish(fetched.message, "");
          return;
        }
      }

      if (packageUpdates.length > 0) {
        setUpdateAllMessage(`Updating ${packageUpdates.length} package(s)...`);

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

        // Package updates are independent — run them concurrently to
        // minimise wall-clock time. Each IPC call hits the Rust backend
        // independently.
        const installResults = await Promise.all(
          packageUpdates.map(async (update) => {
            const entry = packageByName.get(update.name);
            if (!entry) return `${update.name}: not found in registry`;
            const result = await installPackage({
              artifactUrl: entry.artifactUrl,
              checksum: entry.checksum || undefined,
              sdMount: selectedDriveMount,
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
      await onAfterUpdate(selectedDriveMount);
    } catch (err) {
      finish(errorMessage(err), "");
    }
  }, [
    selectedDevice,
    selectedDriveMount,
    versionCheck?.update_available,
    packageUpdates,
    onAfterUpdate,
  ]);

  const isInstalling =
    install.phase !== "idle" &&
    install.phase !== "complete" &&
    install.phase !== "error";

  // Track explicit cancellation so the backend's install-error event
  // (fired after a cancel) doesn't overwrite the user-facing message.
  const cancelledRef = useRef(false);

  const cancelAndReset = useCallback(() => {
    cancelledRef.current = true;
    void cancelInstall();
    setInstall((s) => ({
      ...s,
      phase: "error",
      error: "Installation cancelled",
      message: "",
    }));
  }, []);

  return {
    install,
    cancelInstall: cancelAndReset,
    isInstalling,
    installMinUI,
    updateAll,
    isUpdatingAll,
    updateAllMessage,
    updateAllError,
    dismissInstall,
    dismissValidation,
    retryValidation,
  };
}

function generateLogId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

async function attachProgressListener(
  setInstall: React.Dispatch<React.SetStateAction<InstallState>>,
): Promise<UnlistenFn> {
  return listen<InstallProgressEvent>("install-progress", (event) => {
    const { step, details } = event.payload;
    setInstall((s) => {
      const phase = stepToInstallPhase(step, s.phase);
      const entry = { ...event.payload, id: generateLogId() };
      return { ...s, phase, message: details, log: [...s.log, entry] };
    });
  });
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

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : "Unknown error";
}
