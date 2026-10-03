import { create } from "zustand";
import { api } from "@/services/api";
import type { Achievement } from "@/types";

interface AchievementsState {
  items: Achievement[];
  byGame: Record<number, Achievement[]>;
  loading: boolean;
  loaded: boolean;
  load: (gameId?: number | null, force?: boolean) => Promise<void>;
  unlock: (id: number) => Promise<void>;
  stats: () => { unlocked: number; total: number; percent: number };
}

const inflight = new Map<string, Promise<void>>();

export const useAchievementsStore = create<AchievementsState>((set, get) => ({
  items: [],
  byGame: {},
  loading: false,
  loaded: false,

  load: async (gameId = null, force = false) => {
    const key = gameId == null ? "global" : `game:${gameId}`;
    if (!force) {
      if (gameId == null && get().loaded) return;
      if (gameId != null && get().byGame[gameId]) return;
    }
    const existing = inflight.get(key);
    if (existing) return existing;

    if (gameId == null) set({ loading: true });
    const promise = (async () => {
      try {
        const items = await api.getAchievements(gameId);
        if (gameId == null) {
          const byGame: Record<number, Achievement[]> = {};
          for (const a of items) {
            if (a.gameId != null) (byGame[a.gameId] ??= []).push(a);
          }
          set({ items, byGame, loading: false, loaded: true });
        } else {
          set((s) => ({ byGame: { ...s.byGame, [gameId]: items } }));
        }
      } catch (e) {
        console.error("failed to load achievements", e);
        if (gameId == null) set({ loading: false });
      } finally {
        inflight.delete(key);
      }
    })();
    inflight.set(key, promise);
    return promise;
  },

  unlock: async (id) => {
    const ach = await api.unlockAchievement(id);
    set((s) => ({
      items: s.items.map((a) => (a.id === ach.id ? ach : a)),
      byGame: Object.fromEntries(
        Object.entries(s.byGame).map(([gid, list]) => [
          gid,
          list.map((a) => (a.id === ach.id ? ach : a)),
        ]),
      ),
    }));
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
