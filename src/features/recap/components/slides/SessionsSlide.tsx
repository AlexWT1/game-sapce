import { motion } from "framer-motion";
import { Timer } from "lucide-react";
import { CountUp } from "../CountUp";
import { formatPlaytime } from "@/utils/format";
import { useT } from "@/hooks/useT";
import type { RecapData } from "../../hooks/useRecapData";

interface SessionsSlideProps {
  data: RecapData;
}

export function SessionsSlide({ data }: SessionsSlideProps) {
  const t = useT();

  return (
    <div className="relative flex flex-col items-center justify-center text-center max-w-lg">
      <div className="relative mb-8 grid place-items-center">
        <motion.div
          className="absolute h-24 w-24 rounded-full bg-[var(--color-accent)]/20"
          animate={{ scale: [1, 1.35, 1], opacity: [0.5, 0, 0.5] }}
          transition={{ duration: 2, repeat: Infinity, ease: "easeOut" }}
        />
        <div className="relative grid place-items-center h-16 w-16 rounded-2xl bg-[var(--color-accent)]/15">
          <Timer className="h-9 w-9 text-[var(--color-accent)]" />
        </div>
      </div>

      <h2 className="text-2xl font-bold text-gradient mb-9">{t("recap.sessions")}</h2>

      <CountUp
        to={data.sessionsCount}
        duration={1.4}
        formatter={(n) => Math.round(n).toLocaleString()}
        className="text-6xl md:text-7xl font-bold text-gradient tabular-nums"
      />

      <div className="grid grid-cols-2 gap-3 mt-9 w-full max-w-md">
        <motion.div
          className="glass rounded-2xl p-4"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.35 }}
        >
          <div className="text-[11px] uppercase tracking-[0.12em] text-dim mb-1.5">
            {t("recap.longestSession")}
          </div>
          <div className="text-xl font-bold text-ink tabular-nums">
            {formatPlaytime(data.longestSessionSeconds)}
          </div>
        </motion.div>
        <motion.div
          className="glass rounded-2xl p-4"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.5 }}
        >
          <div className="text-[11px] uppercase tracking-[0.12em] text-dim mb-1.5">
            {t("recap.avgSession")}
          </div>
          <div className="text-xl font-bold text-ink tabular-nums">
            {formatPlaytime(data.avgSessionSeconds)}
          </div>
        </motion.div>
      </div>

      {data.longestSessionGame && (
        <motion.p
          className="mt-7 text-[13px] text-muted"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.75 }}
        >
          {t("recap.marathonKing")} — <span className="text-ink font-semibold">{data.longestSessionGame}</span>
          {" "}
          <span className="text-dim">({t("recap.marathonKingGuess")})</span>
        </motion.p>
      )}
    </div>
  );
}
