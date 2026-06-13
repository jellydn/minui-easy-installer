import type { DeviceProfile } from "./types/device";
import { getAllDeviceProfiles } from "./types/device";

interface DeviceSelectorProps {
  selectedDevice: string | null;
  onSelectDevice: (deviceId: string | null) => void;
}

function DeviceSelector({
  selectedDevice,
  onSelectDevice,
}: DeviceSelectorProps) {
  const devices = getAllDeviceProfiles();

  const handleSelect = (device: DeviceProfile) => {
    if (selectedDevice === device.id) {
      onSelectDevice(null);
    } else {
      onSelectDevice(device.id);
    }
  };

  return (
    <div className="device-selector">
      <h2>Select Your Device</h2>

      {devices.length === 0 && (
        <p className="empty-state">No devices available.</p>
      )}

      {devices.length > 0 && (
        <ul className="device-list">
          {devices.map((device) => (
            <li
              key={device.id}
              className={`device-item ${selectedDevice === device.id ? "selected" : ""}`}
            >
              <button type="button" onClick={() => handleSelect(device)}>
                <span className="device-name">{device.name}</span>
                <span className="device-platform">{device.platform}</span>
              </button>
            </li>
          ))}
        </ul>
      )}

      {selectedDevice && (
        <div className="selected-device">
          <p>Selected: {devices.find((d) => d.id === selectedDevice)?.name}</p>
        </div>
      )}
    </div>
  );
}

export default DeviceSelector;
