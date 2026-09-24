import { useEffect } from "react";
import { events } from "@/services/events";
import { useGamesStore } from "@/stores/gamesStore";
import { useSessionStore } from "@/stores/sessionStore";
import { useStatisticsStore } from "@/stores/statisticsStore";
import { useAchievementsStore } from "@/stores/achievementsStore";
import { useUiStore } from "@/stores/uiStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { dictionaries } from "@/i18n";

/** Subscribes to backend events and refreshes stores accordingly. */
export function useAppEvents() {
  const locale = useSettingsStore((s) => s.locale);

  useEffect(() => {
    const t = dictionaries[locale];
    const unlisteners: Array<Promise<() => void>> = [];

    unlisteners.push(
      events.onGameStarted(() => {
        void useSessionStore.getState().refresh();
        void useGamesStore.getState().load(true);
        void useStatisticsStore.getState().load(true);
        if (useSettingsStore.getState().notifyGameStart) {
          useUiStore.getState().pushNotification(t["home.statusPlaying"], undefined);
        }
      }),
    );

    unlisteners.push(
      events.onGameStopped(() => {
        void useSessionStore.getState().refresh();
        void useGamesStore.getState().load(true);
        void useStatisticsStore.getState().load(true);
        void useAchievementsStore.getState().load(null, true);
        if (useSettingsStore.getState().notifySessionEnd) {
          useUiStore
            .getState()
            .pushNotification("Сессия завершена", "Статистика обновлена");
        }
      }),
    );

    unlisteners.push(
      events.onAchievementUnlocked((ach) => {
        void useAchievementsStore.getState().load(null, true);
        useUiStore.getState().pushNotification(t["toast.achievement"], ach.title);
        if (useSettingsStore.getState().notifyAchievement) {
          useUiStore.getState().pushToast({
            kind: "achievement",
            title: t["toast.achievement"],
            body: ach.title,
          });
        }
      }),
    );

    unlisteners.push(
      events.onStatisticsUpdated(() => {
        void useStatisticsStore.getState().load(true);
        void useGamesStore.getState().load(true);
      }),
    );

    unlisteners.push(
      events.onGameDeleted(() => {
        void useGamesStore.getState().load(true);
        void useStatisticsStore.getState().load(true);
      }),
    );

    unlisteners.push(
      events.onFileDropped((payload) => {
        const path = payload.paths[0];
        if (!path) return;
        const name =
          path.split(/[\\/]/).pop() ?? path;
        const title = name.replace(/\.[^.]+$/, "").replace(/[-_]/g, " ");
        useUiStore.getState().openAddGame({
          title,
          executablePath: path,
        });
      }),
    );

    let cancelled = false;
    void Promise.all(unlisteners).then((fns) => {
      if (cancelled) fns.forEach((f) => f());
    });

    return () => {
      cancelled = true;
      void Promise.all(unlisteners).then((fns) => fns.forEach((f) => f()));
    };
  }, [locale]);
}
