import { useCallback, useRef, useState } from "react";
import type { PackageUpdateInfo } from "../types/package";
import { checkPackageUpdates, fetchPackageRegistry } from "../types/package";
import { fetchMinUIRelease } from "../types/release";
import type { VersionCheckResult } from "../types/version";
import { checkMinuiVersion } from "../types/version";

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
export function useVersionCheck() {
	const [state, setState] = useState<VersionCheckState>({
		isChecking: false,
		versionCheck: null,
		packageUpdates: [],
	});
	const cancelledRef = useRef(false);

	const check = useCallback(async (sdMount: string) => {
		cancelledRef.current = false;
		setState((s) => ({
			...s,
			isChecking: true,
			versionCheck: null,
			packageUpdates: [],
		}));

		try {
			const releaseResult = await fetchMinUIRelease();
			const latestVersion = releaseResult.success
				? releaseResult.data.version
				: undefined;

			const result = await checkMinuiVersion({ sdMount, latestVersion });
			if (!cancelledRef.current && result.success) {
				setState((s) => ({ ...s, versionCheck: result.data }));
			}

			const registryResult = await fetchPackageRegistry();
			if (!cancelledRef.current && registryResult.success) {
				const registryPackages: [string, string][] =
					registryResult.data.packages.map((p) => [p.name, p.version]);
				const updates = await checkPackageUpdates(sdMount, registryPackages);
				if (!cancelledRef.current) {
					setState((s) => ({
						...s,
						packageUpdates: updates.filter((u) => u.update_available),
					}));
				}
			}
		} catch {
			// Version check failure is non-fatal
		} finally {
			if (!cancelledRef.current) {
				setState((s) => ({ ...s, isChecking: false }));
			}
		}
	}, []);

	const reset = useCallback(() => {
		cancelledRef.current = true;
		setState({ isChecking: false, versionCheck: null, packageUpdates: [] });
	}, []);

	return { ...state, check, reset };
}
