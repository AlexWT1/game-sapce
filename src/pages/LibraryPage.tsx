import { useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Grid3x3, List, Plus, Search } from "lucide-react";
import { GameGrid } from "@/components/game/GameGrid";
import { Button } from "@/components/ui/Button";
import { Select } from "@/components/ui/Select";
import { Chip } from "@/components/ui/Chip";
import { useT } from "@/hooks/useT";
import { useGamesStore } from "@/stores/gamesStore";
import { useAchievementsStore } from "@/stores/achievementsStore";
import { useUiStore } from "@/stores/uiStore";
import type { GameFilter, LibraryView, SortKey } from "@/types";
import { cx } from "@/utils/format";

export function LibraryPage() {
  const t = useT();
  const [params, setParams] = useSearchParams();
  const games = useGamesStore((s) => s.games);
  const loading = useGamesStore((s) => s.loading);
  const achievements = useAchievementsStore((s) => s.items);
  const openAddGame = useUiStore((s) => s.openAddGame);

  const [view, setView] = useState<LibraryView>("grid");
  const [sort, setSort] = useState<SortKey>("lastPlayed");
  const [query, setQuery] = useState("");

  const filter = (params.get("filter") as GameFilter | null) ?? "all";
  const genre = params.get("genre");
  const developer = params.get("developer");

  const achCount = useMemo(() => {
    const map = new Map<number, number>();
    for (const a of achievements) {
      if (a.gameId != null) map.set(a.gameId, (map.get(a.gameId) ?? 0) + (a.unlocked ? 1 : 0));
    }
    return map;
  }, [achievements]);

  const list = useMemo(() => {
    let out = games.slice();

    // filters
    switch (filter) {
      case "installed":
        out = out.filter((g) => g.isInstalled && !g.isHidden);
        break;
      case "not-installed":
        out = out.filter((g) => !g.isInstalled && !g.isHidden);
        break;
      case "favorite":
        out = out.filter((g) => g.isFavorite && !g.isHidden);
        break;
      case "recent":
        out = out
          .filter((g) => g.lastPlayedAt && !g.isHidden)
          .sort((a, b) => (b.lastPlayedAt ?? "").localeCompare(a.lastPlayedAt ?? ""));
        break;
      case "controller":
        out = out.filter(
          (g) =>
            !g.isHidden &&
            ["full", "partial", "yes"].includes(g.controllerSupport.toLowerCase()),
        );
        break;
      case "hidden":
        out = out.filter((g) => g.isHidden);
        break;
      default:
        out = out.filter((g) => !g.isHidden);
    }

    if (genre) out = out.filter((g) => g.genre === genre);
    if (developer) out = out.filter((g) => g.developer === developer);

    if (query.trim()) {
      const q = query.trim().toLowerCase();
      out = out.filter(
        (g) =>
          g.title.toLowerCase().includes(q) ||
          g.developer.toLowerCase().includes(q) ||
          g.genre.toLowerCase().includes(q),
      );
    }

    switch (sort) {
      case "title":
        out.sort((a, b) => a.title.localeCompare(b.title));
        break;
      case "playtime":
        out.sort((a, b) => b.totalPlaytimeSeconds - a.totalPlaytimeSeconds);
        break;
      case "lastPlayed":
        out.sort((a, b) => (b.lastPlayedAt ?? "").localeCompare(a.lastPlayedAt ?? ""));
        break;
      case "added":
        out.sort((a, b) => b.createdAt.localeCompare(a.createdAt));
        break;
      case "achievements":
        out.sort(
          (a, b) => (achCount.get(b.id) ?? 0) - (achCount.get(a.id) ?? 0),
        );
        break;
    }

    return out;
  }, [games, filter, genre, developer, query, sort, achCount]);

  const setFilter = (f: GameFilter) => {
    const next = new URLSearchParams(params);
    if (f === "all") next.delete("filter");
    else next.set("filter", f);
    setParams(next, { replace: true });
  };

  return (
    <div className="p-5 space-y-4 min-h-full">
      <div className="flex flex-wrap items-center gap-3">
        <h1 className="text-xl font-bold text-ink mr-2">{t("library.title")}</h1>

        <div className="relative flex-1 min-w-[200px] max-w-xs">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-dim" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t("common.search")}
            className="w-full h-9 pl-9 pr-3 rounded-xl bg-white/[0.05] border border-white/10 text-[13px] text-ink placeholder:text-dim outline-none focus:border-[var(--color-accent)]/60"
          />
        </div>

        <Select
          label={undefined}
          options={[
            { value: "lastPlayed", label: t("library.sort.lastPlayed") },
            { value: "title", label: t("library.sort.title") },
            { value: "playtime", label: t("library.sort.playtime") },
            { value: "added", label: t("library.sort.added") },
            { value: "achievements", label: t("library.sort.achievements") },
          ]}
          value={sort}
          onChange={(e) => setSort(e.target.value as SortKey)}
          className="w-[190px]"
        />

        <div className="flex items-center rounded-xl border border-white/10 overflow-hidden">
          <button
            onClick={() => setView("grid")}
            className={cx(
              "h-9 w-9 grid place-items-center transition",
              view === "grid" ? "bg-[var(--color-accent)]/20 text-[var(--color-accent)]" : "text-muted hover:text-ink",
            )}
            aria-label={t("library.viewGrid")}
          >
            <Grid3x3 className="h-4 w-4" />
          </button>
          <button
            onClick={() => setView("list")}
            className={cx(
              "h-9 w-9 grid place-items-center transition",
              view === "list" ? "bg-[var(--color-accent)]/20 text-[var(--color-accent)]" : "text-muted hover:text-ink",
            )}
            aria-label={t("library.viewList")}
          >
            <List className="h-4 w-4" />
          </button>
        </div>

        <Button size="sm" onClick={() => openAddGame()} icon={<Plus className="h-3.5 w-3.5" />}>
          {t("library.addGame")}
        </Button>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        {(
          [
            ["all", "nav.allGames"],
            ["installed", "nav.installed"],
            ["not-installed", "nav.notInstalled"],
            ["favorite", "nav.favorite"],
            ["recent", "nav.recent"],
            ["controller", "nav.controller"],
            ["hidden", "nav.hidden"],
          ] as Array<[GameFilter, Parameters<ReturnType<typeof useT>>[0]]>
        ).map(([key, labelKey]) => (
          <button key={key} onClick={() => setFilter(key)}>
            <Chip tone={filter === key ? "accent" : "default"}>{t(labelKey)}</Chip>
          </button>
        ))}

        {genre && (
          <button onClick={() => {
            const next = new URLSearchParams(params);
            next.delete("genre");
            setParams(next, { replace: true });
          }}>
            <Chip tone="epic">{genre} ×</Chip>
          </button>
        )}
        {developer && (
          <button onClick={() => {
            const next = new URLSearchParams(params);
            next.delete("developer");
            setParams(next, { replace: true });
          }}>
            <Chip tone="warning">{developer} ×</Chip>
          </button>
        )}

        <span className="ml-auto text-[12px] text-dim tabular-nums">
          {t("library.results")}: {list.length}
        </span>
      </div>

      <GameGrid
        games={list}
        loading={loading}
        variant={view}
        onAdd={() => openAddGame()}
      />
    </div>
  );
}
