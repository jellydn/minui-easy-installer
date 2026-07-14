import { invoke } from "@tauri-apps/api/core";

export interface BiosEntry {
  id: string;
  subdir: string;
  filename: string;
  description: string;
  system: string;
}

export interface BiosStatusEntry {
  entry: BiosEntry;
  present: boolean;
}

export interface BiosStatus {
  entries: BiosStatusEntry[];
  installed_count: number;
}

export interface BiosInstallState {
  status: "idle" | "installing" | "done" | "error";
  error?: string;
  sourceFilename?: string;
}

/**
 * Convert an ArrayBuffer to a base64 string, suitable for passing through
 * Tauri's JSON invoke layer.
 */
export function bufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  // Build the string in chunks to avoid call-stack overflow on large
  // BIOS files (a PlayStation BIOS is ~4 MB; a single String.fromCharCode
  // spread on the full buffer can blow the stack).
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode(
      ...bytes.subarray(i, Math.min(i + chunkSize, bytes.length)),
    );
  }
  return btoa(binary);
}

export async function listBiosCatalog(): Promise<BiosEntry[]> {
  return await invoke<BiosEntry[]>("list_bios_catalog");
}

export async function getBiosStatus(sdMount: string): Promise<BiosStatus> {
  return await invoke<BiosStatus>("get_bios_status", { sdMount });
}

export async function installBiosFile(
  sdMount: string,
  entryId: string,
  base64Payload: string,
): Promise<string> {
  return await invoke<string>("install_bios_file", {
    opts: {
      sdMount,
      entryId,
      base64Payload,
    },
  });
}
