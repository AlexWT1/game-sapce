import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Locale } from "@/i18n";
import { dictionaries } from "@/i18n";

export type ThemeMode = "dark" | "black";
export type AccentColor = "violet" | "blue" | "cyan" | "amber" | "rose";

interface SettingsState {
  locale: ThemeMode extends never ? never : Locale;
  theme: ThemeMode;
  accent: AccentColor;
  animations: boolean;
  transparency: boolean;
  demoMode: boolean;
  launchOnStartup: boolean;
  minimizeToTray: boolean;
  autoDetect: boolean;
  monitoringInterval: number;
  defaultGamesDir: string;
  notifyAchievement: boolean;
  notifyGameStart: boolean;
  notifyFriendOnline: boolean;
  notifySessionEnd: boolean;
  onboardingDone: boolean;
  hydrated: boolean;
  set: (partial: Partial<Omit<SettingsState, "set" | "translate">>) => void;
  translate: (key: keyof (typeof dictionaries)["ru"]) => string;
}

const ACCENTS: Record<AccentColor, string> = {
  violet: "#7c5cff",
  blue: "#5b8cff",
  cyan: "#22d3ee",
  amber: "#f59e0b",
  rose: "#fb7185",
};

export const accentHex = (a: AccentColor) => ACCENTS[a] ?? ACCENTS.violet;

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set, get) => ({
      locale: "ru",
      theme: "dark",
      accent: "violet",
      animations: true,
      transparency: true,
      demoMode: true,
      launchOnStartup: false,
      minimizeToTray: true,
      autoDetect: true,
      monitoringInterval: 2,
      defaultGamesDir: "",
      notifyAchievement: true,
      notifyGameStart: true,
      notifyFriendOnline: true,
      notifySessionEnd: true,
      onboardingDone: false,
      hydrated: false,
      set: (partial) => set(partial),
      translate: (key) => {
        const { locale } = get();
        const dict = dictionaries[locale] ?? dictionaries.ru;
        return dict[key] ?? dictionaries.ru[key] ?? String(key);
      },
    }),
    {
      name: "gs-settings",
      partialize: (s) => ({
        locale: s.locale,
        theme: s.theme,
        accent: s.accent,
        animations: s.animations,
        transparency: s.transparency,
        demoMode: s.demoMode,
        launchOnStartup: s.launchOnStartup,
        minimizeToTray: s.minimizeToTray,
        autoDetect: s.autoDetect,
        monitoringInterval: s.monitoringInterval,
        defaultGamesDir: s.defaultGamesDir,
        notifyAchievement: s.notifyAchievement,
        notifyGameStart: s.notifyGameStart,
        notifyFriendOnline: s.notifyFriendOnline,
        notifySessionEnd: s.notifySessionEnd,
        onboardingDone: s.onboardingDone,
      }),
      onRehydrateStorage: () => (state) => {
        if (state) {
          state.set({ hydrated: true });
          applyVisualPreferences(state.animations, state.transparency);
        }
      },
    },
  ),
);

export function applyAccent(accent: AccentColor) {
  document.documentElement.style.setProperty("--color-accent", accentHex(accent));
  document.documentElement.style.setProperty("--accent-soft", `${accentHex(accent)}29`);
}

export function applyVisualPreferences(animations: boolean, transparency: boolean) {
  const root = document.documentElement;
  root.dataset.animations = animations ? "on" : "off";
  root.dataset.transparency = transparency ? "on" : "off";
  root.style.setProperty(
    "--transition-fast",
    animations ? "160ms cubic-bezier(0.2,0.7,0.3,1)" : "0ms",
  );
  root.style.setProperty(
    "--transition-med",
    animations ? "280ms cubic-bezier(0.2,0.7,0.3,1)" : "0ms",
  );
}
