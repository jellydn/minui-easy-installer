import { useState } from "react";
import Home from "./Home";
import PackageStore from "./PackageStore";
import type { RemovableDrive } from "./types/drive";

type Screen = "home" | "store";

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
		</main>
	);
}

export default App;
