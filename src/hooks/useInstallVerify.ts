import { useEffect } from "react";
import { useGamesStore } from "@/stores/gamesStore";

/** Minimum pause between two focus-triggered verification runs. */
const FOCUS_MIN_INTERVAL = 30_000;
/** Delay before the first run so the initial library load is not competing. */
const STARTUP_DELAY = 1_500;

/**
 * Re-checks that every game in the library still exists on disk.
 * Uninstalled games keep their DB rows and are only flagged as not installed.
 */
export async function verifyInstallations(): Promise<number> {
  return useGamesStore.getState().verifyInstallStatus();
}

/** Runs the install verification on startup and whenever the window regains focus. */
export function useInstallVerify() {
  useEffect(() => {
    let lastRun = 0;
    let inFlight = false;

    const run = async (initial = false) => {
      if (inFlight) return;
      const now = Date.now();
      if (!initial && now - lastRun < FOCUS_MIN_INTERVAL) return;
      lastRun = now;
      inFlight = true;
      try {
        const store = useGamesStore.getState();
        if (initial) await store.load();
        await verifyInstallations();
      } catch {
        // Verification is best-effort — a failed scan must not surface as an error.
      } finally {
        inFlight = false;
      }
    };

    const onFocus = () => void run();
    const onVisibility = () => {
      if (!document.hidden) void run();
    };

    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", onVisibility);
    const timer = window.setTimeout(() => void run(true), STARTUP_DELAY);

    return () => {
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("visibilitychange", onVisibility);
      window.clearTimeout(timer);
    };
  }, []);
}
