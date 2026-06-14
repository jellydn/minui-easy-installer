import { describe, expect, test } from "vitest";
import type { InstalledVersion, VersionCheckResult } from "./version";

describe("Version types", () => {
  test("InstalledVersion has correct shape", () => {
    const version: InstalledVersion = {
      version: "2024.12.25",
      source: "minui.txt",
    };

    expect(version.version).toBe("2024.12.25");
    expect(version.source).toBe("minui.txt");
  });

  test("VersionCheckResult with update available", () => {
    const result: VersionCheckResult = {
      installed: {
        version: "2024.12.25",
        source: "minui.txt",
      },
      latest: "2025.01.01",
      update_available: true,
    };

    expect(result.update_available).toBe(true);
    expect(result.installed).not.toBeNull();
    expect(result.latest).toBe("2025.01.01");
  });

  test("VersionCheckResult with no installed version", () => {
    const result: VersionCheckResult = {
      installed: null,
      latest: "2025.01.01",
      update_available: true,
    };

    expect(result.update_available).toBe(true);
    expect(result.installed).toBeNull();
  });

  test("VersionCheckResult up to date", () => {
    const result: VersionCheckResult = {
      installed: {
        version: "2025.01.01",
        source: "minui.txt",
      },
      latest: "2025.01.01",
      update_available: false,
    };

    expect(result.update_available).toBe(false);
    expect(result.installed?.version).toBe("2025.01.01");
  });
});
