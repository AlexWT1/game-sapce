import { create } from "zustand";
import { api } from "@/services/api";
import type { GameStatistics, Statistics } from "@/types";

interface StatisticsState {
  stats: Statistics | null;
  /** Which window the cached `stats` belongs to: "lifetime" or "range:<from>:<to>". */
  range: string;
  gameStats: Record<number, GameStatistics>;
  loading: boolean;
  loaded: boolean;
  load: (force?: boolean) => Promise<void>;
  loadRange: (from: string, to: string, force?: boolean) => Promise<void>;
  loadGame: (gameId: number, force?: boolean) => Promise<void>;
}

const rangeKey = (from: string, to: string) => `range:${from}:${to}`;

/** Canonical key of the cached statistics window. Shared with the Recap hook. */
export const statsRangeKey = rangeKey;

const inflight = new Map<string, Promise<void>>();

export const useStatisticsStore = create<StatisticsState>((set, get) => ({
  stats: null,
  range: "lifetime",
  gameStats: {},
  loading: false,
  loaded: false,

  load: async (force = false) => {
    if (get().loading) return;
    if (get().range === "lifetime" && get().loaded && !force) return;
    const existing = inflight.get("lifetime");
    if (existing) return existing;
    set({ loading: true });
    const promise = (async () => {
      try {
        const stats = await api.getStatistics();
        set({ stats, range: "lifetime", loading: false, loaded: true });
      } catch (err) {
        console.error("statistics lifetime load failed", err);
        set({ loading: false, loaded: true });
      } finally {
        inflight.delete("lifetime");
      }
    })();
    inflight.set("lifetime", promise);
    return promise;
  },

  loadRange: async (from, to, force = false) => {
    const key = rangeKey(from, to);
    if (get().range === key && get().loaded && !force) return;
    const existing = inflight.get(key);
    if (existing) return existing;
    set({ loading: true });
    const promise = (async () => {
      try {
        const stats = await api.getStatistics(from, to);
        set({ stats, range: key, loading: false, loaded: true });
      } catch (err) {
        console.error(`statistics range load failed (${from}…${to})`, err);
        set({ loading: false });
      } finally {
        inflight.delete(key);
      }
    })();
    inflight.set(key, promise);
    return promise;
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
