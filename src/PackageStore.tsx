import { useCallback, useEffect, useMemo, useState } from "react";
import { getDeviceProfile } from "./types/device";
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

const ALL_CATEGORIES: PackageCategory[] = [
	"Utilities",
	"Emulators",
	"Network",
	"Community",
];

function PackageStore({ selectedDevice, selectedDrive }: PackageStoreProps) {
	const [registry, setRegistry] = useState<PackageRegistry | null>(null);
	const [isLoading, setIsLoading] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [searchQuery, setSearchQuery] = useState("");
	const [selectedCategory, setSelectedCategory] = useState<
		PackageCategory | "All"
	>("All");
	const [installedPackages, setInstalledPackages] = useState<Set<string>>(
		new Set(),
	);
	const [installingPackage, setInstallingPackage] = useState<string | null>(
		null,
	);
	const [installError, setInstallError] = useState<string | null>(null);

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

	useEffect(() => {
		loadRegistry();
	}, [loadRegistry]);

	const filteredPackages = useMemo(() => {
		if (!registry) return [];

		let packages = registry.packages;

		// Filter by selected device
		if (selectedDevice) {
			packages = packages.filter(
				(pkg) =>
					pkg.supportedDevices.length === 0 ||
					pkg.supportedDevices.includes(selectedDevice),
			);
		}

		// Filter by category
		if (selectedCategory !== "All") {
			packages = packages.filter((pkg) => pkg.category === selectedCategory);
		}

		// Filter by search query
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
			if (!selectedDevice || !selectedDrive) {
				setInstallError("Please select a device and drive first");
				return;
			}

			const profile = getDeviceProfile(selectedDevice);
			if (!profile) {
				setInstallError("Unknown device profile");
				return;
			}

			setInstallingPackage(pkg.name);
			setInstallError(null);

			const result = await installPackage({
				artifactUrl: pkg.artifactUrl,
				checksum: pkg.checksum || undefined,
				sdMount: selectedDrive,
				targetDir: pkg.installPathRules.targetDir,
				extractToRoot: pkg.installPathRules.extractToRoot,
			});

			if (result.success) {
				setInstalledPackages((prev) => new Set(prev).add(pkg.name));
			} else {
				setInstallError(
					`Failed to install ${pkg.name}: ${result.error.message}`,
				);
			}

			setInstallingPackage(null);
		},
		[selectedDevice, selectedDrive],
	);

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

	return (
		<div className="screen">
			<h1>Package Store</h1>
			<p className="subtitle">
				Browse and install add-on packages for your MinUI device.
			</p>

			{installError && (
				<div className="store-error">
					<p className="error">{installError}</p>
					<button type="button" onClick={() => setInstallError(null)}>
						Dismiss
					</button>
				</div>
			)}

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

			{filteredPackages.length === 0 ? (
				<div className="store-empty">
					<p>
						{searchQuery
							? `No packages found matching "${searchQuery}"`
							: "No packages available"}
					</p>
				</div>
			) : (
				<div className="package-grid">
					{filteredPackages.map((pkg) => (
						<PackageCard
							key={pkg.name}
							package={pkg}
							isInstalled={installedPackages.has(pkg.name)}
							isInstalling={installingPackage === pkg.name}
							onInstall={handleInstall}
							canInstall={!!selectedDevice && !!selectedDrive}
						/>
					))}
				</div>
			)}
		</div>
	);
}

interface PackageCardProps {
	package: PackageRegistryEntry;
	isInstalled: boolean;
	isInstalling: boolean;
	onInstall: (pkg: PackageRegistryEntry) => void;
	canInstall: boolean;
}

function PackageCard({
	package: pkg,
	isInstalled,
	isInstalling,
	onInstall,
	canInstall,
}: PackageCardProps) {
	return (
		<div className="package-card">
			<div className="package-header">
				<h3 className="package-name">{pkg.name}</h3>
				<span className="package-version">v{pkg.version}</span>
			</div>
			<p className="package-author">by {pkg.author}</p>
			<span className="package-category">{pkg.category}</span>
			<p className="package-description">{pkg.description}</p>
			<div className="package-meta">
				{pkg.downloads !== null && (
					<span className="package-downloads">
						{pkg.downloads.toLocaleString()} downloads
					</span>
				)}
				{pkg.rating !== null && (
					<span className="package-rating">
						{"★".repeat(Math.round(pkg.rating))}
					</span>
				)}
			</div>
			{isInstalled ? (
				<span className="installed-badge">Installed</span>
			) : (
				<button
					type="button"
					className="install-btn"
					onClick={() => onInstall(pkg)}
					disabled={isInstalling || !canInstall}
				>
					{isInstalling ? "Installing..." : "Install"}
				</button>
			)}
		</div>
	);
}

export default PackageStore;
