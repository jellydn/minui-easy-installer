export interface RemovableDrive {
	name: string;
	mount_path: string;
	size_bytes: number | null;
	filesystem: string | null;
	available_bytes: number | null;
}

export function formatSize(bytes: number | null): string {
	if (bytes === null) return "Unknown";
	if (bytes === 0) return "0 B";

	const units = ["B", "KB", "MB", "GB", "TB"];
	const i = Math.floor(Math.log(bytes) / Math.log(1024));
	const size = bytes / 1024 ** i;

	return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function getDriveDisplayName(drive: RemovableDrive): string {
	const size = formatSize(drive.size_bytes);
	return `${drive.name} (${size})`;
}
