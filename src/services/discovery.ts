import { api } from "@/services/api";
import { useGamesStore } from "@/stores/gamesStore";

export interface ScanResult {
  found: number;
  added: number;
}

/**
 * Scans the given directories (or the configured/default ones when empty)
 * for game executables and adds everything that is not in the library yet.
 */
export async function scanForGames(
  dirs: string[] = [],
  limit = 100,
): Promise<ScanResult> {
  const discovered = await api.discoverGames(dirs);
  let added = 0;

  for (const d of discovered.slice(0, limit)) {
    try {
      const exists = await api.gameExists(d.title);
      if (!exists) {
        await useGamesStore.getState().create({
          title: d.title,
          executablePath: d.path,
          installPath: d.installDir,
          platform: d.platform,
          isInstalled: true,
        });
        added += 1;
      }
    } catch {
      /* skip entries that fail to import */
    }
  }

  if (added > 0) {
    await useGamesStore.getState().load(true);
  }
  return { found: discovered.length, added };
}
