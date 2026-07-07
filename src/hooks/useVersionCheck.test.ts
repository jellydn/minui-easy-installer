// @vitest-environment jsdom
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, type Mock, vi } from "vitest";

import { useVersionCheck } from "./useVersionCheck";
import { FORK_PRESETS } from "../types/fork";

vi.mock("../types/package", async () => {
  const actual = await import("../types/package");
  return {
    ...actual,
    checkPackageUpdates: vi.fn(),
    fetchPackageRegistry: vi.fn(),
  };
});

vi.mock("../types/release", async () => {
  const actual = await import("../types/release");
  return {
    ...actual,
    fetchMinUIRelease: vi.fn(),
  };
});

vi.mock("../types/version", async () => {
  const actual = await import("../types/version");
  return {
    ...actual,
    checkMinuiVersion: vi.fn(),
  };
});

/** Build a deferred promise whose resolver the test can call manually. */
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

describe("useVersionCheck race-condition guard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("drops the stale result when a newer check() supersedes it", async () => {
    const { fetchPackageRegistry, checkPackageUpdates } =
      await import("../types/package");
    const { fetchMinUIRelease } = await import("../types/release");
    const { checkMinuiVersion } = await import("../types/version");

    // Two deferreds so we can interleave the two in-flight `check()` calls
    // and prove that only the second one's state commits.
    const release1 = deferred<Awaited<ReturnType<typeof fetchMinUIRelease>>>();
    const release2 = deferred<Awaited<ReturnType<typeof fetchMinUIRelease>>>();
    let releaseCall = 0;
    (fetchMinUIRelease as Mock).mockImplementation(async () => {
      releaseCall += 1;
      return releaseCall === 1 ? release1.promise : release2.promise;
    });

    // Two separate version deferreds with different return values. The
    // FIRST invocation of checkMinuiVersion is the SECOND check() call
    // (the first call bails at the requestId guard BEFORE awaiting
    // checkMinuiVersion). So the first invocation returns the live
    // version2; if a guard removal caused the first call to also reach
    // here, it would get version1.
    const version1 = deferred<Awaited<ReturnType<typeof checkMinuiVersion>>>();
    const version2 = deferred<Awaited<ReturnType<typeof checkMinuiVersion>>>();
    let versionCall = 0;
    (checkMinuiVersion as Mock).mockImplementation(async () => {
      versionCall += 1;
      return versionCall === 1 ? version2.promise : version1.promise;
    });

    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: true,
      data: { version: "1.0", packages: [] },
    });
    (checkPackageUpdates as Mock).mockResolvedValue([]);

    const { result } = renderHook(() => useVersionCheck(FORK_PRESETS.official));

    // Kick off both checks synchronously. Each captures its own requestId
    // and parks at the first await (fetchMinUIRelease).
    let first!: Promise<void>;
    let second!: Promise<void>;
    act(() => {
      first = result.current.check("/sd-1");
      second = result.current.check("/sd-2");
    });

    // Resolve the FIRST call's release fetch. It should resume, notice its
    // requestId is stale, and bail — NOT committing state. We also
    // pre-resolve version2 here so that if the guard were removed and
    // the first call continued to checkMinuiVersion, it could complete
    // and the intermediate assertion below could actually fire.
    await act(async () => {
      release1.resolve({
        success: true,
        data: {
          version: "1.0.0",
          baseArchiveUrl: "",
          extrasArchiveUrl: null,
          checksums: null,
        },
      });
      version2.resolve({
        success: true,
        data: {
          installed: { version: "0.5.0", source: "minui.txt" },
          latest: "2.0.0",
          update_available: true,
        },
      });
      await first;
    });

    // Intermediate assertion: with the guard working, versionCheck stays
    // null. With the guard removed, the first call would have set
    // versionCheck to "2.0.0" here (we just resolved its release AND
    // checkMinuiVersion), and this assertion would fail.
    expect(result.current.versionCheck).toBeNull();
    // Defense-in-depth: with the guard working, the stale call bails
    // BEFORE awaiting checkMinuiVersion. The live call (second one)
    // is still parked at fetchMinUIRelease at this point, so nothing
    // has reached checkMinuiVersion yet — versionCall stays 0.
    expect(versionCall).toBe(0);

    // Now resolve the SECOND call's release. This is the live request, so
    // it must commit. (version2 was already resolved above; the live
    // call's checkMinuiVersion await now resumes immediately.)
    await act(async () => {
      release2.resolve({
        success: true,
        data: {
          version: "2.0.0",
          baseArchiveUrl: "",
          extrasArchiveUrl: null,
          checksums: null,
        },
      });
      await second;
    });

    await waitFor(() => expect(result.current.isChecking).toBe(false));

    expect(result.current.versionCheck?.latest).toBe("2.0.0");
  });

  it("reset() invalidates any in-flight check so the orphaned result cannot clobber state", async () => {
    const { fetchPackageRegistry, checkPackageUpdates } =
      await import("../types/package");
    const { fetchMinUIRelease } = await import("../types/release");
    const { checkMinuiVersion } = await import("../types/version");

    const release = deferred<Awaited<ReturnType<typeof fetchMinUIRelease>>>();
    (fetchMinUIRelease as Mock).mockReturnValue(release.promise);
    (checkMinuiVersion as Mock).mockResolvedValue({
      success: true,
      data: {
        installed: { version: "0.5.0", source: "minui.txt" },
        latest: "1.0.0",
        update_available: false,
      },
    });
    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: true,
      data: { version: "1.0", packages: [] },
    });
    (checkPackageUpdates as Mock).mockResolvedValue([]);

    const { result } = renderHook(() => useVersionCheck(FORK_PRESETS.official));

    act(() => {
      void result.current.check("/sd-1");
    });

    expect(result.current.isChecking).toBe(true);

    act(() => {
      result.current.reset();
    });

    expect(result.current.isChecking).toBe(false);
    expect(result.current.versionCheck).toBeNull();

    // Now resolve the orphaned release fetch. The hook's requestId check
    // must reject it; state must remain empty.
    await act(async () => {
      release.resolve({
        success: true,
        data: {
          version: "9.9.9",
          baseArchiveUrl: "",
          extrasArchiveUrl: null,
          checksums: null,
        },
      });
      // Let the microtask queue drain.
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(result.current.versionCheck).toBeNull();
  });

  it("re-issues check when fork prop changes", async () => {
    const { fetchPackageRegistry, checkPackageUpdates } =
      await import("../types/package");
    const { fetchMinUIRelease } = await import("../types/release");
    const { checkMinuiVersion } = await import("../types/version");

    let callCount = 0;
    (fetchMinUIRelease as Mock).mockImplementation(
      async (fork: { label: string }) => {
        callCount += 1;
        return {
          success: true,
          data: {
            version: "1.0.0",
            baseArchiveUrl: "",
            extrasArchiveUrl: null,
            checksums: null,
            fork,
          },
        };
      },
    );
    (checkMinuiVersion as Mock).mockResolvedValue({
      success: true,
      data: {
        installed: null,
        latest: "1.0.0",
        update_available: false,
      },
    });
    (fetchPackageRegistry as Mock).mockResolvedValue({
      success: true,
      data: { version: "1.0", packages: [] },
    });
    (checkPackageUpdates as Mock).mockResolvedValue([]);

    const { result, rerender } = renderHook(
      ({ fork }) => useVersionCheck(fork),
      { initialProps: { fork: FORK_PRESETS.official } },
    );

    await act(async () => {
      await result.current.check("/sd-1");
    });

    expect(callCount).toBe(1);

    // Rerender with a different fork — the check callback should be
    // recreated and the new fetch should use the new fork.
    rerender({ fork: FORK_PRESETS["minui-zero"] });

    await act(async () => {
      await result.current.check("/sd-1");
    });

    // Two fetches because the fork changed (callCount increments inside
    // the mock). With a fresh Map cache the first fetch for each fork
    // actually goes through.
    expect(callCount).toBe(2);
  });
});
