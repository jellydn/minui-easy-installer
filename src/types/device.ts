export interface DeviceProfile {
  id: string;
  name: string;
  platform: string;
  extrasPlatform: string;
  installPathRules: InstallPathRules;
}

export interface InstallPathRules {
  baseDir: string;
  extrasDir: string;
  toolsDir: string;
}

const DEFAULT_INSTALL_PATH_RULES: InstallPathRules = {
  baseDir: "/",
  extrasDir: "/",
  toolsDir: "/Tools",
};

const DEVICE_PROFILES: DeviceProfile[] = [
  // TrimUI — base archive uses "trimui" for both; TG5040 devices use extras folder "tg5040" (upstream MinUI)
  {
    id: "trimui-brick",
    name: "TrimUI Brick",
    platform: "trimui",
    extrasPlatform: "tg5040",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "trimui-smart-pro",
    name: "TrimUI Smart Pro",
    platform: "trimui",
    extrasPlatform: "tg5040",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },

  // Miyoo — base uses "miyoo"/"miyoo285"/"miyoo355", extras uses "miyoomini"/"my282"/"my355"
  {
    id: "miyoo-mini",
    name: "Miyoo Mini",
    platform: "miyoo",
    extrasPlatform: "miyoomini",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "miyoo-mini-plus",
    name: "Miyoo Mini+",
    platform: "miyoo354",
    extrasPlatform: "miyoomini",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "miyoo-a30",
    name: "Miyoo A30",
    platform: "miyoo",
    extrasPlatform: "my282",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "miyoo-flip",
    name: "Miyoo Flip",
    platform: "miyoo355",
    extrasPlatform: "my355",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "miyoo-mini-flip",
    name: "Miyoo Mini Flip",
    platform: "miyoo285",
    // Extras platform inferred from MinUI release notes; verify against the
    // target release's extras archive if installation fails.
    extrasPlatform: "miyoomini",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "miyoo355",
    name: "Miyoo 355",
    platform: "miyoo355",
    extrasPlatform: "my355",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },

  // Anbernic RG35XX — base and extras use the same name
  {
    id: "rg35xx",
    name: "RG35XX",
    platform: "rg35xx",
    extrasPlatform: "rg35xx",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "rg35xx-plus",
    name: "RG35XX Plus",
    platform: "rg35xxplus",
    extrasPlatform: "rg35xxplus",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "rg35xx-h",
    name: "RG35XX H",
    platform: "rg35xxplus",
    extrasPlatform: "rg35xxplus",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "rg35xx-sp",
    name: "RG35XX SP",
    platform: "rg35xxplus",
    extrasPlatform: "rg35xxplus",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },

  // Other supported devices — base and extras use the same name
  {
    id: "m17",
    name: "M17",
    platform: "m17",
    extrasPlatform: "m17",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "gkdpixel",
    name: "GKD Pixel",
    platform: "gkdpixel",
    extrasPlatform: "gkdpixel",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "magicx",
    name: "MagicX",
    platform: "magicx",
    extrasPlatform: "magicmini",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "rgb30",
    name: "RGB30",
    platform: "rgb30",
    extrasPlatform: "rgb30",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "zero28",
    name: "Zero 28",
    platform: "zero28",
    extrasPlatform: "zero28",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
  {
    id: "my282",
    name: "MY282",
    platform: "my282",
    extrasPlatform: "my282",
    installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
  },
];

export function getDeviceProfile(id: string): DeviceProfile | undefined {
  return DEVICE_PROFILES.find((profile) => profile.id === id);
}

export function getAllDeviceProfiles(): DeviceProfile[] {
  return [...DEVICE_PROFILES];
}
