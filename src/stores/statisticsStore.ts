import { create } from "zustand";
import { api } from "@/services/api";
import type { GameStatistics, Statistics } from "@/types";

interface StatisticsState {
  stats: Statistics | null;
  gameStats: Record<number, GameStatistics>;
  loading: boolean;
  loaded: boolean;
  load: (force?: boolean) => Promise<void>;
  loadGame: (gameId: number, force?: boolean) => Promise<void>;
}

export const useStatisticsStore = create<StatisticsState>((set, get) => ({
  stats: null,
  gameStats: {},
  loading: false,
  loaded: false,

  load: async (force = false) => {
    if (get().loading) return;
    if (get().loaded && !force) return;
    set({ loading: true });
    try {
      const stats = await api.getStatistics();
      set({ stats, loading: false, loaded: true });
    } catch {
      set({ loading: false, loaded: true });
    }
  },

  loadGame: async (gameId, force = false) => {
    if (!force && get().gameStats[gameId]) return;
    try {
      const gameStats = await api.getGameStatistics(gameId);
      set({ gameStats: { ...get().gameStats, [gameId]: gameStats } });
    } catch {
      /* keep previous */
    }
  },
}));
