import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { Sparkles } from "lucide-react";
import { useLocale, useT } from "@/hooks/useT";
import { Button } from "@/components/ui/Button";
import { cx } from "@/utils/format";
import type { RecapPeriod } from "../../hooks/useRecapData";

interface IntroSlideProps {
  period: RecapPeriod;
  onPick: (period: RecapPeriod) => void;
  onStart: () => void;
}

export function IntroSlide({ period, onPick, onStart }: IntroSlideProps) {
  const t = useT();
  const locale = useLocale();
  const [typed, setTyped] = useState("");

  const now = new Date();
  const monthName = now.toLocaleString(locale === "ru" ? "ru-RU" : "en-US", { month: "long" });
  const title =
    period === "year"
      ? `${t("recap.year")} ${now.getFullYear()}`
      : `${t("recap.month")} — ${monthName}`;

  useEffect(() => {
    let i = 0;
    setTyped("");
    const timer = window.setInterval(() => {
      setTyped(title.slice(0, i + 1));
      i += 1;
      if (i >= title.length) window.clearInterval(timer);
    }, 55);
    return () => window.clearInterval(timer);
  }, [title]);

  return (
    <div className="relative flex flex-col items-center justify-center text-center max-w-xl">
      {/* Ambient glow */}
      <motion.div
        className="absolute w-[520px] h-[520px] rounded-full pointer-events-none"
        style={{
          background: "radial-gradient(circle, rgba(124,92,255,0.16), transparent 65%)",
          filter: "blur(40px)",
        }}
        animate={{ scale: [1, 1.12, 1], opacity: [0.5, 0.8, 0.5] }}
        transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
      />

      <motion.div
        initial={{ opacity: 0, scale: 0.7 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.6 }}
        className="mb-6 grid place-items-center h-20 w-20 rounded-3xl bg-[var(--color-accent)]/15 shadow-[0_0_60px_rgba(124,92,255,0.35)]"
      >
        <Sparkles className="h-10 w-10 text-[var(--color-accent)]" />
      </motion.div>

      <h1 className="text-4xl md:text-5xl font-bold text-gradient min-h-[1.2em] mb-4 tracking-tight">
        {typed}
        <span className="animate-pulse">|</span>
      </h1>

      <motion.p
        className="text-base text-muted mb-6 max-w-sm leading-relaxed"
        initial={{ opacity: 0, y: 16 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.9 }}
      >
        {t("recap.heroHint")}
      </motion.p>

      {/* Period picker */}
      <motion.div
        data-no-advance
        className="flex items-center gap-1 p-1 rounded-xl bg-white/5 border border-white/10 mb-8"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.2 }}
        onClick={(e) => e.stopPropagation()}
      >
        {(["month", "year"] as const).map((p) => (
          <button
            key={p}
            onClick={() => onPick(p)}
            className={cx(
              "px-4 h-8 rounded-lg text-[13px] transition",
              period === p
                ? "bg-[var(--color-accent)] text-white font-medium"
                : "text-muted hover:text-ink",
            )}
          >
            {p === "month" ? t("recap.pickMonth") : t("recap.pickYear")}
          </button>
        ))}
      </motion.div>

      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 1.5 }}
      >
        <Button size="lg" onClick={onStart}>
          {t("recap.start")} →
        </Button>
      </motion.div>
    </div>
  );
}
