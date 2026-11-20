import { useEffect } from "react";
import { BarChart3, Clock, Gamepad2, Layers, Sparkles, Trophy } from "lucide-react";
import { Link } from "react-router-dom";
import { GenreChart, PlaytimeChart } from "@/components/charts/Charts";
import { EmptyState } from "@/components/ui/EmptyState";
import { Skeleton } from "@/components/ui/Skeleton";
import { GameCoverThumb } from "@/components/game/GameCoverThumb";
import { useT } from "@/hooks/useT";
import { useStatisticsStore } from "@/stores/statisticsStore";
import { useGamesStore } from "@/stores/gamesStore";
import { formatNumber, formatPlaytime } from "@/utils/format";

export function StatisticsPage() {
  const t = useT();
  const stats = useStatisticsStore((s) => s.stats);
  const loading = useStatisticsStore((s) => s.loading);
  const load = useStatisticsStore((s) => s.load);
  const games = useGamesStore((s) => s.games);

  useEffect(() => {
    void load(true);
  }, [load]);

  if (loading && !stats) {
    return (
      <div className="p-5 space-y-4">
        <Skeleton className="h-10 w-56" />
        <div className="grid sm:grid-cols-2 xl:grid-cols-4 gap-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <Skeleton key={i} className="h-28 rounded-2xl" />
          ))}
        </div>
        <Skeleton className="h-64 rounded-2xl" />
      </div>
    );
  }

  if (!stats || stats.gamesCount === 0) {
    return (
      <div className="p-5">
        <EmptyState
          icon={<BarChart3 className="h-6 w-6" />}
          title={t("stats.empty")}
          body={t("library.emptyHint")}
          action={
            <Link to="/library" className="text-[var(--color-accent)] text-sm hover:underline">
              {t("nav.library")} →
            </Link>
          }
        />
      </div>
    );
  }

  const cards = [
    {
      icon: <Clock className="h-5 w-5 text-[var(--color-accent)]" />,
      label: t("stats.totalPlaytime"),
      value: formatPlaytime(stats.totalPlaytimeSeconds),
    },
    {
      icon: <Gamepad2 className="h-5 w-5 text-sky-400" />,
      label: t("stats.games"),
      value: formatNumber(stats.gamesCount),
      sub: `${stats.installedCount} ${t("game.installed").toLowerCase()}`,
    },
    {
      icon: <Trophy className="h-5 w-5 text-amber-400" />,
      label: t("stats.achievements"),
      value: `${stats.achievementsUnlocked} / ${stats.achievementsTotal}`,
    },
    {
      icon: <Layers className="h-5 w-5 text-emerald-400" />,
      label: t("stats.sessions"),
      value: formatNumber(stats.sessionsCount),
      sub: `${t("stats.avgSession")}: ${formatPlaytime(stats.avgSessionSeconds)}`,
    },
  ];

  return (
    <div className="p-5 space-y-5 min-h-full">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-bold text-ink">{t("stats.title")}</h1>
        <div className="flex items-center gap-3">
          <Link
            to="/recap"
            className="inline-flex items-center gap-1.5 h-8 px-3 rounded-lg border border-white/15 text-[13px] text-ink hover:bg-white/8 hover:border-white/25 transition"
          >
            <Sparkles className="h-3.5 w-3.5 text-[var(--color-accent)]" />
            {t("recap.viewRecap")}
          </Link>
          <span className="text-[12.5px] text-dim">
            {t("stats.longestSession")}: {formatPlaytime(stats.longestSessionSeconds)}
          </span>
        </div>
      </div>

      <div className="grid sm:grid-cols-2 xl:grid-cols-4 gap-4">
        {cards.map((c) => (
          <div key={c.label} className="glass rounded-2xl p-4">
            <div className="flex items-center gap-2 text-muted text-[12.5px] mb-2">
              {c.icon}
              {c.label}
            </div>
            <div className="text-[26px] font-bold text-ink tracking-tight tabular-nums leading-none">
              {c.value}
            </div>
            {c.sub && <div className="text-[12px] text-dim mt-1.5">{c.sub}</div>}
          </div>
        ))}
      </div>

      <div className="grid gap-4 lg:grid-cols-3">
        <section className="glass rounded-2xl p-5 lg:col-span-2">
          <h2 className="text-[14px] font-semibold text-ink mb-4">{t("stats.byDay")}</h2>
          <PlaytimeChart data={stats.daily} height={220} />
        </section>

        <section className="glass rounded-2xl p-5">
          <h2 className="text-[14px] font-semibold text-ink mb-2">{t("stats.byGenre")}</h2>
          <GenreChart data={stats.byGenre} height={180} />
          <ul className="mt-2 space-y-1.5">
            {stats.byGenre.slice(0, 5).map((g) => (
              <li key={g.genre} className="flex justify-between text-[12.5px]">
                <span className="text-muted">{g.genre}</span>
                <span className="text-ink tabular-nums">{g.count}</span>
              </li>
            ))}
          </ul>
        </section>
      </div>

      <div className="grid gap-4 lg:grid-cols-3">
        <section className="glass rounded-2xl p-5 lg:col-span-2">
          <h2 className="text-[14px] font-semibold text-ink mb-4">{t("stats.byWeek")}</h2>
          <PlaytimeChart data={stats.weekly} height={180} />
        </section>

        <section className="glass rounded-2xl p-5">
          <h2 className="text-[14px] font-semibold text-ink mb-3">{t("stats.mostPlayed")}</h2>
          <ul className="space-y-2.5">
            {stats.mostPlayed.map((g, i) => (
              <li key={g.gameId}>
                <Link
                  to={`/game/${g.gameId}`}
                  className="flex items-center gap-3 px-2 py-1.5 rounded-xl hover:bg-white/5 transition"
                >
                  <span className="text-[12px] text-dim w-4 tabular-nums">{i + 1}</span>
                  <GameCoverThumb src={g.coverPath} title={g.title} size={34} />
                  <div className="min-w-0 flex-1">
                    <p className="text-[13px] font-medium text-ink truncate">{g.title}</p>
                    <p className="text-[11.5px] text-muted tabular-nums">
                      {formatPlaytime(g.seconds)}
                    </p>
                  </div>
                </Link>
              </li>
            ))}
          </ul>
          {games.length > stats.mostPlayed.length && (
            <p className="mt-3 text-[12px] text-dim">
              {t("stats.andMore").replace("{n}", String(games.length - stats.mostPlayed.length))}
            </p>
          )}
        </section>
      </div>

      <section className="glass rounded-2xl p-5">
        <h2 className="text-[14px] font-semibold text-ink mb-4">{t("stats.byMonth")}</h2>
        <PlaytimeChart data={stats.monthly} height={180} />
      </section>
    </div>
  );
}
