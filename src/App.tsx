import { useState } from "react";
import BiosInstaller from "./BiosInstaller";
import { ForkProvider, useFork } from "./contexts/ForkContext";
import Home from "./Home";
import PackageStore from "./PackageStore";
import Settings from "./Settings";
import type { RemovableDrive } from "./types/drive";
import WifiWizard from "./WifiWizard";

type Screen = "home" | "store" | "wifi" | "bios" | "settings";

function App() {
  return (
    <ForkProvider>
      <AppShell />
    </ForkProvider>
  );
}

function AppShell() {
  const [screen, setScreen] = useState<Screen>("home");
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null);
  const [selectedDrive, setSelectedDrive] = useState<RemovableDrive | null>(
    null,
  );
  const { fork, setFork } = useFork();

  return (
    <main className="container">
      <nav className="app-nav">
        <button
          type="button"
          className={`nav-btn ${screen === "home" ? "active" : ""}`}
          onClick={() => setScreen("home")}
        >
          Home
        </button>
        <button
          type="button"
          className={`nav-btn ${screen === "store" ? "active" : ""}`}
          onClick={() => setScreen("store")}
        >
          Package Store
        </button>
        <button
          type="button"
          className={`nav-btn ${screen === "wifi" ? "active" : ""}`}
          onClick={() => setScreen("wifi")}
        >
          WiFi Setup
        </button>
        <button
          type="button"
          className={`nav-btn ${screen === "bios" ? "active" : ""}`}
          onClick={() => setScreen("bios")}
        >
          BIOS
        </button>
        <button
          type="button"
          className={`nav-btn ${screen === "settings" ? "active" : ""}`}
          onClick={() => setScreen("settings")}
        >
          Settings
        </button>
      </nav>

      {screen === "home" && (
        <Home
          selectedDevice={selectedDevice}
          onSelectDevice={setSelectedDevice}
          selectedDrive={selectedDrive}
          onSelectDrive={setSelectedDrive}
        />
      )}

      {screen === "store" && selectedDevice && selectedDrive ? (
        <PackageStore
          selectedDevice={selectedDevice}
          selectedDrive={selectedDrive.mount_path}
        />
      ) : screen === "store" ? (
        <div className="screen">
          <h1>Package Store</h1>
          <p className="subtitle">
            Browse and install add-on packages for your MinUI device.
          </p>
          <div className="prerequisite-message">
            <p>Select a device and SD card on the Home screen first.</p>
            <button type="button" onClick={() => setScreen("home")}>
              Go to Home
            </button>
          </div>
        </div>
      ) : null}

      {screen === "wifi" && selectedDrive ? (
        <div className="screen">
          <WifiWizard
            sdMount={selectedDrive.mount_path}
            onComplete={() => setScreen("home")}
            onCancel={() => setScreen("home")}
          />
        </div>
      ) : screen === "wifi" ? (
        <div className="screen">
          <h1>WiFi Setup</h1>
          <p className="subtitle">
            Configure WiFi credentials for your device.
          </p>
          <div className="prerequisite-message">
            <p>Select an SD card on the Home screen first.</p>
            <button type="button" onClick={() => setScreen("home")}>
              Go to Home
            </button>
          </div>
        </div>
      ) : null}

      {screen === "bios" && selectedDrive ? (
        <div className="screen">
          <BiosInstaller
            sdMount={selectedDrive.mount_path}
            onClose={() => setScreen("home")}
          />
        </div>
      ) : screen === "bios" ? (
        <div className="screen">
          <h1>BIOS Files</h1>
          <p className="subtitle">
            Install copyrighted BIOS files onto your SD card.
          </p>
          <div className="prerequisite-message">
            <p>Select an SD card on the Home screen first.</p>
            <button type="button" onClick={() => setScreen("home")}>
              Go to Home
            </button>
          </div>
        </div>
      ) : null}

      {screen === "settings" && (
        <div className="screen">
          <Settings selectedFork={fork} onSelectFork={setFork} />
        </div>
      )}
    </main>
  );
}

export default App;
