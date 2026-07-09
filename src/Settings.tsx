import { useState } from "react";
import type { ForkConfig } from "./types/fork";
import { buildCustomFork, FORK_PRESETS } from "./types/fork";

interface SettingsProps {
  selectedFork: ForkConfig;
  onSelectFork: (fork: ForkConfig) => void;
}

function Settings({ selectedFork, onSelectFork }: SettingsProps) {
  const [customInput, setCustomInput] = useState("");
  const [customError, setCustomError] = useState<string | null>(null);

  const handlePresetClick = (key: string) => {
    const fork = FORK_PRESETS[key];
    if (fork) {
      onSelectFork(fork);
      setCustomError(null);
    }
  };

  const handleCustomSubmit = () => {
    const fork = buildCustomFork(customInput);
    if (fork) {
      onSelectFork(fork);
      setCustomInput("");
      setCustomError(null);
    } else {
      setCustomError(
        'Invalid format. Use "owner/repo" (e.g. "myfork/MinUI-Mod").',
      );
    }
  };

  const currentKey =
    Object.entries(FORK_PRESETS).find(
      ([, f]) => f.owner === selectedFork.owner && f.repo === selectedFork.repo,
    )?.[0] ?? null;

  const isCustom = currentKey === null;

  return (
    <>
      <h1>Settings</h1>
      <p className="subtitle">
        Configure the installer and select your MinUI fork.
      </p>

      <div className="card">
        <h2>Release Source</h2>
        <p className="settings-description">
          Choose which fork of MinUI to install. The installer fetches releases
          from the selected GitHub repository.
        </p>

        <div className="fork-selector">
          <h3>Presets</h3>
          <div className="fork-presets">
            {Object.entries(FORK_PRESETS).map(([key, fork]) => (
              <button
                key={key}
                type="button"
                className={`fork-preset-btn ${currentKey === key ? "active" : ""}`}
                onClick={() => handlePresetClick(key)}
              >
                <span className="fork-preset-label">{fork.label}</span>
                <span className="fork-preset-repo">
                  {fork.owner}/{fork.repo}
                </span>
              </button>
            ))}
          </div>

          <h3>Custom Fork</h3>
          <div className="custom-fork-input">
            <input
              type="text"
              placeholder="owner/repo"
              aria-label="Custom fork owner and repository"
              value={customInput}
              onChange={(e) => {
                setCustomInput(e.target.value);
                setCustomError(null);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleCustomSubmit();
              }}
            />
            <button
              type="button"
              onClick={handleCustomSubmit}
              disabled={!customInput.trim()}
            >
              Use
            </button>
          </div>
          {customError && <p className="custom-error">{customError}</p>}

          {isCustom && (
            <p className="active-fork">
              Active: <strong>{selectedFork.label}</strong> (
              {selectedFork.owner}/{selectedFork.repo})
            </p>
          )}
        </div>
      </div>
    </>
  );
}

export default Settings;
