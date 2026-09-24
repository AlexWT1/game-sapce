import type { GameTheme } from "@/types";

const DEFAULT: GameTheme = {
  gameId: 0,
  primaryColor: "#0f1119",
  secondaryColor: "#7c5cff",
  accentColor: "#a78bfa",
  backgroundColor: "#0a0c1c",
  glowColor: "#7c5cff",
  fontFamily: "Inter",
};

/** Applies game theme CSS variables to an element (or :root if omitted). */
export function applyGameTheme(el: HTMLElement | null, theme: GameTheme | null): void {
  const target = el ?? document.documentElement;
  const t = theme ?? DEFAULT;
  target.style.setProperty("--game-primary", t.primaryColor);
  target.style.setProperty("--game-secondary", t.secondaryColor);
  target.style.setProperty("--game-accent", t.accentColor);
  target.style.setProperty("--game-background", t.backgroundColor);
  target.style.setProperty("--game-glow", t.glowColor);
  target.style.setProperty("--game-font", t.fontFamily);
}

export function resetGameTheme(el: HTMLElement | null): void {
  applyGameTheme(el, null);
}
