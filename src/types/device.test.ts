import { describe, expect, it } from "vitest";
import { getAllDeviceProfiles, getDeviceProfile } from "./device";

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


});
