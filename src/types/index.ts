export type GameSource =
  | "Manual"
  | "Steam"
  | "Epic"
  | "Gog"
  | "Ubisoft"
  | "Ea"
  | "BattleNet"
  | "Xbox"
  | "Demo";

export interface Game {
  id: number;
  title: string;
  description: string;
  executablePath: string | null;
  installPath: string | null;
  coverPath: string | null;
  backgroundPath: string | null;
  iconPath: string | null;
  developer: string;
  publisher: string;
  genre: string;
  releaseYear: number | null;
  platform: string;
  source: string;
  controllerSupport: string;
  isInstalled: boolean;
  isFavorite: boolean;
  isHidden: boolean;
  createdAt: string;
  updatedAt: string;
  lastPlayedAt: string | null;
  totalPlaytimeSeconds: number;
  launchCount: number;
}

export interface GameInput {
  title?: string;
  description?: string | null;
  executablePath?: string | null;
  installPath?: string | null;
  coverPath?: string | null;
  backgroundPath?: string | null;
  iconPath?: string | null;
  developer?: string | null;
  publisher?: string | null;
  genre?: string | null;
  releaseYear?: number | null;
  platform?: string | null;
  source?: string | null;
  controllerSupport?: string | null;
  isInstalled?: boolean | null;
  isFavorite?: boolean | null;
  isHidden?: boolean | null;
}

export interface GameSession {
  id: number;
  gameId: number;
  startedAt: string;
  endedAt: string | null;
  durationSeconds: number;
  pid: number | null;
}

export type Rarity = "Common" | "Rare" | "Epic" | "Legendary";

export interface Achievement {
  id: number;
  gameId: number | null;
  gameTitle: string | null;
  title: string;
  description: string;
  iconPath: string | null;
  rarity: string;
  unlocked: boolean;
  unlockedAt: string | null;
  progressCurrent: number;
  progressTarget: number;
  scope: "game" | "app" | string;
}

export interface GameTheme {
  gameId: number;
  primaryColor: string;
  secondaryColor: string;
  accentColor: string;
  backgroundColor: string;
  glowColor: string;
  fontFamily: string;
}

export interface GenreCount {
  genre: string;
  count: number;
}

export interface PlaytimeByDay {
  date: string;
  seconds: number;
}

export interface RankedPlaytime {
  gameId: number;
  title: string;
  seconds: number;
  coverPath: string | null;
}

export interface Statistics {
  totalPlaytimeSeconds: number;
  gamesCount: number;
  installedCount: number;
  sessionsCount: number;
  achievementsUnlocked: number;
  achievementsTotal: number;
  favoriteCount: number;
  mostPlayed: RankedPlaytime[];
  byGenre: GenreCount[];
  daily: PlaytimeByDay[];
  weekly: PlaytimeByDay[];
  monthly: PlaytimeByDay[];
  avgSessionSeconds: number;
  longestSessionSeconds: number;
}

export interface GameStatistics {
  totalPlaytimeSeconds: number;
  launchCount: number;
  sessionsCount: number;
  avgSessionSeconds: number;
  longestSessionSeconds: number;
  lastSessionAt: string | null;
  playtimeWeekSeconds: number;
  playtimeMonthSeconds: number;
  playtimeYearSeconds: number;
  daily: PlaytimeByDay[];
}

export interface SearchResults {
  games: Game[];
  achievements: Achievement[];
  genres: string[];
  developers: string[];
}

export interface RunningGame {
  gameId: number;
  title: string;
  pid: number;
  startedAt: string;
  coverPath: string | null;
}

export interface AppInfo {
  version: string;
  databasePath: string;
  platform: string;
}

export type SettingsMap = Record<string, unknown>;

export type GameFilter =
  | "all"
  | "installed"
  | "not-installed"
  | "favorite"
  | "recent"
  | "controller"
  | "hidden";

export type LibraryView = "grid" | "list";

export type SortKey =
  | "title"
  | "playtime"
  | "lastPlayed"
  | "added"
  | "achievements";

export interface Friend {
  id: number;
  name: string;
  online: boolean;
  currentGame: string | null;
  lastSeen: string;
  status: string;
}

export interface StoreAccount {
  provider: string;
  accountId: string;
  profileUrl: string;
  personaName: string;
  avatarUrl: string;
  linkedAt: string;
  lastSyncedAt: string | null;
}

export interface SteamSyncResult {
  owned: number;
  installed: number;
  added: number;
  updated: number;
}

export interface BulkDeleteResult {
  deletedIds: number[];
  skipped: number;
}
