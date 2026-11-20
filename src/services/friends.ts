import type { Friend } from "@/types";
import { useSettingsStore } from "@/stores/settingsStore";

function makeFriends(): Friend[] {
  const ru = useSettingsStore.getState().locale === "ru";
  return [
    {
      id: 1,
      name: "LexaGame",
      online: true,
      currentGame: "Cyberpunk 2077",
      lastSeen: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
      status: ru ? "Играет в Cyberpunk 2077" : "Playing Cyberpunk 2077",
    },
    {
      id: 2,
      name: "Sunny",
      online: true,
      currentGame: "Hogwarts Legacy",
      lastSeen: new Date(Date.now() - 4 * 60 * 60 * 1000).toISOString(),
      status: ru ? "Играет в Hogwarts Legacy" : "Playing Hogwarts Legacy",
    },
    {
      id: 3,
      name: "KillerFox",
      online: false,
      currentGame: null,
      lastSeen: new Date(Date.now() - 26 * 60 * 60 * 1000).toISOString(),
      status: ru ? "В сети" : "Online",
    },
    {
      id: 4,
      name: "MoonDiver",
      online: false,
      currentGame: null,
      lastSeen: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString(),
      status: ru ? "не в сети" : "offline",
    },
  ];
}

export async function fetchFriends(): Promise<Friend[]> {
  await new Promise((r) => setTimeout(r, 150));
  if (!useSettingsStore.getState().demoMode) return [];
  return makeFriends();
}
