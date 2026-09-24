import { invoke } from "@tauri-apps/api/core";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import type {
  AppInfo,
  Achievement,
  DiscoveredGame,
  Game,
  GameInput,
  GameSession,
  GameStatistics,
  GameTheme,
  RunningGame,
  SearchResults,
  SettingsMap,
  Statistics,
} from "@/types";

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err) {
    const message = typeof err === "string" ? err : String(err);
    throw new Error(message);
  }
}

export const api = {
  // games
  getGames: (includeHidden = false) =>
    call<Game[]>("get_games", { includeHidden }),
  getGame: (id: number) => call<Game>("get_game", { id }),
  createGame: (input: GameInput) => call<Game>("create_game", { input }),
  updateGame: (id: number, input: GameInput) =>
    call<Game>("update_game", { id, input }),
  deleteGame: (id: number) => call<void>("delete_game", { id }),
  toggleFavorite: (id: number) => call<Game>("toggle_favorite", { id }),
  setGameExecutable: (id: number, path: string) =>
    call<Game>("set_game_executable", { id, path }),
  gameExists: (title: string) => call<boolean>("game_exists", { title }),

  // themes
  getGameTheme: (gameId: number) => call<GameTheme>("get_game_theme", { gameId }),
  setGameTheme: (theme: GameTheme) => call<GameTheme>("set_game_theme", { themeInput: theme }),

  // launch / sessions
  launchGame: (gameId: number) => call<RunningGame>("launch_game", { gameId }),
  stopGame: (gameId: number) => call<void>("stop_game", { gameId }),
  getRunningGames: () => call<RunningGame[]>("get_running_games"),
  isGameRunning: (gameId: number) => call<boolean>("is_game_running", { gameId }),
  startGameSession: (gameId: number) =>
    call<GameSession>("start_game_session", { gameId }),
  finishGameSession: (sessionId: number) =>
    call<GameSession>("finish_game_session", { sessionId }),
  getGameSessions: (gameId: number) =>
    call<GameSession[]>("get_game_sessions", { gameId }),

  // statistics
  getStatistics: () => call<Statistics>("get_statistics"),
  getGameStatistics: (gameId: number) =>
    call<GameStatistics>("get_game_statistics", { gameId }),

  // achievements
  getAchievements: (gameId?: number | null) =>
    call<Achievement[]>("get_achievements", { gameId: gameId ?? null }),
  unlockAchievement: (id: number) => call<Achievement>("unlock_achievement", { id }),

  // settings
  getSettings: () => call<SettingsMap>("get_settings"),
  updateSettings: (settings: SettingsMap) =>
    call<SettingsMap>("update_settings", { settings }),
  getSetting: (key: string) => call<string | null>("get_setting", { key }),

  // system (dialogs via JS plugin — reliable on Linux)
  windowMinimize: () => call<void>("window_minimize"),
  windowToggleMaximize: () => call<void>("window_toggle_maximize"),
  windowClose: () => call<void>("window_close"),
  getAppInfo: () => call<AppInfo>("get_app_info"),
  selectGameExecutable: async () => {
    const path = await openDialog({ multiple: false, title: "Выберите исполняемый файл" });
    return typeof path === "string" ? path : null;
  },
  selectGameDirectory: async () => {
    const path = await openDialog({ directory: true, title: "Выберите папку с играми" });
    return typeof path === "string" ? path : null;
  },
  selectAnyFile: async () => {
    const path = await openDialog({ multiple: false });
    return typeof path === "string" ? path : null;
  },
  discoverGames: (dirs: string[] = []) =>
    call<DiscoveredGame[]>("discover_games", { dirs }),
  exportData: async (path?: string | null) => {
    let target = path ?? null;
    if (!target) {
      const picked = await saveDialog({
        title: "Экспорт данных",
        defaultPath: "game-space-export.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!picked) throw new Error("Операция отменена");
      target = picked;
    }
    return call<string>("export_data", { path: target });
  },
  importData: async (path?: string | null) => {
    let source = path ?? null;
    if (!source) {
      const picked = await openDialog({
        title: "Импорт данных",
        multiple: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!picked) throw new Error("Операция отменена");
      source = picked;
    }
    return call<[number, number]>("import_data", { path: source });
  },
  resetData: () => call<void>("reset_data"),
  searchAll: (query: string) => call<SearchResults>("search_all", { query }),
  openPath: (path: string) => call<void>("open_path", { path }),
  getDefaultGameDirs: () => call<string[]>("get_default_game_dirs"),
};

export type Api = typeof api;
