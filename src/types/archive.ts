export interface DownloadResult {
	success: boolean;
	file_path: string | null;
	checksum_verified: boolean | null;
	error: string | null;
}

export interface DownloadProgress {
	bytes_downloaded: number;
	total_bytes: number | null;
	percentage: number | null;
}

export type DownloadError = {
	message: string;
	code: "NETWORK_ERROR" | "CHECKSUM_ERROR" | "FILE_ERROR" | "UNKNOWN_ERROR";
};

export type DownloadResultEither =
	| { success: true; data: DownloadResult }
	| { success: false; error: DownloadError };

export async function downloadArchive(
	url: string,
	checksum?: string,
): Promise<DownloadResultEither> {
	try {
		const { invoke } = await import("@tauri-apps/api/core");
		const result = await invoke<DownloadResult>("download_and_verify_archive", {
			url,
			checksum: checksum || null,
		});

		if (result.success) {
			return { success: true, data: result };
		}

		return {
			success: false,
			error: {
				message: result.error || "Download failed",
				code:
					result.checksum_verified === false
						? "CHECKSUM_ERROR"
						: "NETWORK_ERROR",
			},
		};
	} catch (err) {
		const message = err instanceof Error ? err.message : "Unknown error";
		return {
			success: false,
			error: { message, code: "UNKNOWN_ERROR" },
		};
	}
}

export async function verifyChecksum(
	filePath: string,
	expectedChecksum: string,
): Promise<{ success: boolean; verified: boolean; error?: string }> {
	try {
		const { invoke } = await import("@tauri-apps/api/core");
		const verified = await invoke<boolean>("verify_archive_checksum", {
			filePath,
			expectedChecksum,
		});
		return { success: true, verified };
	} catch (err) {
		const message = err instanceof Error ? err.message : "Unknown error";
		return { success: false, verified: false, error: message };
	}
}
