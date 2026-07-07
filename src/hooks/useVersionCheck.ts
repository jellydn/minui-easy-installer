import { useCallback, useRef, useState } from "react";
import type { PackageUpdateInfo } from "../types/package";
import { checkPackageUpdates, fetchPackageRegistry } from "../types/package";
import { fetchMinUIRelease } from "../types/release";
import type { VersionCheckResult } from "../types/version";
import { checkMinuiVersion } from "../types/version";
import type { ForkConfig } from "../types/fork";

interface VersionCheckState {
  isChecking: boolean;
  versionCheck: VersionCheckResult | null;
  packageUpdates: PackageUpdateInfo[];
}

/**
 * Encapsulates the version-check data fetch that was previously a useEffect
 * in Home.tsx. Returns state and a `check` function to call explicitly
 * when the drive changes — converting the effect into an event-driven pattern.
 */
export function useVersionCheck(fork: ForkConfig) {
  const [state, setState] = useState<VersionCheckState>({
    isChecking: false,
    versionCheck: null,
    packageUpdates: [],
  });
  const requestIdRef = useRef(0);

  const check = useCallback(async (sdMount: string) => {
    const requestId = ++requestIdRef.current;
    setState((s) => ({
      ...s,
      isChecking: true,
      versionCheck: null,
      packageUpdates: [],
    }));

    try {
      const releaseResult = await fetchMinUIRelease(fork);
      if (requestId !== requestIdRef.current) return;

      const latestVersion = releaseResult.success
        ? releaseResult.data.version
        : undefined;

      const result = await checkMinuiVersion({
        sdMount,
        latestVersion,
        expectedPrefix: fork.versionPrefix,
      });
      if (requestId !== requestIdRef.current) return;

      if (result.success) {
        setState((s) => ({ ...s, versionCheck: result.data }));
      }

      const registryResult = await fetchPackageRegistry();
      if (requestId !== requestIdRef.current) return;

      if (registryResult.success) {
        const registryPackages: [string, string][] =
          registryResult.data.packages.map((p) => [p.name, p.version]);
        const updates = await checkPackageUpdates(sdMount, registryPackages);
        if (requestId !== requestIdRef.current) return;

        setState((s) => ({
          ...s,
          packageUpdates: updates.filter((u) => u.update_available),
        }));
      }
    } catch {
      // Version check failure is non-fatal
    } finally {
      if (requestId === requestIdRef.current) {
        setState((s) => ({ ...s, isChecking: false }));
      }
    }
  }, [fork]);

  const reset = useCallback(() => {
    requestIdRef.current++;
    setState({ isChecking: false, versionCheck: null, packageUpdates: [] });
  }, []);

  return { ...state, check, reset };
}
