import { useEffect, useMemo } from "react";
import { Outlet, useNavigate } from "react-router-dom";
import { Sidebar } from "@/components/layout/Sidebar";
import { Topbar } from "@/components/layout/Topbar";
import { CommandPalette } from "@/components/command/CommandPalette";
import { ToastHost } from "@/components/ui/Toast";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import { Onboarding } from "@/components/onboarding/Onboarding";
import { RecapAutoShow } from "@/features/recap/RecapAutoShow";
import { GameFormModal } from "@/features/games/GameFormModal";
import { MissingExecutableDialog } from "@/features/games/MissingExecutableDialog";
import { useHotkeys } from "@/hooks/useHotkeys";
import { useAppEvents } from "@/hooks/useAppEvents";
import { useInstallVerify } from "@/hooks/useInstallVerify";
import { useT } from "@/hooks/useT";
import { useUiStore } from "@/stores/uiStore";
import { useSettingsStore, applyAccent, applyVisualPreferences } from "@/stores/settingsStore";
import { api } from "@/services/api";
import { useSessionStore } from "@/stores/sessionStore";
import { useGamesStore } from "@/stores/gamesStore";
import { usePlayerStore } from "@/stores/playerStore";
import { PlayerNameModal } from "@/features/profile/PlayerNameModal";
import { useStatisticsStore } from "@/stores/statisticsStore";
import { useAchievementsStore } from "@/stores/achievementsStore";
import { cx } from "@/utils/format";

export function AppLayout() {
  const t = useT();
  const navigate = useNavigate();
  const setPalette = useUiStore((s) => s.setCommandPalette);
  const paletteOpen = useUiStore((s) => s.commandPaletteOpen);
  const dragOver = useUiStore((s) => s.dragOver);
  const setDragOver = useUiStore((s) => s.setDragOver);
  const onboardingDone = useSettingsStore((s) => s.onboardingDone);
  const accent = useSettingsStore((s) => s.accent);
  const theme = useSettingsStore((s) => s.theme);
  const animations = useSettingsStore((s) => s.animations);
  const transparency = useSettingsStore((s) => s.transparency);

  useAppEvents();
  useInstallVerify();

  const hotkeys = useMemo(
    () => [
      {
        combo: "ctrl+k",
        handler: () => setPalette(!paletteOpen),
      },
      {
        combo: "meta+k",
        handler: () => setPalette(!paletteOpen),
      },
      {
        combo: "escape",
        handler: () => setPalette(false),
      },
    ],
    [paletteOpen, setPalette],
  );
  useHotkeys(hotkeys);

  // Load initial data.
  useEffect(() => {
    void useGamesStore.getState().load();
    void useSessionStore.getState().refresh();
    void useStatisticsStore.getState().load();
    void useAchievementsStore.getState().load();
    void usePlayerStore.getState().load();
  }, []);

  // Live playtime ticks (only when games are running).
  const runningCount = useSessionStore((s) => s.running.length);
  useEffect(() => {
    if (runningCount === 0) return;
    const id = window.setInterval(() => useSessionStore.getState().tick(), 1000);
    return () => window.clearInterval(id);
  }, [runningCount]);

  // Global theme / accent / visual preferences.
  useEffect(() => {
    applyAccent(accent);
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.setProperty(
      "color-scheme",
      theme === "black" ? "dark" : "dark",
    );
    if (theme === "black") {
      document.documentElement.style.setProperty("--color-bg", "#000000");
      document.documentElement.style.setProperty("--color-bg-2", "#050505");
    } else {
      document.documentElement.style.setProperty("--color-bg", "#070912");
      document.documentElement.style.setProperty("--color-bg-2", "#0a0d1a");
    }
  }, [accent, theme]);

  useEffect(() => {
    applyVisualPreferences(animations, transparency);
  }, [animations, transparency]);

  // Sync demoMode from backend (source of truth for seed/visibility).
  useEffect(() => {
    let alive = true;
    void api
      .getSettings()
      .then(async (s) => {
        if (!alive) return;
        const mode = s.demo_mode;
        if (mode === "on" || mode === "off") {
          const next = mode === "on";
          const prev = useSettingsStore.getState().demoMode;
          if (next !== prev) {
            useSettingsStore.getState().set({ demoMode: next });
            await useGamesStore.getState().load(true);
            await useStatisticsStore.getState().load(true);
            await useAchievementsStore.getState().load(null, true);
          }
        }
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, []);

  // Drag & drop visual state.
  useEffect(() => {
    const onDragOver = (e: DragEvent) => {
      if (e.dataTransfer?.types.includes("Files")) {
        e.preventDefault();
        setDragOver(true);
      }
    };
    const onDragLeave = (e: DragEvent) => {
      if (e.relatedTarget === null) setDragOver(false);
    };
    const onDrop = (e: DragEvent) => {
      if (e.dataTransfer?.types.includes("Files")) {
        e.preventDefault();
        setDragOver(false);
      }
    };
    window.addEventListener("dragover", onDragOver);
    window.addEventListener("dragleave", onDragLeave);
    window.addEventListener("drop", onDrop);
    return () => {
      window.removeEventListener("dragover", onDragOver);
      window.removeEventListener("dragleave", onDragLeave);
      window.removeEventListener("drop", onDrop);
    };
  }, [setDragOver]);

  // Tray "navigate" events.
  useEffect(() => {
    void import("@tauri-apps/api/event").then(({ listen }) =>
      listen<{ path: string }>("navigate", (e) => navigate(e.payload.path)),
    );
  }, [navigate]);

  if (!onboardingDone) {
    return (
      <>
        <Onboarding />
        <GameFormModal />
        <ToastHost />
      </>
    );
  }

  return (
    <div className="relative z-10 h-full flex flex-col overflow-hidden">
      <div className="flex flex-1 min-h-0">
        <Sidebar />
        <div className="flex-1 min-w-0 flex flex-col">
          <Topbar />
          <main className="flex-1 min-h-0 overflow-y-auto">
            <Outlet />
          </main>
        </div>
      </div>

      <CommandPalette />
      <GameFormModal />
      <MissingExecutableDialog />
      <ConfirmDialog />
      <ToastHost />
      <PlayerNameModal />
      <RecapAutoShow />

      {dragOver && (
        <div className="drag-overlay" onDragLeave={() => setDragOver(false)}>
          <div className="text-center space-y-3">
            <div className="mx-auto h-16 w-16 rounded-2xl border-2 border-dashed border-[var(--color-accent)]/60 grid place-items-center">
              <span className="text-[var(--color-accent)] text-2xl">+</span>
            </div>
            <p className="text-lg font-semibold text-ink">{t("dialog.dropTitle")}</p>
          </div>
        </div>
      )}
    </div>
  );
}

export const layoutClass = cx("min-h-full");
