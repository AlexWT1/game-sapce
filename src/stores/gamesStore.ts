import { create } from "zustand";
import { api } from "@/services/api";
import type { Game, GameInput } from "@/types";

interface GamesState {
  games: Game[];
  loading: boolean;
  error: string | null;
  loaded: boolean;
  load: (force?: boolean) => Promise<void>;
  create: (input: GameInput) => Promise<Game>;
  update: (id: number, input: GameInput) => Promise<Game>;
  remove: (id: number) => Promise<void>;
  toggleFavorite: (id: number) => Promise<void>;
  setExecutable: (id: number, path: string) => Promise<Game>;
  upsertLocal: (game: Game) => void;
}

export const useGamesStore = create<GamesState>((set, get) => ({
  games: [],
  loading: false,
  error: null,
  loaded: false,

  load: async (force = false) => {
    if (get().loading) return;
    if (get().loaded && !force) return;
    set({ loading: true, error: null });
    try {
      const games = await api.getGames(true);
      set({ games, loading: false, loaded: true });
    } catch (e) {
      set({ loading: false, error: String(e), loaded: true });
    }
  },

  create: async (input) => {
    const game = await api.createGame(input);
    set({ games: [game, ...get().games] });
    return game;
  },

  update: async (id, input) => {
    const game = await api.updateGame(id, input);
    set({ games: get().games.map((g) => (g.id === id ? game : g)) });
    return game;
  },

  remove: async (id) => {
    await api.deleteGame(id);
    set({ games: get().games.filter((g) => g.id !== id) });
  },

  toggleFavorite: async (id) => {
    const game = await api.toggleFavorite(id);
    set({ games: get().games.map((g) => (g.id === id ? game : g)) });
  },

  setExecutable: async (id, path) => {
    const game = await api.setGameExecutable(id, path);
    set({ games: get().games.map((g) => (g.id === id ? game : g)) });
    return game;
  },

  upsertLocal: (game) => {
    const games = get().games;
    const idx = games.findIndex((g) => g.id === game.id);
    if (idx === -1) set({ games: [game, ...games] });
    else {
      const next = games.slice();
      next[idx] = game;
      set({ games: next });
    }
  },
}));
