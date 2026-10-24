import { Link } from "react-router-dom";
import { ChevronRight, Clock, Trophy, Users } from "lucide-react";
import { useT } from "@/hooks/useT";
import { useStatisticsStore } from "@/stores/statisticsStore";
import { useAchievementsStore } from "@/stores/achievementsStore";
import { fetchFriends } from "@/services/friends";
import { useSettingsStore } from "@/stores/settingsStore";
import { useEffect, useState } from "react";
import { ProgressBar } from "@/components/ui/ProgressBar";
import { GameCoverThumb } from "@/components/game/GameCoverThumb";
import { formatPlaytime, percent, relativeTime } from "@/utils/format";
import { useLocaleCode } from "@/hooks/useT";
import type { Friend } from "@/types";
import { GenreChart } from "@/components/charts/Charts";

export function StatsPanel() {
  const t = useT();
  const locale = useLocaleCode();
  const stats = useStatisticsStore((s) => s.stats);
  const achievements = useAchievementsStore((s) => s.items);
  const demoMode = useSettingsStore((s) => s.demoMode);
  const [friends, setFriends] = useState<Friend[]>([]);

  useEffect(() => {
    void fetchFriends().then(setFriends);
  }, [demoMode]);

  const unlocked = achievements.filter((a) => a.unlocked);
  const lastAch = unlocked[0];
  const gameAchProgress = achievements.filter((a) => a.gameId != null).slice(0, 6);
  // Playtime in the current calendar month. Backend monthly buckets are UTC-based
  // ("%Y-%m-01", see duration_by_period in src-tauri stats.rs), so build the key in UTC too.
  const now = new Date();
  const monthKey = now.toISOString().slice(0, 7);
  const monthly = stats?.monthly ?? [];
  const monthSeconds =
    monthly.find((m) => m.date.startsWith(monthKey))?.seconds ?? 0;
  // Ring scale: playtime this month vs the maximum possible time in it
  // (days in the month × 24 h), computed in UTC like the month key above.
  const daysInMonth = new Date(
    Date.UTC(now.getUTCFullYear(), now.getUTCMonth() + 1, 0),
  ).getUTCDate();
  const maxMonthSeconds = daysInMonth * 24 * 60 * 60;

  const top = stats?.mostPlayed ?? [];

  return (
    <aside className="w-[300px] shrink-0 hidden xl:flex flex-col gap-4" aria-label={t("home.stats")}>
      {/* Total playtime */}
      <section className="glass rounded-2xl p-4">
        <h3 className="text-[13px] font-semibold text-ink mb-3">{t("home.totalPlaytime")}</h3>
        <div className="flex items-center gap-4">
          <div className="relative h-28 w-28 shrink-0" title={t("home.monthProgress")}>
            <svg viewBox="0 0 100 100" className="h-full w-full -rotate-90">
              <circle cx="50" cy="50" r="42" fill="none" stroke="rgba(255,255,255,0.08)" strokeWidth="10" />
              <circle
                cx="50"
                cy="50"
                r="42"
                fill="none"
                stroke="var(--color-accent)"
                strokeWidth="10"
                strokeLinecap="round"
                strokeDasharray={`${Math.min(100, percent(monthSeconds, maxMonthSeconds) * 2.64)} 999`}
              />
            </svg>
            <div className="absolute inset-0 grid place-items-center text-center">
              <div>
                <div className="text-lg font-bold text-ink leading-none">
                  {formatPlaytime(monthSeconds).split(" ").slice(0, 2).join(" ")}
                </div>
                <div className="text-[10px] text-dim mt-1">{t("home.thisMonth")}</div>
              </div>
            </div>
          </div>
          <ul className="flex-1 space-y-1.5">
            {(stats?.byGenre ?? []).slice(0, 5).map((g, i) => {
              const colors = ["#7c5cff", "#5b8cff", "#22d3ee", "#34d399", "#f59e0b"];
              const pct = Math.round(
                (g.count / Math.max(1, stats?.gamesCount ?? 1)) * 100,
              );
              return (
                <li key={g.genre} className="flex items-center gap-2 text-[12px]">
                  <span
                    className="h-2 w-2 rounded-full shrink-0"
                    style={{ background: colors[i % colors.length] }}
                  />
                  <span className="text-muted truncate flex-1">{g.genre}</span>
                  <span className="text-ink tabular-nums">{pct}%</span>
                </li>
              );
            })}
          </ul>
        </div>
        <div className="mt-3 hidden">
          <GenreChart data={stats?.byGenre ?? []} height={80} />
        </div>
      </section>

      {/* Achievements */}
      <section className="glass rounded-2xl p-4">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-[13px] font-semibold text-ink">{t("home.achievements")}</h3>
          <Link to="/achievements" className="text-[11.5px] text-[var(--color-accent)] hover:underline">
            {t("home.viewAll")}
          </Link>
        </div>

        {lastAch ? (
          <div className="flex items-start gap-3">
            <div className="h-11 w-11 rounded-xl bg-amber-500/15 border border-amber-400/25 grid place-items-center shrink-0">
              <Trophy className="h-5 w-5 text-amber-300" />
            </div>
            <div className="min-w-0">
              <p className="text-[13px] font-medium text-ink truncate">{lastAch.title}</p>
              <p className="text-[11.5px] text-muted truncate">
                {t("home.lastUnlocked")}
                {lastAch.gameTitle ? ` · ${lastAch.gameTitle}` : ""}
              </p>
            </div>
          </div>
        ) : (
          <p className="text-[12.5px] text-dim">{t("achievements.empty")}</p>
        )}

        <div className="mt-3.5 pt-3 border-t border-white/8">
          <div className="flex items-center justify-between text-[12px] mb-1.5">
            <span className="text-muted">{t("home.yourProgress")}</span>
            <span className="text-ink font-semibold tabular-nums">
              {unlocked.filter((a) => a.gameId != null).length} /{" "}
              {achievements.filter((a) => a.gameId != null).length || gameAchProgress.length}
            </span>
          </div>
          <ProgressBar
            value={percent(
              unlocked.filter((a) => a.gameId != null).length,
              achievements.filter((a) => a.gameId != null).length || 1,
            )}
            color="#a78bfa"
          />
        </div>
      </section>

      {/* Top playtime */}
      <section className="glass rounded-2xl p-4">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-[13px] font-semibold text-ink">{t("home.topPlaytime")}</h3>
          <Clock className="h-3.5 w-3.5 text-dim" />
        </div>
        <ul className="space-y-2">
          {top.map((g) => (
            <li key={g.gameId} className="flex items-center gap-2.5">
              <GameCoverThumb src={g.coverPath} title={g.title} size={28} />
              <span className="flex-1 text-[12.5px] text-ink truncate">{g.title}</span>
              <span className="text-[11.5px] text-muted tabular-nums shrink-0">
                {formatPlaytime(g.seconds)}
              </span>
            </li>
          ))}
          {top.length === 0 && <p className="text-[12.5px] text-dim">{t("common.nothing")}</p>}
        </ul>
      </section>

      {/* Friends */}
      <section className="glass rounded-2xl p-4">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-[13px] font-semibold text-ink">{t("home.friendsActivity")}</h3>
          <Users className="h-3.5 w-3.5 text-dim" />
        </div>
        <ul className="space-y-2.5">
          {friends.slice(0, 4).map((f) => (
            <li key={f.id} className="flex items-start gap-2.5">
              <span
                className={`mt-1.5 h-2 w-2 rounded-full shrink-0 ${
                  f.online ? "bg-emerald-400 shadow-[0_0_6px_#34d399]" : "bg-white/25"
                }`}
              />
              <div className="min-w-0 flex-1">
                <p className="text-[12.5px] font-medium text-ink truncate">{f.name}</p>
                <p className="text-[11.5px] text-muted truncate">
                  {f.online && f.currentGame
                    ? `${f.status}`
                    : relativeTime(f.lastSeen, locale)}
                </p>
              </div>
            </li>
          ))}
        </ul>
        <Link
          to="/friends"
          className="mt-3 flex items-center justify-center gap-1 text-[12px] text-[var(--color-accent)] hover:underline"
        >
          {t("home.viewAll")} <ChevronRight className="h-3.5 w-3.5" />
        </Link>
      </section>
    </aside>
  );
}
