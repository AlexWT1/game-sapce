import {
  BarChart3,
  Clock,
  Gamepad2,
  Heart,
  Home,
  LayoutGrid,
  Library,
  Settings,
  ShoppingBag,
  Trophy,
  Users,
  EyeOff,
  Gamepad,
} from "lucide-react";
import { useState } from "react";
import { NavLink, useNavigate, useLocation } from "react-router-dom";
import { useT } from "@/hooks/useT";
import { useUiStore } from "@/stores/uiStore";
import { useGamesStore } from "@/stores/gamesStore";
import { cx } from "@/utils/format";
import type { GameFilter } from "@/types";

interface NavItem {
  to: string;
  labelKey: "nav.home" | "nav.library" | "nav.achievements" | "nav.statistics" | "nav.friends" | "nav.shop" | "nav.settings";
  icon: React.ReactNode;
}

const NAV: NavItem[] = [
  { to: "/", labelKey: "nav.home", icon: <Home className="h-[18px] w-[18px]" /> },
  { to: "/library", labelKey: "nav.library", icon: <Library className="h-[18px] w-[18px]" /> },
  { to: "/achievements", labelKey: "nav.achievements", icon: <Trophy className="h-[18px] w-[18px]" /> },
  { to: "/statistics", labelKey: "nav.statistics", icon: <BarChart3 className="h-[18px] w-[18px]" /> },
  { to: "/friends", labelKey: "nav.friends", icon: <Users className="h-[18px] w-[18px]" /> },
  { to: "/shop", labelKey: "nav.shop", icon: <ShoppingBag className="h-[18px] w-[18px]" /> },
  { to: "/settings", labelKey: "nav.settings", icon: <Settings className="h-[18px] w-[18px]" /> },
];

const FILTERS: Array<{ key: GameFilter; labelKey: Parameters<ReturnType<typeof useT>>[0]; icon: React.ReactNode }> = [
  { key: "all", labelKey: "nav.allGames", icon: <LayoutGrid className="h-4 w-4" /> },
  { key: "installed", labelKey: "nav.installed", icon: <Gamepad2 className="h-4 w-4" /> },
  { key: "favorite", labelKey: "nav.favorite", icon: <Heart className="h-4 w-4" /> },
  { key: "recent", labelKey: "nav.recent", icon: <Clock className="h-4 w-4" /> },
  { key: "controller", labelKey: "nav.controller", icon: <Gamepad className="h-4 w-4" /> },
  { key: "hidden", labelKey: "nav.hidden", icon: <EyeOff className="h-4 w-4" /> },
];

export function Sidebar() {
  const t = useT();
  const collapsed = useUiStore((s) => s.sidebarCollapsed);
  const games = useGamesStore((s) => s.games);
  const navigate = useNavigate();
  const location = useLocation();

  const myGames = games
    .filter((g) => !g.isHidden)
    .slice()
    .sort((a, b) => (b.lastPlayedAt ?? "").localeCompare(a.lastPlayedAt ?? ""))
    .slice(0, 8);

  const activeFilter = (key: GameFilter) =>
    location.pathname === "/library" &&
    new URLSearchParams(location.search).get("filter") === key;

  return (
    <div className="relative h-full shrink-0">
      <aside
        className={cx(
          "relative z-30 h-full flex flex-col border-r border-white/8",
          "bg-[#0a0c18]/80 backdrop-blur-xl transition-[width] duration-300 ease-in-out overflow-hidden",
          collapsed ? "w-[72px]" : "w-[var(--sidebar-width)]",
        )}
        aria-label="Sidebar"
      >
        <div
          data-tauri-drag-region="deep"
          className={cx(
            "flex items-center h-[var(--topbar-height)] border-b border-white/8",
            "transition-[padding] duration-300 ease-in-out",
            collapsed ? "px-5" : "px-4",
          )}
        >
          <div className="flex items-center gap-2.5 min-w-0">
            <div className="h-8 w-8 rounded-xl bg-gradient-to-br from-[#7c5cff] to-[#3b82f6] grid place-items-center shrink-0 shadow-[0_0_20px_-4px_#7c5cff]">
              <Gamepad2 className="h-4.5 w-4.5 h-[18px] w-[18px] text-white" />
            </div>
            <div
              className={cx(
                "min-w-0 overflow-hidden transition-all duration-300 ease-in-out",
                collapsed ? "opacity-0 w-0" : "opacity-100 max-w-[180px]",
              )}
            >
              <div className="text-[15px] font-bold tracking-tight text-ink leading-none whitespace-nowrap">
                Game Space
              </div>
            </div>
          </div>
        </div>

        <nav
          className={cx(
            "flex-1 overflow-y-auto no-scrollbar py-3 space-y-1",
            "transition-[padding] duration-300 ease-in-out",
            collapsed ? "px-3.5" : "px-2.5",
          )}
        >
          {NAV.map((item) => (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.to === "/"}
              className={({ isActive }) =>
                cx(
                  "group relative flex items-center gap-3 px-3 h-10 rounded-xl text-[13.5px] transition",
                  isActive
                    ? "bg-[var(--color-accent)]/14 text-ink font-medium"
                    : "text-muted hover:text-ink hover:bg-white/6",
                )
              }
            >
            {({ isActive }) => (
              <>
                {isActive && (
                  <span className="absolute left-0 top-1/2 -translate-y-1/2 h-5 w-[3px] rounded-full bg-[var(--color-accent)] shadow-[0_0_8px_var(--color-accent)]" />
                )}
                <span className={isActive ? "text-[var(--color-accent)]" : "text-muted group-hover:text-ink"}>
                  {item.icon}
                </span>
                <span
                  className={cx(
                    "truncate overflow-hidden transition-all duration-300 ease-in-out",
                    collapsed
                      ? "max-w-0 opacity-0"
                      : "max-w-[160px] opacity-100",
                  )}
                >
                  {t(item.labelKey)}
                </span>
              </>
            )}
          </NavLink>
        ))}

        <div
          className={cx(
            "transition-all duration-300 ease-in-out overflow-hidden",
            collapsed
              ? "opacity-0 max-h-0 pointer-events-none"
              : "opacity-100 max-h-[800px]",
          )}
          aria-hidden={collapsed}
        >
          <div className="pt-5 pb-1.5 px-3 text-[10.5px] font-semibold tracking-[0.12em] text-dim">
            {t("nav.myGames")}
          </div>
          <div className="space-y-0.5">
            {myGames.map((g) => (
              <button
                key={g.id}
                onClick={() => navigate(`/game/${g.id}`)}
                className={cx(
                  "w-full flex items-center gap-2.5 px-3 h-9 rounded-lg text-[13px] transition text-left",
                  location.pathname === `/game/${g.id}`
                    ? "bg-white/8 text-ink"
                    : "text-muted hover:text-ink hover:bg-white/5",
                )}
                title={g.title}
              >
                <GameCoverThumb src={g.coverPath} title={g.title} />
                <span className="truncate">{g.title}</span>
              </button>
            ))}
          </div>

          <div className="pt-5 pb-1.5 px-3 text-[10.5px] font-semibold tracking-[0.12em] text-dim">
            {t("nav.filters")}
          </div>
          <div className="space-y-0.5">
            {FILTERS.map((f) => (
              <button
                key={f.key}
                onClick={() =>
                  navigate(
                    f.key === "all" ? "/library" : `/library?filter=${f.key}`,
                  )
                }
                className={cx(
                  "w-full flex items-center gap-3 px-3 h-9 rounded-lg text-[13px] transition text-left",
                  activeFilter(f.key)
                    ? "bg-[var(--color-accent)]/14 text-ink"
                    : "text-muted hover:text-ink hover:bg-white/5",
                )}
              >
                <span className={activeFilter(f.key) ? "text-[var(--color-accent)]" : "text-dim"}>
                  {f.icon}
                </span>
                <span className="truncate">{t(f.labelKey)}</span>
              </button>
            ))}
          </div>
        </div>
      </nav>

      <div
        className={cx(
          "transition-all duration-300 ease-in-out overflow-hidden",
          collapsed ? "opacity-0 max-h-0 p-0" : "opacity-100 max-h-40 p-3",
        )}
        aria-hidden={collapsed}
      >
        <div className="glass rounded-2xl p-3.5 flex items-center gap-3">
          <div className="h-10 w-10 rounded-xl bg-gradient-to-br from-[#7c5cff]/40 to-[#3b82f6]/30 grid place-items-center shrink-0">
            <Gamepad2 className="h-5 w-5 text-[#a78bfa]" />
          </div>
          <div className="min-w-0">
            <div className="text-[12.5px] font-semibold text-ink">Game Space</div>
            <div className="text-[11px] text-dim leading-snug truncate">
              {t("app.tagline")}
            </div>
          </div>
        </div>
      </div>
      </aside>
    </div>
  );
}

function GameCoverThumb({ src, title }: { src: string | null; title: string }) {
  const [broken, setBroken] = useState<string | null>(null);
  if (src && broken !== src) {
    return (
      <img
        src={src}
        alt=""
        className="h-5.5 w-5 h-[22px] w-[22px] rounded object-cover shrink-0"
        loading="lazy"
        onError={() => setBroken(src)}
      />
    );
  }
  return (
    <span className="h-[22px] w-[22px] rounded bg-white/10 grid place-items-center text-[9px] font-bold text-muted shrink-0">
      {title.slice(0, 1).toUpperCase()}
    </span>
  );
}
