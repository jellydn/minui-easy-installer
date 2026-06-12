import { useState } from "react";
import Home from "./Home";

type Screen = "home";

function App() {
  const [screen] = useState<Screen>("home");

  return <main className="container">{screen === "home" && <Home />}</main>;
}

export default App;
