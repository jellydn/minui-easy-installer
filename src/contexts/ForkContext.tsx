import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { ForkConfig } from "../types/fork";
import { FORK_PRESETS, rehydrateFork } from "../types/fork";

const STORAGE_KEY = "selectedFork:v1";
const LEGACY_STORAGE_KEY = "selectedFork";

/** Load the persisted fork selection from localStorage, or default to official. */
function loadPersistedFork(): ForkConfig {
  try {
    let raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      raw = localStorage.getItem(LEGACY_STORAGE_KEY);
      if (raw) {
        localStorage.setItem(STORAGE_KEY, raw);
        localStorage.removeItem(LEGACY_STORAGE_KEY);
      }
    }
    if (raw) {
      const fork = rehydrateFork(JSON.parse(raw));
      if (fork) return fork;
    }
  } catch {
    // Corrupt localStorage — fall through to default
  }
  return FORK_PRESETS.official;
}

interface ForkContextValue {
  fork: ForkConfig;
  setFork: (fork: ForkConfig) => void;
}

const ForkContext = createContext<ForkContextValue | null>(null);

export function ForkProvider({ children }: { children: ReactNode }) {
  const [fork, setForkState] = useState<ForkConfig>(loadPersistedFork);

  const setFork = useCallback((next: ForkConfig) => {
    setForkState(next);
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
      // localStorage write failed — non-fatal
    }
  }, []);

  const value = useMemo(() => ({ fork, setFork }), [fork, setFork]);

  return <ForkContext.Provider value={value}>{children}</ForkContext.Provider>;
}

export function useFork(): ForkContextValue {
  const ctx = useContext(ForkContext);
  if (!ctx) {
    throw new Error("useFork must be used inside <ForkProvider>");
  }
  return ctx;
}
