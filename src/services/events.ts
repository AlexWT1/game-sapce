import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Achievement, RunningGame } from "@/types";

export interface GameStartedPayload {
  gameId: number;
  pid?: number;
  title?: string;
  recovered?: boolean;
}

export interface GameStoppedPayload {
  gameId: number;
  durationSeconds?: number;
}

export interface FileDroppedPayload {
  paths: string[];
}

export interface NavigatePayload {
  path: string;
}

type Handler<T> = (payload: T) => void;

async function on<T>(event: string, handler: Handler<T>): Promise<UnlistenFn> {
  return listen<T>(event, (e) => handler(e.payload));
}

export const events = {
  onGameStarted: (h: Handler<GameStartedPayload>) => on("game-started", h),
  onGameStopped: (h: Handler<GameStoppedPayload>) => on("game-stopped", h),
  onAchievementUnlocked: (h: Handler<Achievement>) =>
    on("achievement-unlocked", h),
  onStatisticsUpdated: (h: Handler<Record<string, never>>) =>
    on("statistics-updated", h),
  onFileDropped: (h: Handler<FileDroppedPayload>) => on("file-dropped", h),
  onNavigate: (h: Handler<NavigatePayload>) => on("navigate", h),
  onGameDeleted: (h: Handler<{ gameId: number }>) => on("game-deleted", h),
};

export type RunningChanged = RunningGame[];
