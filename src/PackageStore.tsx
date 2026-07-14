import { useCallback, useMemo, useState } from "react";
import { useMountEffect } from "./hooks/useMountEffect";
import PackageCard from "./PackageCard";
import { getDeviceProfile } from "./types/device";
import type { PackageInstallState } from "./types/install";
import type {
  PackageCategory,
  PackageRegistry,
  PackageRegistryEntry,
} from "./types/package";
import { fetchPackageRegistry, installPackage } from "./types/package";

interface PackageStoreProps {
  selectedDevice: string | null;
  selectedDrive: string | null;
}

const ALL_CATEGORIES: PackageCategory[] = ["Utilities", "Emulators"];

function PackageStore({ selectedDevice, selectedDrive }: PackageStoreProps) {
  const extrasPlatform = useMemo(() => {
    const profile = selectedDevice ? getDeviceProfile(selectedDevice) : null;
    return profile?.extrasPlatform || selectedDevice || "{platform}";
  }, [selectedDevice]);

  const [registry, setRegistry] = useState<PackageRegistry | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState<
    PackageCategory | "All"
  >("All");
  const [installStates, setInstallStates] = useState<
    Record<string, PackageInstallState>
  >({});

  const loadRegistry = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    const result = await fetchPackageRegistry();

    if (result.success) {
      setRegistry(result.data);
    } else {
      setError(result.error.message);
    }

    setIsLoading(false);
  }, []);

  useMountEffect(() => {
    loadRegistry();
  });

  const filteredPackages = useMemo(() => {
    if (!registry) return [];

    let packages = registry.packages;

    if (selectedDevice) {
      packages = packages.filter(
        (pkg) =>
          pkg.supportedDevices.length === 0 ||
          pkg.supportedDevices.includes(selectedDevice),
      );
    }

    if (selectedCategory !== "All") {
      packages = packages.filter((pkg) => pkg.category === selectedCategory);
    }

    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      packages = packages.filter(
        (pkg) =>
          pkg.name.toLowerCase().includes(query) ||
          pkg.description.toLowerCase().includes(query),
      );
    }

    return packages;
  }, [registry, selectedDevice, selectedCategory, searchQuery]);

  const handleRetry = () => {
    loadRegistry();
  };

  const handleInstall = useCallback(
    async (pkg: PackageRegistryEntry) => {
      if (!selectedDevice || !selectedDrive) return;

      setInstallStates((prev) => ({
        ...prev,
        [pkg.name]: { status: "installing" },
      }));

      const result = await installPackage({
        artifactUrl: pkg.artifactUrl,
        checksum: pkg.checksum || undefined,
        sdMount: selectedDrive,
        targetDir: pkg.installPathRules.targetDir,
        extractToRoot: pkg.installPathRules.extractToRoot,
        pakName: pkg.installPathRules.pakName || pkg.name.replace(/\s+/g, "."),
        platform: extrasPlatform,
      });

      setInstallStates((prev) => ({
        ...prev,
        [pkg.name]: result.success
          ? { status: "done" }
          : { status: "error", error: result.error.message },
      }));
    },
    [selectedDevice, selectedDrive, extrasPlatform],
  );

  const handleInstallAll = useCallback(async () => {
    const pending = filteredPackages.filter(
      (pkg) => installStates[pkg.name]?.status !== "done",
    );

    const installing: Record<string, PackageInstallState> = {};
    for (const pkg of pending) {
      installing[pkg.name] = { status: "installing" };
    }
    setInstallStates((prev) => ({ ...prev, ...installing }));

    await Promise.allSettled(pending.map((pkg) => handleInstall(pkg)));
  }, [filteredPackages, installStates, handleInstall]);

  const installCounts = useMemo(() => {
    const states = Object.values(installStates);
    return {
      done: states.filter((s) => s.status === "done").length,
      installing: states.filter((s) => s.status === "installing").length,
      error: states.filter((s) => s.status === "error").length,
    };
  }, [installStates]);

  if (isLoading) {
    return (
      <div className="screen">
        <h1>Package Store</h1>
        <p className="subtitle">
          Browse and install add-on packages for your MinUI device.
        </p>
        <div className="store-loading">
          <div className="install-spinner" />
          <p>Loading packages...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="screen">
        <h1>Package Store</h1>
        <p className="subtitle">
          Browse and install add-on packages for your MinUI device.
        </p>
        <div className="store-error">
          <p className="error">Failed to load packages: {error}</p>
          <button type="button" onClick={handleRetry}>
            Retry
          </button>
        </div>
      </div>
    );
  }

  const hasPendingPackages = filteredPackages.some(
    (pkg) =>
      !installStates[pkg.name] || installStates[pkg.name]?.status === "idle",
  );

  return (
    <div className="screen">
      <h1>Package Store</h1>
      <p className="subtitle">
        Browse and install add-on packages for your MinUI device.
      </p>

      <div className="store-controls">
        <input
          type="text"
          placeholder="Search packages..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="search-input"
        />

        <div className="category-filters">
          <button
            type="button"
            className={`category-btn ${selectedCategory === "All" ? "active" : ""}`}
            onClick={() => setSelectedCategory("All")}
          >
            All
          </button>
          {ALL_CATEGORIES.map((category) => (
            <button
              key={category}
              type="button"
              className={`category-btn ${selectedCategory === category ? "active" : ""}`}
              onClick={() => setSelectedCategory(category)}
            >
              {category}
            </button>
          ))}
        </div>
      </div>

      {installCounts.installing > 0 && (
        <div className="batch-progress">
          <div className="install-spinner" />
          <p>
            Installing {installCounts.done + installCounts.installing} of{" "}
            {installCounts.done +
              installCounts.installing +
              installCounts.error}{" "}
            packages...
          </p>
        </div>
      )}

      {filteredPackages.length === 0 ? (
        <div className="store-empty">
          <p>
            {searchQuery
              ? `No packages found matching "${searchQuery}"`
              : "No packages available"}
          </p>
        </div>
      ) : (
        <>
          {hasPendingPackages && (
            <button
              type="button"
              className="install-all-btn"
              onClick={handleInstallAll}
              disabled={installCounts.installing > 0 || !selectedDrive}
            >
              {installCounts.installing > 0 ? "Installing..." : "Install All"}
            </button>
          )}
          <div className="package-grid">
            {filteredPackages.map((pkg) => (
              <PackageCard
                key={pkg.name}
                pkg={pkg}
                installState={installStates[pkg.name] || { status: "idle" }}
                onInstall={handleInstall}
                canInstall={!!selectedDrive}
                extrasPlatform={extrasPlatform}
              />
            ))}
          </div>
        </>
      )}
    </div>
  );
}

export default PackageStore;
