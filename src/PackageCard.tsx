import type { PackageRegistryEntry } from "./types/package";

interface PackageCardProps {
	package: PackageRegistryEntry;
	installState: PackageInstallState;
	onInstall: (pkg: PackageRegistryEntry) => void;
	canInstall: boolean;
	extrasPlatform: string;
}

interface PackageInstallState {
	status: "idle" | "installing" | "done" | "error";
	error?: string;
}

function installDestination(
	pkg: PackageRegistryEntry,
	platform: string,
): string {
	const baseDir = pkg.category === "Emulators" ? "Emus" : "Tools";
	const pakName = pkg.installPathRules.pakName || pkg.name.replace(/\s+/g, ".");
	return `${baseDir}/${platform}/${pakName}.pak/`;
}

function PackageCard({
	package: pkg,
	installState,
	onInstall,
	canInstall,
	extrasPlatform,
}: PackageCardProps) {
	const destLabel = installDestination(pkg, extrasPlatform);

	return (
		<div className="package-card">
			<div className="package-header">
				<h3 className="package-name">{pkg.name}</h3>
				<span className="package-version">v{pkg.version}</span>
			</div>
			{pkg.repository && (
				<a
					href={pkg.repository}
					target="_blank"
					rel="noopener noreferrer"
					className="package-link"
				>
					{pkg.repository.replace("https://github.com/", "")}
				</a>
			)}
			<span
				className={`package-category category-${pkg.category.toLowerCase()}`}
			>
				{pkg.category}
			</span>
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
			<p className="package-destination">
				Installs to: <code>{destLabel}</code>
			</p>
			{installState.status === "done" ? (
				<span className="installed-badge">Installed</span>
			) : installState.status === "installing" ? (
				<div className="installing-progress">
					<div className="install-spinner" />
					<span>Installing...</span>
				</div>
			) : (
				<button
					type="button"
					className="install-btn"
					onClick={() => onInstall(pkg)}
					disabled={!canInstall}
				>
					Install
				</button>
			)}
			{installState.status === "error" && installState.error && (
				<p className="error">{installState.error}</p>
			)}
		</div>
	);
}

export default PackageCard;
