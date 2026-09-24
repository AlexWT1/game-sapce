import { create } from "zustand";
import { api } from "@/services/api";
import type { RunningGame } from "@/types";

interface SessionState {
  running: RunningGame[];
  /** Local live counters: gameId → seconds since session start */
  liveSeconds: Record<number, number>;
  loaded: boolean;
  refresh: () => Promise<void>;
  launch: (gameId: number) => Promise<RunningGame>;
  stop: (gameId: number) => Promise<void>;
  isRunning: (gameId: number) => boolean;
  startedAt: (gameId: number) => number | null;
  tick: () => void;
}

export const useSessionStore = create<SessionState>((set, get) => ({
  running: [],
  liveSeconds: {},
  loaded: false,

  refresh: async () => {
    try {
      const running = await api.getRunningGames();
      const liveSeconds: Record<number, number> = {};
      const now = Date.now();
      for (const r of running) {
        const start = new Date(r.startedAt).getTime();
        liveSeconds[r.gameId] =
          Number.isFinite(start) ? Math.max(0, Math.floor((now - start) / 1000)) : 0;
      }
      set({ running, liveSeconds, loaded: true });
    } catch {
      set({ loaded: true });
    }
  },

  launch: async (gameId) => {
    const r = await api.launchGame(gameId);
    set({
      running: [...get().running.filter((x) => x.gameId !== gameId), r],
      liveSeconds: { ...get().liveSeconds, [gameId]: 0 },
    });
    return r;
  },

  stop: async (gameId) => {
    await api.stopGame(gameId);
    set({
      running: get().running.filter((x) => x.gameId !== gameId),
    });
  },

  isRunning: (gameId) => get().running.some((r) => r.gameId === gameId),

  startedAt: (gameId) => {
    const r = get().running.find((x) => x.gameId === gameId);
    return r ? new Date(r.startedAt).getTime() : null;
  },

  tick: () => {
    const { running, liveSeconds } = get();
    if (running.length === 0) return;
    const next: Record<number, number> = { ...liveSeconds };
    for (const r of running) {
      const start = new Date(r.startedAt).getTime();
      next[r.gameId] = Number.isFinite(start)
        ? Math.max(0, Math.floor((Date.now() - start) / 1000))
        : (next[r.gameId] ?? 0) + 1;
    }
    set({ liveSeconds: next });
  },
}));
