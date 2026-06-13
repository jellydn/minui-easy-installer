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

export type InstallErrorCode = InstallError["code"];

/** Infers error code from a Rust error message string */
export function classifyError(
	errorMsg: string,
	defaultCode: InstallErrorCode = "COPY_ERROR",
): InstallErrorCode {
	if (errorMsg.includes("download")) return "DOWNLOAD_ERROR";
	if (errorMsg.includes("extraction") || errorMsg.includes("extract"))
		return "EXTRACTION_ERROR";
	if (errorMsg.includes("checksum")) return "CHECKSUM_ERROR";
	return defaultCode;
}

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
		const code = classifyError(errorMsg);

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
