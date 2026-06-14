export type AppErrorCode =
	| "DOWNLOAD_ERROR"
	| "EXTRACTION_ERROR"
	| "COPY_ERROR"
	| "CHECKSUM_ERROR"
	| "UNKNOWN_ERROR";

export interface AppError {
	message: string;
	code: AppErrorCode;
}

/** Infers error code from a Rust error message string */
export function classifyError(
	errorMsg: string,
	defaultCode: AppErrorCode = "COPY_ERROR",
): AppErrorCode {
	if (errorMsg.includes("download")) return "DOWNLOAD_ERROR";
	if (errorMsg.includes("extraction") || errorMsg.includes("extract"))
		return "EXTRACTION_ERROR";
	if (errorMsg.includes("checksum")) return "CHECKSUM_ERROR";
	return defaultCode;
}
