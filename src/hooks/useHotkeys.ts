import { useEffect } from "react";

export type HotkeyHandler = (e: KeyboardEvent) => void;

function matches(e: KeyboardEvent, combo: string): boolean {
  const parts = combo.toLowerCase().split("+");
  const key = parts[parts.length - 1] ?? "";
  const needCtrl = parts.includes("ctrl");
  const needMeta = parts.includes("meta") || parts.includes("cmd");
  const needShift = parts.includes("shift");
  const needAlt = parts.includes("alt");

  if (needCtrl !== (e.ctrlKey || e.metaKey)) return false;
  if (needMeta && !e.metaKey) return false;
  if (needAlt !== e.altKey) return false;
  if (needShift !== e.shiftKey) return false;

  const pressed = e.key.toLowerCase();
  if (key === "k" && pressed === "k") return true;
  if (key === "escape" && pressed === "escape") return true;
  return pressed === key;
}

/** Registers global keyboard shortcuts. Combo examples: "ctrl+k", "escape". */
export function useHotkeys(bindings: Array<{ combo: string; handler: HotkeyHandler }>) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      for (const b of bindings) {
        if (matches(e, b.combo)) {
          e.preventDefault();
          b.handler(e);
          return;
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [bindings]);
}
