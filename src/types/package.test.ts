import { describe, expect, test } from "vitest";
import { fetchPackageRegistry } from "./package";

describe("fetchPackageRegistry", () => {
  test("loads and converts local store.json", async () => {
    const result = await fetchPackageRegistry();

    expect(result.success).toBe(true);
    if (!result.success) return;

    expect(result.data.version).toBe("1.0.0");
    expect(result.data.packages.length).toBeGreaterThan(0);

    // Verify all packages are present
    const names = result.data.packages.map((p) => p.name);

    // Emu paks
    expect(names).toContain("Dreamcast");
    expect(names).toContain("Nintendo DS");
    expect(names).toContain("N64");
    expect(names).toContain("Pico-8");
    expect(names).toContain("Portmaster");
    expect(names).toContain("PSP");
    expect(names).toContain("MAME 2003 Plus");

    // Tool paks
    expect(names).toContain("Wifi");
    expect(names).toContain("SSH Server");
    expect(names).toContain("Grout");
    expect(names).toContain("Syncthing");

    // Verify Grout uses custom download_url
    const grout = result.data.packages.find((p) => p.name === "Grout")!;
    expect(grout.artifactUrl).toBe(
      "https://github.com/rommapp/grout/releases/download/v4.8.1.0/Grout-MinUI.zip",
    );
    expect(grout.category).toBe("Utilities");

    // Verify emu paks go to Emus
    const dc = result.data.packages.find((p) => p.name === "Dreamcast")!;
    expect(dc.category).toBe("Emulators");
    expect(dc.installPathRules.targetDir).toBe("/Emus");

    // Verify device-filtered tools have supportedDevices
    const mediaPlayer = result.data.packages.find(
      (p) => p.name === "Media Player",
    )!;
    expect(mediaPlayer.supportedDevices).toEqual(["brick"]);

    // Verify default tools have empty supportedDevices
    const wifi = result.data.packages.find((p) => p.name === "Wifi")!;
    expect(wifi.supportedDevices).toEqual([]);
  });
});
