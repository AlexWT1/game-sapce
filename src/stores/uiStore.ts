import { create } from "zustand";
import type { Game } from "@/types";

export type ToastKind = "info" | "success" | "error" | "achievement";

export interface Toast {
  id: number;
  kind: ToastKind;
  title: string;
  body?: string;
}

export interface ConfirmRequest {
  title: string;
  body?: string;
  confirmLabel?: string;
  danger?: boolean;
  onConfirm: () => void | Promise<void>;
}

interface UiState {
  sidebarCollapsed: boolean;
  commandPaletteOpen: boolean;
  toasts: Toast[];
  confirm: ConfirmRequest | null;
  addGameOpen: boolean;
  addGamePrefill: Partial<Game> & { executablePath?: string | null; title?: string };
  editingGame: Game | null;
  missingExeGame: Game | null;
  dragOver: boolean;
  notificationsOpen: boolean;
  recentNotifications: Array<{ id: number; title: string; body?: string; at: string }>;

  toggleSidebar: () => void;
  setCommandPalette: (open: boolean) => void;
  pushToast: (t: Omit<Toast, "id">) => void;
  dismissToast: (id: number) => void;
  askConfirm: (req: ConfirmRequest | null) => void;
  openAddGame: (prefill?: Partial<Game> & { executablePath?: string | null }) => void;
  closeAddGame: () => void;
  openEditGame: (game: Game | null) => void;
  openMissingExe: (game: Game) => void;
  closeMissingExe: () => void;
  setDragOver: (v: boolean) => void;
  setNotificationsOpen: (v: boolean) => void;
  pushNotification: (title: string, body?: string) => void;
}

let toastSeq = 1;

export const useUiStore = create<UiState>((set, get) => ({
  sidebarCollapsed: false,
  commandPaletteOpen: false,
  toasts: [],
  confirm: null,
  addGameOpen: false,
  addGamePrefill: {},
  editingGame: null,
  missingExeGame: null,
  dragOver: false,
  notificationsOpen: false,
  recentNotifications: [],

  toggleSidebar: () => set({ sidebarCollapsed: !get().sidebarCollapsed }),
  setCommandPalette: (open) => set({ commandPaletteOpen: open }),
  pushToast: (t) => {
    const id = toastSeq++;
    set({ toasts: [...get().toasts, { ...t, id }] });
    window.setTimeout(() => get().dismissToast(id), t.kind === "error" ? 6000 : 4000);
  },
  dismissToast: (id) => set({ toasts: get().toasts.filter((t) => t.id !== id) }),
  askConfirm: (req) => set({ confirm: req }),
  openAddGame: (prefill = {}) =>
    set({ addGameOpen: true, addGamePrefill: prefill, editingGame: null }),
  closeAddGame: () => set({ addGameOpen: false, addGamePrefill: {} }),
  openEditGame: (game) => set({ editingGame: game, addGameOpen: false }),
  openMissingExe: (game) => set({ missingExeGame: game }),
  closeMissingExe: () => set({ missingExeGame: null }),
  setDragOver: (v) => set({ dragOver: v }),
  setNotificationsOpen: (v) => set({ notificationsOpen: v }),
  pushNotification: (title, body) => {
    const item = {
      id: Date.now(),
      title,
      body,
      at: new Date().toISOString(),
    };
    set({ recentNotifications: [item, ...get().recentNotifications].slice(0, 20) });
  },
}));
