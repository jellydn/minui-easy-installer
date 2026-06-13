import { useState } from "react";
import Home from "./Home";
import PackageStore from "./PackageStore";
import type { RemovableDrive } from "./types/drive";
import WifiWizard from "./WifiWizard";

type Screen = "home" | "store" | "wifi";

function App() {
	const [screen, setScreen] = useState<Screen>("home");
	const [selectedDevice, setSelectedDevice] = useState<string | null>(null);
	const [selectedDrive, setSelectedDrive] = useState<RemovableDrive | null>(
		null,
	);

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
			</nav>

			{screen === "home" && (
				<Home
					selectedDevice={selectedDevice}
					onSelectDevice={setSelectedDevice}
					selectedDrive={selectedDrive}
					onSelectDrive={setSelectedDrive}
				/>
			)}

			{screen === "store" && (
				<PackageStore
					selectedDevice={selectedDevice}
					selectedDrive={selectedDrive?.mount_path || null}
				/>
			)}

			{screen === "wifi" && (
				<div className="card">
					<WifiWizard
						sdMount={selectedDrive?.mount_path || null}
						onComplete={() => setScreen("home")}
						onCancel={() => setScreen("home")}
					/>
				</div>
			)}
		</main>
	);
}

export default App;
