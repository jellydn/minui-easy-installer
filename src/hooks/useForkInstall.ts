import { useCallback, useEffect, useRef, useState } from "react";
import { useFork } from "../contexts/ForkContext";
import {
  INITIAL_INSTALL_STATE,
  InstallOrchestrator,
  type OrchestratorState,
} from "../lib/InstallOrchestrator";
import type { VersionCheckResult } from "../types/version";

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
  install: OrchestratorState["install"];
  isInstalling: boolean;
  installMinUI: () => Promise<void>;
  cancelInstall: () => void;
  updateAll: () => Promise<void>;
  isUpdatingAll: boolean;
  updateAllMessage: string;
  updateAllError: string | null;
  dismissInstall: () => void;
  dismissValidation: () => void;
  retryValidation: () => Promise<void>;
}

/**
 * Thin React adapter over `InstallOrchestrator`.
 *
 * Creates one orchestrator per hook instance (via useRef), subscribes
 * to state changes, and syncs them to React state. The orchestrator
 * owns all Tauri event listener lifecycle and business logic.
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

  const [state, setState] = useState<OrchestratorState>({
    install: INITIAL_INSTALL_STATE,
    updateAll: { isUpdatingAll: false, message: "", error: null },
  });

  const orchestratorRef = useRef<InstallOrchestrator | null>(null);
  if (!orchestratorRef.current) {
    orchestratorRef.current = new InstallOrchestrator();
  }
  const orch = orchestratorRef.current;

  // Subscribe to orchestrator state changes.
  useEffect(() => {
    return orch.subscribe(setState);
  }, [orch]);

  const installMinUI = useCallback(async () => {
    if (!selectedDevice || !selectedDriveMount) return;
    await orch.start(fork, selectedDevice, selectedDriveMount);
  }, [orch, fork, selectedDevice, selectedDriveMount]);

  const cancelInstall = useCallback(() => {
    orch.cancel();
  }, [orch]);

  const updateAll = useCallback(async () => {
    if (!selectedDevice || !selectedDriveMount) return;
    await orch.updateAll(
      fork,
      selectedDevice,
      selectedDriveMount,
      versionCheck,
      packageUpdates,
      onAfterUpdate,
    );
  }, [
    orch,
    fork,
    selectedDevice,
    selectedDriveMount,
    versionCheck,
    packageUpdates,
    onAfterUpdate,
  ]);

  const retryValidation = useCallback(async () => {
    if (!selectedDevice || !selectedDriveMount) return;
    await orch.retryValidation(selectedDevice, selectedDriveMount);
  }, [orch, selectedDevice, selectedDriveMount]);

  const dismissInstall = useCallback(() => {
    orch.dismissInstall();
  }, [orch]);

  const dismissValidation = useCallback(() => {
    orch.dismissValidation();
  }, [orch]);

  return {
    install: state.install,
    isInstalling: orch.isInstalling,
    installMinUI,
    cancelInstall,
    updateAll,
    isUpdatingAll: state.updateAll.isUpdatingAll,
    updateAllMessage: state.updateAll.message,
    updateAllError: state.updateAll.error,
    dismissInstall,
    dismissValidation,
    retryValidation,
  };
}
