import type { Friend } from "@/types";
import { useSettingsStore } from "@/stores/settingsStore";

/**
 * Mock friends data. Architecture is ready for a real backend later —
 * swap `fetchFriends` implementation when an API exists.
 */
const FRIENDS: Friend[] = [
  {
    id: 1,
    name: "LexaGame",
    online: true,
    currentGame: "Cyberpunk 2077",
    lastSeen: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
    status: "Играет в Cyberpunk 2077",
  },
  {
    id: 2,
    name: "Sunny",
    online: true,
    currentGame: "Hogwarts Legacy",
    lastSeen: new Date(Date.now() - 4 * 60 * 60 * 1000).toISOString(),
    status: "Играет в Hogwarts Legacy",
  },
  {
    id: 3,
    name: "KillerFox",
    online: false,
    currentGame: null,
    lastSeen: new Date(Date.now() - 26 * 60 * 60 * 1000).toISOString(),
    status: "В сети",
  },
  {
    id: 4,
    name: "MoonDiver",
    online: false,
    currentGame: null,
    lastSeen: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString(),
    status: "не в сети",
  },
];

export async function fetchFriends(): Promise<Friend[]> {
  // Simulate tiny latency so skeletons are visible.
  await new Promise((r) => setTimeout(r, 150));
  if (!useSettingsStore.getState().demoMode) return [];
  return FRIENDS.map((f) => ({ ...f }));
}
