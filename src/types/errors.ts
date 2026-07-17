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

/**
 * Extract a human-readable message from any error value.
 *
 * Tauri v2's {@link https://v2.tauri.app/reference/javascript/core/#invoke | `invoke()`}
 * rejects with a plain string when a Rust command returns `Err(String)` — not
 * an `Error` instance. This helper normalises `Error`, `string`, and
 * `{ message }` values so callers never produce "Unknown error" from a
 * perfectly descriptive error.
 *
 * @example
 * errorMessage(new Error("boom"))         // "boom"
 * errorMessage("plain string")            // "plain string"
 * errorMessage({ message: "hi" })          // "hi"
 * errorMessage(null)                      // "Unknown error"
 */
export function errorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  if (err && typeof err === "object" && "message" in err) {
    const msg = (err as { message: unknown }).message;
    // Only coerce primitives — avoid "null" / "[object Object]" noise
    if (typeof msg === "string") return msg;
    if (typeof msg === "number" || typeof msg === "boolean") return String(msg);
  }
  return "Unknown error";
}

/**
 * Wrap any rejection value in an `Error` so it carries a proper stack trace.
 *
 * Use this when rejecting a promise with a value that may not be an `Error`
 * (e.g. a Tauri `invoke()` plain-string rejection).
 */
export function asError(err: unknown): Error {
  return err instanceof Error ? err : new Error(errorMessage(err));
}
