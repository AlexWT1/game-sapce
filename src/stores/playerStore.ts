import { create } from "zustand";
import { api } from "@/services/api";

/**
 * Nickname shown until the user saves their own in the database
 * (текущей базе без сохранённого ника соответствует именно он).
 */
export const DEFAULT_PLAYER_NAME = "Lusfan";

/** Hard input cap, mirrored by the backend (`set_player_name`). */
export const PLAYER_NAME_MAX_LEN = 32;

type PlayerNameError = "empty" | "tooLong" | "saveFailed";

interface PlayerState {
  /** `null` — nothing saved in the DB yet; pending load keeps `loaded` false. */
  playerName: string | null;
  loaded: boolean;
  setPlayerError: PlayerNameError | null;
  /** Shared input draft so onboarding and prompts keep the typed value. */
  nameDraft: string;
  setNameDraft: (name: string) => void;
  /** Reads the nickname from the backend (`settings.player_name`). */
  load: () => Promise<void>;
  /** Saves the nickname in the DB and reflects it in the UI immediately. */
  setPlayerName: (name: string) => Promise<boolean>;
  clearError: () => void;
}

export const usePlayerStore = create<PlayerState>((set) => ({
  playerName: null,
  loaded: false,
  setPlayerError: null,
  nameDraft: "",
  setNameDraft: (name) => set({ nameDraft: name }),

  load: async () => {
    try {
      const name = await api.getPlayerName();
      set({ playerName: name, loaded: true });
    } catch (err) {
      console.error("load player name failed", err);
      set({ loaded: true });
    }
  },

  setPlayerName: async (name) => {
    const value = name.trim();
    if (!value) {
      set({ setPlayerError: "empty" });
      return false;
    }
    if (value.length > PLAYER_NAME_MAX_LEN) {
      set({ setPlayerError: "tooLong" });
      return false;
    }
    try {
      const saved = await api.setPlayerName(value);
      set({ playerName: saved, setPlayerError: null });
      return true;
    } catch (err) {
      console.error("save player name failed", err);
      set({ setPlayerError: "saveFailed" });
      return false;
    }
  },

  clearError: () => set({ setPlayerError: null }),
}));

/** Displayed nickname: DB value or the default one for legacy databases. */
export const usePlayerDisplayName = () =>
  usePlayerStore((s) => (s.loaded ? s.playerName : null) ?? DEFAULT_PLAYER_NAME);
