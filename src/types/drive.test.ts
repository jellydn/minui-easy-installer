import { describe, expect, it } from "vitest";
import type { RemovableDrive } from "./drive";
import { formatSize, getDriveDisplayName } from "./drive";

describe("formatSize", () => {
  it("returns Unknown for null", () => {
    expect(formatSize(null)).toBe("Unknown");
  });

  it("returns 0 B for 0 bytes", () => {
    expect(formatSize(0)).toBe("0 B");
  });

  it("formats bytes correctly", () => {
    expect(formatSize(1023)).toBe("1023 B");
  });

  it("formats kilobytes correctly", () => {
    expect(formatSize(1024)).toBe("1.0 KB");
    expect(formatSize(1536)).toBe("1.5 KB");
  });

  it("formats megabytes correctly", () => {
    expect(formatSize(1048576)).toBe("1.0 MB");
  });

  it("formats gigabytes correctly", () => {
    expect(formatSize(32000000000)).toBe("29.8 GB");
  });
});

describe("getDriveDisplayName", () => {
  it("returns name with formatted size", () => {
    const drive: RemovableDrive = {
      name: "SD_CARD",
      mount_path: "/Volumes/SD_CARD",
      size_bytes: 32000000000,
      filesystem: "FAT32",
      available_bytes: 28000000000,
    };

    const result = getDriveDisplayName(drive);
    expect(result).toContain("SD_CARD");
    expect(result).toContain("GB");
  });

  it("handles null size", () => {
    const drive: RemovableDrive = {
      name: "UNKNOWN",
      mount_path: "/Volumes/UNKNOWN",
      size_bytes: null,
      filesystem: null,
      available_bytes: null,
    };

    const result = getDriveDisplayName(drive);
    expect(result).toContain("UNKNOWN");
    expect(result).toContain("Unknown");
  });
});
