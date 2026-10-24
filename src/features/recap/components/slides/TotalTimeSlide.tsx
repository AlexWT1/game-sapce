import { Clock } from "lucide-react";
import { motion } from "framer-motion";
import { CountUp } from "../CountUp";
import { getPlaytimeFact } from "../../utils/funFacts";
import { useLocale, useT } from "@/hooks/useT";

interface TotalTimeSlideProps {
  totalSeconds: number;
}

export function TotalTimeSlide({ totalSeconds }: TotalTimeSlideProps) {
  const t = useT();
  const locale = useLocale();
  const hours = totalSeconds / 3600;
  const fact = getPlaytimeFact(hours, locale);
  const showHours = totalSeconds >= 3600;
  const minutes = Math.round(totalSeconds / 60);

  return (
    <div className="relative flex flex-col items-center justify-center text-center max-w-lg">
      <motion.div
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.5 }}
        className="mb-7 grid place-items-center h-16 w-16 rounded-2xl bg-[var(--color-accent)]/15"
      >
        <Clock className="h-9 w-9 text-[var(--color-accent)]" />
      </motion.div>

      <div className="flex items-baseline gap-3 tabular-nums">
        <CountUp
          to={showHours ? hours : minutes}
          duration={1.6}
          formatter={(n) => Math.round(n).toLocaleString()}
          className="text-6xl md:text-7xl font-bold text-gradient"
        />
        <span className="text-xl text-muted">
          {showHours ? t("recap.hours") : t("recap.minutes")}
        </span>
      </div>

      <p className="text-[12.5px] text-dim mt-2 mb-8 uppercase tracking-[0.14em]">
        {t("recap.totalTime")}
      </p>

      <motion.div
        className="glass rounded-2xl p-5 max-w-sm"
        initial={{ opacity: 0, y: 24 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.55 }}
      >
        <p className="text-[15px] text-muted leading-relaxed">{fact}</p>
      </motion.div>
    </div>
  );
}
