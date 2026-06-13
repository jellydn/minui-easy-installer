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

const DEFAULT_INSTALL_PATH_RULES: InstallPathRules = {
	baseDir: "/",
	extrasDir: "/",
	toolsDir: "/Tools",
};

const DEVICE_PROFILES: DeviceProfile[] = [
	// TrimUI
	{
		id: "trimui-brick",
		name: "TrimUI Brick",
		platform: "trimui",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "trimui-smart-pro",
		name: "TrimUI Smart Pro",
		platform: "trimui",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},

	// Miyoo
	{
		id: "miyoo-mini",
		name: "Miyoo Mini",
		platform: "miyoo",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "miyoo-mini-plus",
		name: "Miyoo Mini+",
		platform: "miyoo",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "miyoo-a30",
		name: "Miyoo A30",
		platform: "miyoo285",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "miyoo-flip",
		name: "Miyoo Flip",
		platform: "miyoo354",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "miyoo355",
		name: "Miyoo 355",
		platform: "miyoo355",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},

	// Anbernic RG35XX
	{
		id: "rg35xx",
		name: "RG35XX",
		platform: "rg35xx",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "rg35xx-plus",
		name: "RG35XX Plus",
		platform: "rg35xxplus",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "rg35xx-h",
		name: "RG35XX H",
		platform: "rg35xxplus",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "rg35xx-sp",
		name: "RG35XX SP",
		platform: "rg35xxplus",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},

	// Other supported devices
	{
		id: "m17",
		name: "M17",
		platform: "m17",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "gkdpixel",
		name: "GKD Pixel",
		platform: "gkdpixel",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "magicx",
		name: "MagicX",
		platform: "magicx",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "rgb30",
		name: "RGB30",
		platform: "rgb30",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "zero28",
		name: "Zero 28",
		platform: "zero28",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "my282",
		name: "MY282",
		platform: "my282",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
];

export function getDeviceProfile(id: string): DeviceProfile | undefined {
	return DEVICE_PROFILES.find((profile) => profile.id === id);
}

export function getAllDeviceProfiles(): DeviceProfile[] {
	return [...DEVICE_PROFILES];
}
