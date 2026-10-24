import { motion } from "framer-motion";
import { Flame } from "lucide-react";
import { ActivityHeatmap } from "../ActivityHeatmap";
import { useT } from "@/hooks/useT";
import type { RecapData } from "../../hooks/useRecapData";

interface HeatmapSlideProps {
  data: RecapData;
  periodLabel: string;
}

export function HeatmapSlide({ data, periodLabel }: HeatmapSlideProps) {
  const t = useT();
  const hasActivity = data.activeDays > 0;

  return (
    <div
      data-no-advance
      className="relative flex flex-col items-center justify-center w-full max-w-2xl"
      onClick={(e) => e.stopPropagation()}
    >
      <div className="flex items-center gap-3 mb-8">
        <Flame className="h-6 w-6 text-[var(--color-accent)]" />
        <h2 className="text-2xl font-bold text-gradient">{t("recap.activity")}</h2>
      </div>

      {hasActivity ? (
        <>
          <motion.div
            className="glass rounded-2xl p-5 w-full"
            initial={{ opacity: 0, y: 24 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
          >
            <ActivityHeatmap data={data.heatmap} />
            <div className="text-[10.5px] text-dim mt-3">
              {periodLabel} · {t("recap.heatCaptionDays")}
            </div>
          </motion.div>

          <div className="grid grid-cols-2 gap-3 mt-5 w-full max-w-md">
            <motion.div
              className="glass rounded-2xl p-4 text-center"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.5 }}
            >
              <div className="text-2xl font-bold text-gradient tabular-nums">{data.activeDays}</div>
              <div className="text-[11px] uppercase tracking-[0.12em] text-dim mt-1">
                {t("recap.activeDays")}
              </div>
            </motion.div>
            <motion.div
              className="glass rounded-2xl p-4 text-center"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.62 }}
            >
              <div className="text-2xl font-bold text-gradient tabular-nums">{data.longestStreak}</div>
              <div className="text-[11px] uppercase tracking-[0.12em] text-dim mt-1">
                {t("recap.longestStreak")}
              </div>
            </motion.div>
          </div>
        </>
      ) : (
        <motion.p
          className="text-muted"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.3 }}
        >
          {t("recap.heatEmpty")}
        </motion.p>
      )}
    </div>
  );
}
