import { motion } from "framer-motion";
import { Trophy } from "lucide-react";
import { CountUp } from "../CountUp";
import { ProgressBar } from "@/components/ui/ProgressBar";
import { useT } from "@/hooks/useT";
import type { Rarity } from "@/types";
import type { RecapData } from "../../hooks/useRecapData";

interface AchievementsSlideProps {
  data: RecapData;
}

export function AchievementsSlide({ data }: AchievementsSlideProps) {
  const t = useT();
  const { achievementsUnlocked, achievementsTotal, rarestAchievement } = data;
  const pct = achievementsTotal > 0 ? Math.round((achievementsUnlocked / achievementsTotal) * 100) : 0;

  return (
    <div className="relative flex flex-col items-center justify-center text-center max-w-lg">
      {/* Golden glow trophy */}
      <div className="relative grid place-items-center mb-7">
        <motion.div
          className="absolute h-28 w-28 rounded-full"
          style={{ background: "radial-gradient(circle, rgba(251,191,36,0.35), transparent 70%)" }}
          animate={{ scale: [1, 1.25, 1], opacity: [0.6, 1, 0.6] }}
          transition={{ duration: 2.4, repeat: Infinity, ease: "easeInOut" }}
        />
        <div className="relative grid place-items-center h-20 w-20 rounded-3xl bg-amber-400/15 border border-amber-400/30">
          <Trophy className="h-10 w-10 text-amber-400" />
        </div>
      </div>

      <h2 className="text-2xl font-bold text-gradient mb-2">{t("recap.achievements")}</h2>

      <div className="flex items-baseline gap-3 tabular-nums mb-4">
        <CountUp
          to={achievementsUnlocked}
          duration={1.4}
          className="text-5xl md:text-6xl font-bold text-amber-300"
        />
        <span className="text-xl text-dim">{t("recap.of")}</span>
        <span className="text-3xl font-semibold text-muted tabular-nums">{achievementsTotal}</span>
      </div>

      <motion.div
        className="w-full max-w-xs mb-3"
        initial={{ opacity: 0, scaleX: 0.6 }}
        animate={{ opacity: 1, scaleX: 1 }}
        transition={{ delay: 0.4 }}
      >
        <ProgressBar value={pct} max={100} color="#fbbf24" height={8} />
        <div className="text-[11.5px] text-dim mt-1.5">
          {t("recap.completion")}: {pct}%
        </div>
      </motion.div>

      {rarestAchievement ? (
        <motion.div
          className="glass rounded-2xl p-5 mt-6 max-w-sm w-full border-amber-400/25"
          initial={{ opacity: 0, y: 24, scale: 0.95 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          transition={{ delay: 0.65, type: "spring" }}
        >
          <div className="text-[10.5px] uppercase tracking-[0.16em] text-amber-300/80 mb-1.5">
            {t("recap.rarestAch")}
          </div>
          <div className="text-[15px] font-bold text-ink mb-1">
            {rarestAchievement.title}
          </div>
          <div className="text-[12.5px] text-muted leading-relaxed">
            {rarestAchievement.description}
          </div>
          <div className="flex items-center justify-center gap-2 mt-3 text-[11px] text-dim">
            <span className="px-2 py-0.5 rounded-full bg-amber-400/15 text-amber-300">
              {t(`achievements.rarity.${rarestAchievement.rarity as Rarity}` as never)}
            </span>
            {rarestAchievement.gameTitle && <span>{rarestAchievement.gameTitle}</span>}
          </div>
        </motion.div>
      ) : (
        <motion.p
          className="mt-6 text-[13px] text-dim"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.65 }}
        >
          {t("recap.noAch")}
        </motion.p>
      )}
    </div>
  );
}
