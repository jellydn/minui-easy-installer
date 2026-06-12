import { useState } from "react";
import Home from "./Home";
import type { RemovableDrive } from "./types/drive";

type Screen = "home";

function App() {
	const [screen] = useState<Screen>("home");
	const [selectedDevice, setSelectedDevice] = useState<string | null>(null);
	const [selectedDrive, setSelectedDrive] = useState<RemovableDrive | null>(
		null,
	);

	return (
		<main className="container">
			{screen === "home" && (
				<Home
					selectedDevice={selectedDevice}
					onSelectDevice={setSelectedDevice}
					selectedDrive={selectedDrive}
					onSelectDrive={setSelectedDrive}
				/>
			)}
		</main>
	);
}

export default App;
