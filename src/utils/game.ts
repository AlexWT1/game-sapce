import type { Game } from "@/types";

/** Games without a local executable can still be launched when they come from a store.
 *  Launcher-based games are launchable as long as the launcher file is present. */
export function canLaunch(game: Game): boolean {
  if (game.useLauncher) return Boolean(game.launcherPath);
  return Boolean(game.executablePath) || game.source === "Steam";
}

/** Steam-linked games can be (re)installed through the Steam client. */
export function isSteamGame(game: Game): boolean {
  return game.source === "Steam";
}
