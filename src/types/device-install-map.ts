import deviceInstallMap from "./device-install-map.json";

export type DeviceId = keyof typeof deviceInstallMap.devices;

export interface DevicePak {
	name: string;
	type: "emulator" | "tool";
	description: string;
	warning?: string;
	requires?: string;
	configFile?: string;
	configFormat?: string;
}

export interface DeviceInstallRules {
	name: string;
	basePlatform: string;
	extrasPlatform: string;
	install: {
		base: { action: "copy_to_root" };
		extras: { action: "copy_platform_folders" };
	};
	devicePaks: DevicePak[];
}

export const SHARED_BIOS: readonly string[] = deviceInstallMap.sharedBios;

const devices = deviceInstallMap.devices as Record<string, DeviceInstallRules>;

export function getDeviceInstallRules(
	deviceId: string,
): DeviceInstallRules | undefined {
	return devices[deviceId];
}

export function getAllDeviceIds(): string[] {
	return Object.keys(devices);
}

export function getExtrasPlatform(deviceId: string): string | undefined {
	return devices[deviceId]?.extrasPlatform;
}

export function getBasePlatform(deviceId: string): string | undefined {
	return devices[deviceId]?.basePlatform;
}

export function getDevicePaks(deviceId: string): DevicePak[] {
	return devices[deviceId]?.devicePaks ?? [];
}

export function validateDeviceInstallMap(): {
	valid: boolean;
	errors: string[];
} {
	const errors: string[] = [];

	for (const [id, rules] of Object.entries(devices)) {
		if (!rules.name) errors.push(`${id}: missing name`);
		if (!rules.basePlatform) errors.push(`${id}: missing basePlatform`);
		if (!rules.extrasPlatform) errors.push(`${id}: missing extrasPlatform`);
		if (!rules.install) errors.push(`${id}: missing install config`);
	}

	return { valid: errors.length === 0, errors };
}
