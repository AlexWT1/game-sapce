import { useState } from "react";
import { motion } from "framer-motion";
import { Clock, MoreHorizontal, Play, Trophy } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { ProgressBar } from "@/components/ui/ProgressBar";
import { ContextMenu } from "@/components/ui/ContextMenu";
import { buildGameMenu } from "@/components/game/GameCard";
import { LaunchButton } from "@/components/game/LaunchButton";
import { useT } from "@/hooks/useT";
import { useGamesStore } from "@/stores/gamesStore";
import { useSessionStore } from "@/stores/sessionStore";
import { useAchievementsStore } from "@/stores/achievementsStore";
import { formatHMS, formatPlaytime, percent } from "@/utils/format";
import type { Game } from "@/types";

export function HeroSection() {
  const t = useT();
  const navigate = useNavigate();
  const games = useGamesStore((s) => s.games);
  const loading = useGamesStore((s) => s.loading);
  const achievements = useAchievementsStore((s) => s.items);
  const running = useSessionStore((s) => s.running);
  const live = useSessionStore((s) => s.liveSeconds);

  const runningIds = new Set(running.map((r) => r.gameId));
  const [brokenCover, setBrokenCover] = useState<string | null>(null);
  const hero: Game | undefined =
    games.find((g) => runningIds.has(g.id)) ??
    games
      .filter((g) => !g.isHidden)
      .slice()
      .sort((a, b) => (b.lastPlayedAt ?? "").localeCompare(a.lastPlayedAt ?? ""))[0];

  if (loading && !hero) {
    return <div className="skeleton h-[300px] w-full rounded-3xl" />;
  }
  if (!hero) return null;

  const gameAch = achievements.filter((a) => a.gameId === hero.id);
  const unlocked = gameAch.filter((a) => a.unlocked).length;
  const isRunning = runningIds.has(hero.id);
  const liveSec = live[hero.id] ?? 0;

  return (
    <ContextMenu items={buildGameMenu(hero, t)} className="w-full">
      <motion.section
        initial={{ opacity: 1, y: 0 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.35, ease: [0.2, 0.7, 0.3, 1] }}
        className="relative w-full h-[300px] rounded-3xl overflow-hidden border border-white/10 vignette"
        onClick={() => navigate(`/game/${hero.id}`)}
        role="button"
        tabIndex={0}
      >
        {hero.coverPath && brokenCover !== hero.coverPath && (
          <img
            src={hero.coverPath}
            alt=""
            className="absolute inset-0 h-full w-full object-cover object-[center_20%] scale-105"
            onError={() => setBrokenCover(hero.coverPath)}
          />
        )}
        <div className="absolute inset-0 bg-gradient-to-r from-[#070912] via-[#070912]/75 to-transparent" />
        <div className="absolute inset-0 bg-gradient-to-t from-[#070912] via-transparent to-transparent" />

        {/* subtle particles */}
        <div className="absolute inset-0 opacity-40 pointer-events-none">
          {Array.from({ length: 14 }).map((_, i) => (
            <motion.span
              key={i}
              className="absolute h-1 w-1 rounded-full bg-white/50"
              style={{
                left: `${(i * 37) % 100}%`,
                top: `${(i * 53) % 100}%`,
              }}
              animate={{ y: [0, -12, 0], opacity: [0.2, 0.7, 0.2] }}
              transition={{ duration: 3 + (i % 4), repeat: Infinity, delay: i * 0.2 }}
            />
          ))}
        </div>

        <div className="relative h-full flex flex-col justify-between p-7 z-10">
          <div>
            <div className="flex items-center gap-3 mb-3">
              <span
                className={`inline-flex items-center gap-1.5 text-[12px] font-medium px-2.5 py-1 rounded-full border ${
                  isRunning
                    ? "text-emerald-300 bg-emerald-500/10 border-emerald-400/30"
                    : "text-muted bg-white/6 border-white/10"
                }`}
              >
                {isRunning ? (
                  <>
                    <span className="live-dot" />
                    {t("home.statusPlaying")}
                  </>
                ) : (
                  t("home.statusIdle")
                )}
              </span>
              <span className="text-[12px] text-muted flex items-center gap-1">
                <Clock className="h-3.5 w-3.5" />
                {isRunning ? formatHMS(liveSec) : formatPlaytime(hero.totalPlaytimeSeconds)}
              </span>
            </div>

            <h1 className="text-[34px] leading-tight font-extrabold tracking-tight text-gradient uppercase">
              {hero.title}
            </h1>

            <p className="mt-2.5 text-[13.5px] text-muted/90 leading-relaxed line-clamp-3 max-w-[480px]">
              {hero.description || `${hero.developer} · ${hero.genre}`}
            </p>
          </div>

          <div className="flex items-end gap-4">
            <LaunchButton game={hero} size="lg" />

            <button
              onClick={(e) => {
                e.stopPropagation();
                navigate(`/game/${hero.id}`);
              }}
              className="h-12 w-11 grid place-items-center rounded-xl bg-white/10 border border-white/10 text-ink hover:bg-white/15 transition"
              aria-label={t("game.more")}
            >
              <MoreHorizontal className="h-5 w-5" />
            </button>

            <div className="ml-auto hidden sm:block glass rounded-2xl px-4 py-3 min-w-[170px]">
              <div className="flex items-center justify-between text-[12px] mb-1.5">
                <span className="text-muted flex items-center gap-1.5">
                  <Trophy className="h-3.5 w-3.5 text-amber-400" />
                  {t("home.achievements")}
                </span>
                <span className="text-ink font-semibold tabular-nums">
                  {unlocked} / {gameAch.length}
                </span>
              </div>
              <ProgressBar
                value={percent(unlocked, gameAch.length || 1)}
                color="var(--game-accent, #a78bfa)"
                height={5}
              />
              <div className="mt-3 pt-2.5 border-t border-white/8 flex flex-col gap-1.5 text-[12px]">
                <span className="text-muted flex items-center gap-1.5">
                  <Clock className="h-3.5 w-3.5" />
                  {t("home.playTime")}
                </span>
                <span className="text-ink font-semibold tabular-nums ml-5">
                  {formatPlaytime(hero.totalPlaytimeSeconds)}
                </span>
              </div>
            </div>
          </div>
        </div>
      </motion.section>
    </ContextMenu>
  );
}

export function _unusedPlayIcon() {
  return <Play className="h-4 w-4" />;
}
