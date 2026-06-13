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
	{
		id: "trimui-brick",
		name: "TrimUI Brick",
		platform: "trimui-brick",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "trimui-smart-pro",
		name: "TrimUI Smart Pro",
		platform: "trimui-smart-pro",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "miyoo-mini-plus",
		name: "Miyoo Mini+",
		platform: "miyoo-mini-plus",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "miyoo-a30",
		name: "Miyoo A30",
		platform: "miyoo-a30",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "miyoo-flip",
		name: "Miyoo Flip",
		platform: "miyoo-flip",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "rg35xx-plus",
		name: "RG35XX Plus",
		platform: "rg35xx-plus",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "rg35xx-h",
		name: "RG35XX H",
		platform: "rg35xx-h",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
	{
		id: "rg35xx-sp",
		name: "RG35XX SP",
		platform: "rg35xx-sp",
		installPathRules: { ...DEFAULT_INSTALL_PATH_RULES },
	},
];

export function getDeviceProfile(id: string): DeviceProfile | undefined {
	return DEVICE_PROFILES.find((profile) => profile.id === id);
}

export function getAllDeviceProfiles(): DeviceProfile[] {
	return [...DEVICE_PROFILES];
}
