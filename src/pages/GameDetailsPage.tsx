import { useEffect, useMemo, useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  ArrowLeft,
  Calendar,
  Clock,
  Gamepad2,
  Heart,
  Monitor,
  Palette,
  Pencil,
  Tag,
  Trophy,
} from "lucide-react";
import { LaunchButton } from "@/components/game/LaunchButton";
import { ProgressBar } from "@/components/ui/ProgressBar";
import { Chip } from "@/components/ui/Chip";
import { ContextMenu } from "@/components/ui/ContextMenu";
import { buildGameMenu } from "@/components/game/GameCard";
import { PlaytimeChart } from "@/components/charts/Charts";
import { EmptyState } from "@/components/ui/EmptyState";
import { useT, useLocaleCode } from "@/hooks/useT";
import { useGamesStore } from "@/stores/gamesStore";
import { useSessionStore } from "@/stores/sessionStore";
import { useStatisticsStore } from "@/stores/statisticsStore";
import { useAchievementsStore } from "@/stores/achievementsStore";
import { useUiStore } from "@/stores/uiStore";
import { api } from "@/services/api";
import { applyGameTheme } from "@/utils/theme";
import {
  cx,
  formatDateTime,
  formatHMS,
  formatNumber,
  formatPlaytime,
  percent,
} from "@/utils/format";

export function GameDetailsPage() {
  const { id } = useParams();
  const gameId = Number(id);
  const t = useT();
  const locale = useLocaleCode();
  const games = useGamesStore((s) => s.games);
  const toggleFavorite = useGamesStore((s) => s.toggleFavorite);
  const openEdit = useUiStore((s) => s.openEditGame);
  const running = useSessionStore((s) => s.running.some((r) => r.gameId === gameId));
  const live = useSessionStore((s) => s.liveSeconds[gameId] ?? 0);
  const gameStats = useStatisticsStore((s) => s.gameStats[gameId]);
  const loadGameStats = useStatisticsStore((s) => s.loadGame);
  const achievements = useAchievementsStore((s) => s.byGame[gameId]);
  const rootRef = useRef<HTMLDivElement>(null);

  const [sessions, setSessions] = useState<
    Array<{ id: number; startedAt: string; endedAt: string | null; durationSeconds: number }>
  >([]);
  const [themeLoaded, setThemeLoaded] = useState(false);
  const [brokenCover, setBrokenCover] = useState<string | null>(null);

  const game = games.find((g) => g.id === gameId);

  useEffect(() => {
    if (!gameId) return;
    void loadGameStats(gameId, true);
    api
      .getGameSessions(gameId)
      .then(setSessions)
      .catch(() => setSessions([]));
    api
      .getGameTheme(gameId)
      .then((theme) => applyGameTheme(rootRef.current, theme))
      .catch(() => applyGameTheme(rootRef.current, null))
      .finally(() => setThemeLoaded(true));
    void useAchievementsStore.getState().load(gameId, true);
  }, [gameId, loadGameStats, running]);

  const gameAch = useMemo(() => achievements ?? [], [achievements]);
  const unlocked = gameAch.filter((a) => a.unlocked).length;

  if (!game) {
    return (
      <div className="p-8">
        <EmptyState title={t("game.notFound")} action={<Link to="/library" className="text-[var(--color-accent)]">← {t("nav.library")}</Link>} />
      </div>
    );
  }

  const menu = buildGameMenu(game, t);
  const onCoverError = () => setBrokenCover(game.coverPath);

  return (
    <div ref={rootRef} className="min-h-full" data-theme-loaded={themeLoaded}>
      {/* Hero */}
      <ContextMenu items={menu} className="w-full">
        <section className="relative h-[280px] overflow-hidden vignette border-b border-white/8">
          {game.coverPath && brokenCover !== game.coverPath && (
            <img
              src={game.coverPath}
              alt=""
              className="absolute inset-0 h-full w-full object-cover object-[center_25%] scale-110 blur-[1px] opacity-70"
              onError={onCoverError}
            />
          )}
          <div className="absolute inset-0 bg-gradient-to-t from-[#070912] via-[#070912]/70 to-[#070912]/40" />
          <div
            className="absolute inset-0 opacity-30"
            style={{
              background:
                "radial-gradient(600px 200px at 20% 100%, var(--game-glow), transparent)",
            }}
          />

          <div className="relative h-full flex items-end gap-6 px-6 pb-6 max-w-[1100px]">
            <div className="h-[160px] w-[120px] rounded-xl overflow-hidden border border-white/15 shadow-2xl shrink-0">
              {game.coverPath && brokenCover !== game.coverPath ? (
                <img
                  src={game.coverPath}
                  alt={game.title}
                  className="h-full w-full object-cover"
                  onError={onCoverError}
                />
              ) : (
                <div className="h-full w-full grid place-items-center bg-white/5">
                  <Gamepad2 className="h-8 w-8 text-dim" />
                </div>
              )}
            </div>

            <div className="min-w-0 flex-1">
              <Link
                to="/library"
                className="inline-flex items-center gap-1.5 text-[12px] text-muted hover:text-ink mb-2 transition"
              >
                <ArrowLeft className="h-3.5 w-3.5" /> {t("nav.library")}
              </Link>
              <h1 className="text-[28px] font-extrabold text-gradient leading-tight truncate">
                {game.title}
              </h1>
              <div className="mt-2 flex flex-wrap items-center gap-2">
                {game.genre && <Chip tone="accent">{game.genre}</Chip>}
                {game.releaseYear && <Chip>{game.releaseYear}</Chip>}
                <Chip tone={game.isInstalled ? "success" : "danger"}>
                  {!game.isInstalled && (
                    <span className="h-1.5 w-1.5 rounded-full bg-red-400" />
                  )}
                  {game.isInstalled ? t("game.installed") : t("game.notInstalled")}
                </Chip>
                {game.controllerSupport !== "unknown" && (
                  <Chip>🎮 {game.controllerSupport}</Chip>
                )}
              </div>
            </div>

            <div className="flex items-center gap-2 shrink-0">
              <button
                onClick={() => void toggleFavorite(game.id)}
                className={cx(
                  "h-11 w-11 grid place-items-center rounded-xl border transition",
                  game.isFavorite
                    ? "bg-rose-500/15 border-rose-400/40 text-rose-400"
                    : "bg-white/6 border-white/10 text-muted hover:text-ink",
                )}
                aria-label={t("game.favorite")}
              >
                <Heart className={cx("h-5 w-5", game.isFavorite && "fill-current")} />
              </button>
              <button
                onClick={() => openEdit(game)}
                className="h-11 w-11 grid place-items-center rounded-xl bg-white/6 border border-white/10 text-muted hover:text-ink transition"
                aria-label={t("game.edit")}
              >
                <Pencil className="h-4.5 w-4.5 h-[18px] w-[18px]" />
              </button>
              <LaunchButton game={game} size="lg" />
            </div>
          </div>
        </section>
      </ContextMenu>

      <div className="p-6 grid gap-6 lg:grid-cols-3 max-w-[1200px]">
        {/* Live strip */}
        {running && (
          <div className="lg:col-span-3 glass rounded-2xl px-4 py-3 flex items-center gap-4 border-emerald-400/25">
            <span className="live-dot" />
            <span className="text-[13px] font-medium text-emerald-300">
              {t("home.statusPlaying")}
            </span>
            <span className="text-lg font-bold tabular-nums text-ink ml-auto">
              {formatHMS(live)}
            </span>
          </div>
        )}

        {/* About */}
        <section className="lg:col-span-2 glass rounded-2xl p-5">
          <h2 className="text-[15px] font-semibold text-ink mb-3">{t("game.about")}</h2>
          <p className="text-[13.5px] text-muted leading-relaxed whitespace-pre-wrap">
            {game.description || "—"}
          </p>

          <dl className="mt-5 grid grid-cols-2 sm:grid-cols-3 gap-4 text-[13px]">
            <Meta icon={<Gamepad2 className="h-4 w-4" />} label={t("game.developer")} value={game.developer || "—"} />
            <Meta icon={<Tag className="h-4 w-4" />} label={t("game.publisher")} value={game.publisher || "—"} />
            <Meta icon={<Tag className="h-4 w-4" />} label={t("game.genre")} value={game.genre || "—"} />
            <Meta icon={<Calendar className="h-4 w-4" />} label={t("game.year")} value={String(game.releaseYear ?? "—")} />
            <Meta icon={<Monitor className="h-4 w-4" />} label={t("game.platform")} value={game.platform} />
            <Meta icon={<Palette className="h-4 w-4" />} label={t("game.source")} value={game.source} />
          </dl>

          {game.executablePath && (
            <div className="mt-5 pt-4 border-t border-white/8">
              <p className="text-[11px] uppercase tracking-wide text-dim mb-1.5">
                {t("game.specifyFile")}
              </p>
              <code className="block text-[12.5px] text-muted break-all">{game.executablePath}</code>
            </div>
          )}

          {game.useLauncher && game.launcherPath && (
            <div className="mt-4">
              <p className="text-[11px] uppercase tracking-wide text-dim mb-1.5">
                {t("game.launcherPath")}
              </p>
              <code className="block text-[12.5px] text-muted break-all">{game.launcherPath}</code>
              {game.launcherArgs && (
                <p className="mt-1.5">
                  <span className="text-[11px] uppercase tracking-wide text-dim">
                    {t("game.launcherArgs")}:{" "}
                  </span>
                  <code className="text-[12.5px] text-muted">{game.launcherArgs}</code>
                </p>
              )}
            </div>
          )}
        </section>

        {/* Stats cards */}
        <section className="space-y-4">
          <div className="glass rounded-2xl p-5 space-y-3.5">
            <h2 className="text-[15px] font-semibold text-ink">{t("stats.title")}</h2>
            <StatRow label={t("game.stat.total")} value={formatPlaytime(game.totalPlaytimeSeconds)} />
            <StatRow label={t("game.stat.launches")} value={formatNumber(game.launchCount)} />
            <StatRow label={t("game.stat.avg")} value={formatPlaytime(gameStats?.avgSessionSeconds ?? 0)} />
            <StatRow label={t("game.stat.longest")} value={formatPlaytime(gameStats?.longestSessionSeconds ?? 0)} />
            <StatRow
              label={t("game.stat.last")}
              value={gameStats?.lastSessionAt ? formatDateTime(gameStats.lastSessionAt, locale) : "—"}
            />
            <StatRow label={t("game.stat.week")} value={formatPlaytime(gameStats?.playtimeWeekSeconds ?? 0)} />
            <StatRow label={t("game.stat.month")} value={formatPlaytime(gameStats?.playtimeMonthSeconds ?? 0)} />
          </div>

          <div className="glass rounded-2xl p-5">
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-[14px] font-semibold text-ink flex items-center gap-2">
                <Trophy className="h-4 w-4 text-amber-400" />
                {t("nav.achievements")}
              </h3>
              <span className="text-[12.5px] text-muted tabular-nums">
                {unlocked} / {gameAch.length}
              </span>
            </div>
            <ProgressBar value={percent(unlocked, gameAch.length || 1)} color="var(--game-accent)" />
            <ul className="mt-3 space-y-2">
              {gameAch.slice(0, 5).map((a) => (
                <li
                  key={a.id}
                  className={cx(
                    "flex items-start gap-2.5 text-[12.5px]",
                    a.unlocked ? "text-ink" : "text-dim",
                  )}
                >
                  <span
                    className={cx(
                      "mt-0.5 h-4 w-4 rounded grid place-items-center shrink-0 text-[10px]",
                      a.unlocked ? "bg-amber-400/20 text-amber-300" : "bg-white/8",
                    )}
                  >
                    {a.unlocked ? "✓" : "•"}
                  </span>
                  <span className="min-w-0">
                    <span className="block truncate">{a.title}</span>
                    <span className="block text-[11px] text-dim truncate">{a.description}</span>
                  </span>
                </li>
              ))}
              {gameAch.length === 0 && (
                <li className="text-[12.5px] text-dim">{t("achievements.empty")}</li>
              )}
            </ul>
          </div>
        </section>

        {/* Chart */}
        <section className="lg:col-span-2 glass rounded-2xl p-5">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-[14px] font-semibold text-ink flex items-center gap-2">
              <Clock className="h-4 w-4 text-[var(--color-accent)]" />
              {t("stats.byDay")}
            </h3>
            <div className="flex gap-1.5">
              <Chip tone="accent">{t("game.stat.week")}: {formatPlaytime(gameStats?.playtimeWeekSeconds ?? 0)}</Chip>
            </div>
          </div>
          <PlaytimeChart data={gameStats?.daily ?? []} height={180} />
        </section>

        {/* Sessions */}
        <section className="glass rounded-2xl p-5">
          <h3 className="text-[14px] font-semibold text-ink mb-3">{t("game.sessions")}</h3>
          {sessions.length === 0 ? (
            <p className="text-[13px] text-dim">{t("game.noSessions")}</p>
          ) : (
            <ul className="space-y-2 max-h-64 overflow-y-auto pr-1">
              {sessions.map((s) => (
                <li
                  key={s.id}
                  className="flex items-center justify-between gap-2 px-3 py-2 rounded-lg bg-white/[0.03] border border-white/6 text-[12.5px]"
                >
                  <span className="text-muted truncate">{formatDateTime(s.startedAt, locale)}</span>
                  <span className="text-ink tabular-nums font-medium shrink-0">
                    {formatPlaytime(s.durationSeconds)}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  );
}

function Meta({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div>
      <dt className="flex items-center gap-1.5 text-[11px] uppercase tracking-wide text-dim">
        {icon} {label}
      </dt>
      <dd className="mt-0.5 text-ink truncate">{value}</dd>
    </div>
  );
}

function StatRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between gap-3 text-[13px]">
      <span className="text-muted">{label}</span>
      <span className="text-ink font-medium tabular-nums">{value}</span>
    </div>
  );
}
