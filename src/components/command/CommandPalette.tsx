import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { AnimatePresence, motion } from "framer-motion";
import {
  ArrowRight,
  Gamepad2,
  Hash,
  CornerDownLeft,
  Trophy,
  User,
} from "lucide-react";
import { useNavigate } from "react-router-dom";
import { api } from "@/services/api";
import { useUiStore } from "@/stores/uiStore";
import { useT } from "@/hooks/useT";
import { cx, formatPlaytime } from "@/utils/format";
import type { SearchResults } from "@/types";

interface Cmd {
  id: string;
  group: string;
  label: string;
  hint?: string;
  icon: React.ReactNode;
  run: () => void;
}

export function CommandPalette() {
  const open = useUiStore((s) => s.commandPaletteOpen);
  const setOpen = useUiStore((s) => s.setCommandPalette);
  const openAddGame = useUiStore((s) => s.openAddGame);
  const t = useT();
  const navigate = useNavigate();
  const inputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResults | null>(null);
  const [cursor, setCursor] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (open) {
      setQuery("");
      setResults(null);
      setCursor(0);
      window.setTimeout(() => inputRef.current?.focus(), 30);
    }
  }, [open]);

  useEffect(() => {
    if (!open || query.trim().length < 1) {
      setResults(null);
      return;
    }
    let cancelled = false;
    const id = window.setTimeout(() => {
      api
        .searchAll(query)
        .then((r) => {
          if (!cancelled) setResults(r);
        })
        .catch(() => {
          if (!cancelled) setResults(null);
        });
    }, 120);
    return () => {
      cancelled = true;
      window.clearTimeout(id);
    };
  }, [open, query]);

  const navCmds: Cmd[] = useMemo(
    () => [
      {
        id: "nav-home",
        group: t("command.navigate"),
        label: t("nav.home"),
        icon: <Gamepad2 className="h-4 w-4" />,
        run: () => navigate("/"),
      },
      {
        id: "nav-library",
        group: t("command.navigate"),
        label: t("nav.library"),
        icon: <Gamepad2 className="h-4 w-4" />,
        run: () => navigate("/library"),
      },
      {
        id: "nav-achievements",
        group: t("command.navigate"),
        label: t("nav.achievements"),
        icon: <Trophy className="h-4 w-4" />,
        run: () => navigate("/achievements"),
      },
      {
        id: "nav-stats",
        group: t("command.navigate"),
        label: t("nav.statistics"),
        icon: <Hash className="h-4 w-4" />,
        run: () => navigate("/statistics"),
      },
      {
        id: "nav-friends",
        group: t("command.navigate"),
        label: t("nav.friends"),
        icon: <User className="h-4 w-4" />,
        run: () => navigate("/friends"),
      },
      {
        id: "nav-settings",
        group: t("command.navigate"),
        label: t("nav.settings"),
        icon: <Gamepad2 className="h-4 w-4" />,
        run: () => navigate("/settings"),
      },
      {
        id: "add-game",
        group: t("command.open"),
        label: t("library.addGame"),
        icon: <Gamepad2 className="h-4 w-4" />,
        run: () => openAddGame(),
      },
    ],
    [navigate, openAddGame, t],
  );

  const filteredNav = query.trim()
    ? navCmds.filter((c) => c.label.toLowerCase().includes(query.toLowerCase()))
    : navCmds;

  const gameCmds: Cmd[] = (results?.games ?? []).map((g) => ({
    id: `game-${g.id}`,
    group: t("command.games"),
    label: g.title,
    hint: g.genre || g.developer || undefined,
    icon: <Gamepad2 className="h-4 w-4" />,
    run: () => navigate(`/game/${g.id}`),
  }));

  const achievementCmds: Cmd[] = (results?.achievements ?? []).map((a) => ({
    id: `ach-${a.id}`,
    group: t("command.achievements"),
    label: a.title,
    hint: a.gameTitle ?? undefined,
    icon: <Trophy className="h-4 w-4" />,
    run: () => navigate("/achievements"),
  }));

  const genreCmds: Cmd[] = (results?.genres ?? []).map((g) => ({
    id: `genre-${g}`,
    group: t("command.genres"),
    label: g,
    icon: <Hash className="h-4 w-4" />,
    run: () => navigate(`/library?genre=${encodeURIComponent(g)}`),
  }));

  const devCmds: Cmd[] = (results?.developers ?? []).map((d) => ({
    id: `dev-${d}`,
    group: t("command.developers"),
    label: d,
    icon: <User className="h-4 w-4" />,
    run: () => navigate(`/library?developer=${encodeURIComponent(d)}`),
  }));

  const all = [...gameCmds, ...filteredNav, ...achievementCmds, ...genreCmds, ...devCmds];

  useEffect(() => {
    setCursor((c) => Math.min(c, Math.max(0, all.length - 1)));
  }, [all.length]);

  const runCursor = () => {
    const cmd = all[cursor];
    if (!cmd) return;
    cmd.run();
    setOpen(false);
  };

  if (!open) return null;

  return createPortal(
    <AnimatePresence>
      <motion.div
        className="fixed inset-0 z-[85] flex items-start justify-center pt-[12vh] px-4"
        initial={{ opacity: 1 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.1 }}
      >
        <div className="absolute inset-0 bg-black/60 backdrop-blur-sm" onClick={() => setOpen(false)} />
        <motion.div
          className="relative w-full max-w-xl rounded-2xl overflow-hidden border border-white/10 bg-[#0d1020]/95 shadow-2xl"
          initial={{ opacity: 1, y: 0, scale: 1 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, y: -8 }}
          transition={{ duration: 0.12 }}
        >
          <div className="flex items-center gap-2.5 px-4 h-14 border-b border-white/8">
            <Gamepad2 className="h-4 w-4 text-[var(--color-accent)]" />
            <input
              ref={inputRef}
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setCursor(0);
              }}
              onKeyDown={(e) => {
                if (e.key === "ArrowDown") {
                  e.preventDefault();
                  setCursor((c) => Math.min(all.length - 1, c + 1));
                } else if (e.key === "ArrowUp") {
                  e.preventDefault();
                  setCursor((c) => Math.max(0, c - 1));
                } else if (e.key === "Enter") {
                  e.preventDefault();
                  runCursor();
                }
              }}
              placeholder={t("command.placeholder")}
              className="flex-1 bg-transparent outline-none text-[15px] text-ink placeholder:text-dim"
            />
            <kbd className="text-[10px] text-dim border border-white/10 rounded px-1.5 py-0.5">
              Esc
            </kbd>
          </div>

          <div ref={listRef} className="max-h-[50vh] overflow-y-auto py-2">
            {all.length === 0 ? (
              <p className="text-center text-sm text-dim py-8">{t("command.noResults")}</p>
            ) : (
              all.map((cmd, i) => {
                const active = i === cursor;
                const showGroup = i === 0 || all[i - 1]?.group !== cmd.group;
                return (
                  <div key={cmd.id}>
                    {showGroup && (
                      <div className="px-4 pt-2 pb-1 text-[10.5px] font-semibold tracking-[0.1em] text-dim uppercase">
                        {cmd.group}
                      </div>
                    )}
                    <button
                      onMouseEnter={() => setCursor(i)}
                      onClick={() => {
                        cmd.run();
                        setOpen(false);
                      }}
                      className={cx(
                        "w-full flex items-center gap-3 px-4 py-2.5 text-left transition",
                        active ? "bg-[var(--color-accent)]/14" : "hover:bg-white/5",
                      )}
                    >
                      <span className={active ? "text-[var(--color-accent)]" : "text-muted"}>
                        {cmd.icon}
                      </span>
                      <span className="flex-1 text-[13.5px] text-ink truncate">{cmd.label}</span>
                      {cmd.hint && (
                        <span className="text-[11.5px] text-dim truncate max-w-[140px]">
                          {cmd.hint}
                        </span>
                      )}
                      {active && <CornerDownLeft className="h-3.5 w-3.5 text-dim" />}
                    </button>
                  </div>
                );
              })
            )}
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>,
    document.body,
  );
}

// silence unused import if tree-shaken elsewhere
export const __paletteFormat = formatPlaytime;
export const __paletteArrow = ArrowRight;
