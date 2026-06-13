export interface DeviceProfile {
  id: string;
  name: string;
  platform: string;
  installPathRules: InstallPathRules;
}

export interface InstallPathRules {
  baseDir: string;
  extrasDir: string;
  toolsDir: string;
}

const DEVICE_PROFILES: DeviceProfile[] = [
  {
    id: "trimui-brick",
    name: "TrimUI Brick",
    platform: "trimui-brick",
    installPathRules: {
      baseDir: "/",
      extrasDir: "/",
      toolsDir: "/Tools",
    },
  },
  {
    id: "trimui-smart-pro",
    name: "TrimUI Smart Pro",
    platform: "trimui-smart-pro",
    installPathRules: {
      baseDir: "/",
      extrasDir: "/",
      toolsDir: "/Tools",
    },
  },
  {
    id: "miyoo-mini-plus",
    name: "Miyoo Mini+",
    platform: "miyoo-mini-plus",
    installPathRules: {
      baseDir: "/",
      extrasDir: "/",
      toolsDir: "/Tools",
    },
  },
  {
    id: "miyoo-a30",
    name: "Miyoo A30",
    platform: "miyoo-a30",
    installPathRules: {
      baseDir: "/",
      extrasDir: "/",
      toolsDir: "/Tools",
    },
  },
  {
    id: "miyoo-flip",
    name: "Miyoo Flip",
    platform: "miyoo-flip",
    installPathRules: {
      baseDir: "/",
      extrasDir: "/",
      toolsDir: "/Tools",
    },
  },
  {
    id: "rg35xx-plus",
    name: "RG35XX Plus",
    platform: "rg35xx-plus",
    installPathRules: {
      baseDir: "/",
      extrasDir: "/",
      toolsDir: "/Tools",
    },
  },
  {
    id: "rg35xx-h",
    name: "RG35XX H",
    platform: "rg35xx-h",
    installPathRules: {
      baseDir: "/",
      extrasDir: "/",
      toolsDir: "/Tools",
    },
  },
  {
    id: "rg35xx-sp",
    name: "RG35XX SP",
    platform: "rg35xx-sp",
    installPathRules: {
      baseDir: "/",
      extrasDir: "/",
      toolsDir: "/Tools",
    },
  },
];

export function getDeviceProfile(id: string): DeviceProfile | undefined {
  return DEVICE_PROFILES.find((profile) => profile.id === id);
}

export function getAllDeviceProfiles(): DeviceProfile[] {
  return [...DEVICE_PROFILES];
}
