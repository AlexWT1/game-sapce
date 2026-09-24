import { create } from "zustand";
import { api } from "@/services/api";
import type { Achievement } from "@/types";

interface AchievementsState {
  items: Achievement[];
  loading: boolean;
  loaded: boolean;
  load: (gameId?: number | null, force?: boolean) => Promise<void>;
  unlock: (id: number) => Promise<void>;
  stats: () => { unlocked: number; total: number; percent: number };
}

export const useAchievementsStore = create<AchievementsState>((set, get) => ({
  items: [],
  loading: false,
  loaded: false,

  load: async (gameId = null, force = false) => {
    if (get().loading) return;
    if (get().loaded && !force && gameId == null) return;
    set({ loading: true });
    try {
      const items = await api.getAchievements(gameId);
      set({ items, loading: false, loaded: true });
    } catch {
      set({ loading: false, loaded: true });
    }
  },

  unlock: async (id) => {
    const ach = await api.unlockAchievement(id);
    set({
      items: get().items.map((a) => (a.id === ach.id ? ach : a)),
    });
  },

  stats: () => {
    const items = get().items;
    const total = items.length;
    const unlocked = items.filter((a) => a.unlocked).length;
    return {
      unlocked,
      total,
      percent: total === 0 ? 0 : Math.round((unlocked / total) * 100),
    };
  },
}));
