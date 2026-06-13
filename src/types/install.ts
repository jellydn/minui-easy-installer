export interface InstallResult {
	success: boolean;
	error: string | null;
	base_files_copied: number;
	extras_files_copied: number;
	extras_warning: string | null;
}

export type InstallError = {
	message: string;
	code:
		| "DOWNLOAD_ERROR"
		| "EXTRACTION_ERROR"
		| "COPY_ERROR"
		| "CHECKSUM_ERROR"
		| "UNKNOWN_ERROR";
};

export type InstallResultEither =
	| { success: true; data: InstallResult }
	| { success: false; error: InstallError };

export type InstallPhase =
	| "idle"
	| "downloading"
	| "extracting"
	| "copying"
	| "complete"
	| "error";

export async function installMinui(options: {
	baseUrl: string;
	extrasUrl?: string;
	baseChecksum?: string;
	extrasChecksum?: string;
	sdMount: string;
	platform: string;
	extrasDir: string;
	version: string;
}): Promise<InstallResultEither> {
	try {
		const { invoke } = await import("@tauri-apps/api/core");
		const result = await invoke<InstallResult>("install_minui", {
			baseUrl: options.baseUrl,
			extrasUrl: options.extrasUrl || null,
			baseChecksum: options.baseChecksum || null,
			extrasChecksum: options.extrasChecksum || null,
			sdMount: options.sdMount,
			platform: options.platform,
			extrasDir: options.extrasDir,
			version: options.version,
		});

		if (result.success) {
			return { success: true, data: result };
		}

		const errorMsg = result.error || "Installation failed";
		let code: InstallError["code"] = "COPY_ERROR";

		if (errorMsg.includes("download")) {
			code = "DOWNLOAD_ERROR";
		} else if (
			errorMsg.includes("extraction") ||
			errorMsg.includes("extract")
		) {
			code = "EXTRACTION_ERROR";
		} else if (errorMsg.includes("checksum")) {
			code = "CHECKSUM_ERROR";
		}

		return {
			success: false,
			error: { message: errorMsg, code },
		};
	} catch (err) {
		const message = err instanceof Error ? err.message : "Unknown error";
		return {
			success: false,
			error: { message, code: "UNKNOWN_ERROR" },
		};
	}
}
