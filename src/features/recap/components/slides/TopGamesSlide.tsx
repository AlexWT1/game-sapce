import { motion } from "framer-motion";
import { Gamepad2 } from "lucide-react";
import type { RankedPlaytime } from "@/types";
import { formatPlaytime } from "@/utils/format";
import { ProgressBar } from "@/components/ui/ProgressBar";
import { StaggerContainer, itemVariants } from "../SlideWrapper";
import { useT } from "@/hooks/useT";

interface TopGamesSlideProps {
  games: RankedPlaytime[];
}

export function TopGamesSlide({ games }: TopGamesSlideProps) {
  const t = useT();
  const maxSeconds = games[0]?.seconds ?? 1;

  if (games.length === 0) {
    return (
      <div className="flex flex-col items-center text-center">
        <Gamepad2 className="h-12 w-12 text-dim mb-4" />
        <p className="text-muted">{t("recap.topGamesEmpty")}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center w-full max-w-lg">
      <div className="flex items-center gap-2 mb-8">
        <Gamepad2 className="h-6 w-6 text-[var(--color-accent)]" />
        <h2 className="text-2xl font-bold text-gradient">{t("recap.topGames")}</h2>
      </div>

      <StaggerContainer className="w-full space-y-3" staggerChildren={0.12}>
        {games.map((game, index) => (
          <motion.div
            key={game.gameId}
            variants={itemVariants}
            className="flex items-center gap-4 p-4 glass rounded-2xl hover:bg-white/[0.06] transition-colors"
          >
            {/* Rank */}
            <span className={`
              w-8 h-8 rounded-xl flex items-center justify-center text-sm font-bold shrink-0
              ${index === 0 ? "bg-amber-500/20 text-amber-400" :
                index === 1 ? "bg-slate-400/20 text-slate-300" :
                index === 2 ? "bg-amber-700/20 text-amber-600" :
                "bg-white/10 text-dim"}
            `}>
              {index + 1}
            </span>

            {/* Cover or placeholder */}
            <div className="w-12 h-12 rounded-lg bg-white/10 overflow-hidden shrink-0">
              {game.coverPath ? (
                <img src={game.coverPath} alt="" className="w-full h-full object-cover" />
              ) : (
                <div className="w-full h-full grid place-items-center text-lg font-bold text-muted">
                  {game.title[0]}
                </div>
              )}
            </div>

            {/* Info */}
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-ink truncate">{game.title}</p>
              <div className="flex items-center gap-2 mt-1">
                <div className="flex-1">
                  <ProgressBar
                    value={game.seconds}
                    max={maxSeconds}
                    height={4}
                    color="var(--color-accent)"
                    className="max-w-[120px]"
                  />
                </div>
                <span className="text-xs text-dim tabular-nums whitespace-nowrap">
                  {formatPlaytime(game.seconds)}
                </span>
              </div>
            </div>
          </motion.div>
        ))}
      </StaggerContainer>
    </div>
  );
}