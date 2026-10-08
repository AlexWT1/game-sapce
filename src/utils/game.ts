import type { Game } from "@/types";

/** Games without a local executable can still be launched when they come from a store. */
export function canLaunch(game: Game): boolean {
  return Boolean(game.executablePath) || game.source === "Steam";
}

/** Steam-linked games can be (re)installed through the Steam client. */
export function isSteamGame(game: Game): boolean {
  return game.source === "Steam";
}
