import { describe, expect, it } from "vitest";
import { getAllDeviceProfiles, getDeviceProfile } from "./device";
import { getAllDeviceIds, getDeviceInstallRules } from "./device-install-map";

describe("getDeviceProfile", () => {
  it("returns profile for valid device id", () => {
    const profile = getDeviceProfile("trimui-brick");
    expect(profile).toBeDefined();
    expect(profile?.name).toBe("TrimUI Brick");
    expect(profile?.platform).toBe("trimui");
  });

  it("returns undefined for unknown device id", () => {
    const profile = getDeviceProfile("unknown-device");
    expect(profile).toBeUndefined();
  });

  it("returns all supported devices", () => {
    const profiles = getAllDeviceProfiles();
    expect(profiles).toHaveLength(17);
  });

  it("each profile has required fields", () => {
    const profiles = getAllDeviceProfiles();
    for (const profile of profiles) {
      expect(profile.id).toBeTruthy();
      expect(profile.name).toBeTruthy();
      expect(profile.platform).toBeTruthy();
      expect(profile.installPathRules).toBeDefined();
      expect(profile.installPathRules.baseDir).toBeTruthy();
      expect(profile.installPathRules.extrasDir).toBeTruthy();
      expect(profile.installPathRules.toolsDir).toBeTruthy();
    }
  });

  it("is in sync with device-install-map.json", () => {
    const deviceIds = getAllDeviceProfiles()
      .map((p) => p.id)
      .sort();
    const mapIds = getAllDeviceIds().sort();

    // Find devices in device.ts but not in the map
    const onlyInDevice = deviceIds.filter((id) => !mapIds.includes(id));
    // Find devices in the map but not in device.ts
    const onlyInMap = mapIds.filter((id) => !deviceIds.includes(id));

    const errors: string[] = [];
    if (onlyInDevice.length > 0) {
      errors.push(
        `device.ts has IDs not in device-install-map.json: ${onlyInDevice.join(", ")}`,
      );
    }
    if (onlyInMap.length > 0) {
      errors.push(
        `device-install-map.json has IDs not in device.ts: ${onlyInMap.join(", ")}`,
      );
    }

    // Check platform/extrasPlatform alignment
    for (const id of deviceIds) {
      if (!mapIds.includes(id)) continue;
      const profile = getDeviceProfile(id)!;
      const rules = getDeviceInstallRules(id)!;
      if (profile.platform !== rules.basePlatform) {
        errors.push(
          `${id}: device.ts platform="${profile.platform}" != map basePlatform="${rules.basePlatform}"`,
        );
      }
      if (profile.extrasPlatform !== rules.extrasPlatform) {
        errors.push(
          `${id}: device.ts extrasPlatform="${profile.extrasPlatform}" != map extrasPlatform="${rules.extrasPlatform}"`,
        );
      }
    }

    expect(errors).toEqual([]);
  });
});
