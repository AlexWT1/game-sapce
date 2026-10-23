import { useEffect, useMemo } from "react";
import { useStatisticsStore, statsRangeKey } from "@/stores/statisticsStore";
import { useAchievementsStore } from "@/stores/achievementsStore";
import type { Achievement, GenreCount, PlaytimeByDay, RankedPlaytime } from "@/types";

export type RecapPeriod = "month" | "year";

export interface RecapData {
  period: RecapPeriod;
  /** YYYY-MM-DD window the aggregates cover (`to` exclusive). */
  from: string;
  to: string;
  totalSeconds: number;
  topGames: RankedPlaytime[];
  topGenre: GenreCount | null;
  genres: GenreCount[];
  sessionsCount: number;
  avgSessionSeconds: number;
  longestSessionSeconds: number;
  /** Playful guess: the most-played game is the likeliest marathon king. */
  longestSessionGame: string | null;
  achievementsUnlocked: number;
  achievementsTotal: number;
  rarestAchievement: Achievement | null;
  heatmap: PlaytimeByDay[];
  activeDays: number;
  longestStreak: number;
  gamesCount: number;
  favoriteCount: number;
  installedCount: number;
}

const RARITY_ORDER: Record<string, number> = {
  Common: 0,
  Rare: 1,
  Epic: 2,
  Legendary: 3,
};

/** Local-time range for the selected period (`to` exclusive), same dates the backend expects. */
export function recapRange(period: RecapPeriod): { from: string; to: string } {
  const now = new Date();
  const fmt = (d: Date) =>
    `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  if (period === "year") {
    const year = now.getFullYear();
    return { from: `${year}-01-01`, to: `${year + 1}-01-01` };
  }
  const first = new Date(now.getFullYear(), now.getMonth(), 1);
  const next = new Date(now.getFullYear(), now.getMonth() + 1, 1);
  return { from: fmt(first), to: fmt(next) };
}

/**
 * Aggregates statistics for a concrete period. Session-based aggregates come
 * from the ranged `get_statistics` (backend filters by `started_at` window),
 * so Steam-seeded lifetime playtime never leaks into a month/year recap.
 */
export function useRecapData(period: RecapPeriod): { data: RecapData | null; loading: boolean } {
  const { from, to } = useMemo(() => recapRange(period), [period]);
  const range = statsRangeKey(from, to);

  const stats = useStatisticsStore((s) => s.stats);
  const statsRange = useStatisticsStore((s) => s.range);
  const statsLoading = useStatisticsStore((s) => s.loading);
  const loadRange = useStatisticsStore((s) => s.loadRange);
  const achievements = useAchievementsStore((s) => s.items);
  const achLoading = useAchievementsStore((s) => s.loading);
  const loadAchievements = useAchievementsStore((s) => s.load);

  useEffect(() => {
    void loadAchievements();
  }, [loadAchievements]);

  // Refetch whenever the cached window doesn't match the selected period
  // (covers the initial load, period switches and demo-mode force reloads).
  useEffect(() => {
    if (statsRange !== range) void loadRange(from, to);
  }, [statsRange, range, from, to, loadRange]);

  const data = useMemo<RecapData | null>(() => {
    if (!stats || statsRange !== range) return null;

    const topGames = stats.mostPlayed.slice(0, 5);
    const topGenre = stats.byGenre[0] ?? null;
    const activeDays = stats.daily.filter((d) => d.seconds > 0).length;
    const longestStreak = calcLongestStreak(stats.daily);

    const rangedUnlocked = achievements.filter(
      (a) => a.unlocked && a.unlockedAt != null && a.unlockedAt >= from && a.unlockedAt < to,
    );
    const rarestAchievement = rangedUnlocked.reduce<Achievement | null>((rarest, ach) => {
      const achRank = RARITY_ORDER[ach.rarity] ?? 0;
      const rareRank = rarest ? RARITY_ORDER[rarest.rarity] ?? 0 : -1;
      return achRank >= rareRank ? ach : rarest;
    }, null);

    return {
      period,
      from,
      to,
      totalSeconds: stats.totalPlaytimeSeconds,
      topGames,
      topGenre,
      genres: stats.byGenre,
      sessionsCount: stats.sessionsCount,
      avgSessionSeconds: stats.avgSessionSeconds,
      longestSessionSeconds: stats.longestSessionSeconds,
      longestSessionGame: topGames[0]?.title ?? null,
      achievementsUnlocked: stats.achievementsUnlocked,
      achievementsTotal: stats.achievementsTotal,
      rarestAchievement,
      heatmap: stats.daily,
      activeDays,
      longestStreak,
      gamesCount: stats.gamesCount,
      favoriteCount: stats.favoriteCount,
      installedCount: stats.installedCount,
    };
  }, [stats, statsRange, range, from, to, period, achievements]);

  return { data, loading: statsLoading || achLoading };
}

function calcLongestStreak(days: PlaytimeByDay[]): number {
  let maxStreak = 0;
  let currentStreak = 0;
  for (const day of days) {
    if (day.seconds > 0) {
      currentStreak += 1;
      maxStreak = Math.max(maxStreak, currentStreak);
    } else {
      currentStreak = 0;
    }
  }
  return maxStreak;
}
