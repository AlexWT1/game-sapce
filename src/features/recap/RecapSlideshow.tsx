import { useCallback, useEffect, useRef, useState } from "react";
import { AnimatePresence } from "framer-motion";
import { ChevronLeft, ChevronRight, Sparkles, X } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { SlideWrapper } from "./components/SlideWrapper";
import { IntroSlide } from "./components/slides/IntroSlide";
import { TotalTimeSlide } from "./components/slides/TotalTimeSlide";
import { TopGamesSlide } from "./components/slides/TopGamesSlide";
import { FavoriteGenreSlide } from "./components/slides/FavoriteGenreSlide";
import { SessionsSlide } from "./components/slides/SessionsSlide";
import { AchievementsSlide } from "./components/slides/AchievementsSlide";
import { HeatmapSlide } from "./components/slides/HeatmapSlide";
import { SummarySlide } from "./components/slides/SummarySlide";
import type { RecapData, RecapPeriod } from "./hooks/useRecapData";
import { useLocale, useT } from "@/hooks/useT";
import { cx } from "@/utils/format";

interface RecapSlideshowProps {
  data: RecapData;
  period: RecapPeriod;
  onPeriodChange: (period: RecapPeriod) => void;
  /** Custom close target (used by the auto-show overlay); defaults to navigating to /statistics. */
  onClose?: () => void;
}

export function RecapSlideshow({ data, period, onPeriodChange, onClose }: RecapSlideshowProps) {
  const t = useT();
  const locale = useLocale();
  const navigate = useNavigate();
  const [index, setIndex] = useState(0);
  const wheelLock = useRef(0);
  const total = 8;

  const next = useCallback(() => setIndex((i) => Math.min(total - 1, i + 1)), []);
  const prev = useCallback(() => setIndex((i) => Math.max(0, i - 1)), []);
  const close = useCallback(() => {
    if (onClose) onClose();
    else navigate("/statistics");
  }, [onClose, navigate]);

  // Keyboard navigation: ← / → / Escape.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "ArrowRight") {
        e.preventDefault();
        next();
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        prev();
      } else if (e.key === "Escape") {
        close();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [next, prev, close]);

  // Wheel navigation with a small lock to avoid skipping 3 slides per gesture.
  const onWheel = (e: React.WheelEvent) => {
    const now = Date.now();
    if (now - wheelLock.current < 650) return;
    if (Math.abs(e.deltaY) < 24) return;
    const step = e.deltaY > 0 ? 1 : -1;
    setIndex((i) => Math.min(total - 1, Math.max(0, i + step)));
  };

  const advanceOnClick = (e: React.MouseEvent) => {
    // Don't steal clicks from buttons/links and explicitly opted-out zones.
    const el = e.target as HTMLElement;
    if (el.closest("button, a, [data-no-advance]")) return;
    next();
  };

  // Human label of the covered window, e.g. «октябрь 2026» / «2026».
  const now = new Date();
  const monthName = now.toLocaleString(locale === "ru" ? "ru-RU" : "en-US", { month: "long" });
  const periodLabel =
    period === "year" ? String(now.getFullYear()) : `${monthName} ${now.getFullYear()}`;

  const slides = [
    <IntroSlide period={period} onPick={onPeriodChange} onStart={next} />,
    <TotalTimeSlide totalSeconds={data.totalSeconds} />,
    <TopGamesSlide games={data.topGames} />,
    <FavoriteGenreSlide data={data} />,
    <SessionsSlide data={data} />,
    <AchievementsSlide data={data} />,
    <HeatmapSlide data={data} periodLabel={periodLabel} />,
    <SummarySlide data={data} onClose={close} />,
  ];

  return (
    <div
      className="fixed inset-0 z-40 bg-[var(--color-bg)] flex flex-col overflow-hidden recap-backdrop"
      onWheel={onWheel}
      onClick={advanceOnClick}
    >
      {/* Header */}
      <div className="pointer-events-none absolute top-0 inset-x-0 z-20 flex items-center justify-between px-5 py-4">
        <div className="flex items-center gap-2 text-[12.5px] text-dim">
          <Sparkles className="h-4 w-4 text-[var(--color-accent)]" />
          {t("recap.title")}
          <span className="text-muted">· {periodLabel}</span>
        </div>
        <button
          onClick={close}
          className="pointer-events-auto grid place-items-center h-9 w-9 rounded-lg text-muted hover:text-ink hover:bg-white/8 transition"
          aria-label={t("recap.close")}
        >
          <X className="h-[18px] w-[18px]" />
        </button>
      </div>

      {/* Slide progress segments */}
      <div className="pointer-events-none absolute top-14 inset-x-5 z-20 flex gap-1.5">
        {slides.map((_, i) => (
          <div
            key={i}
            className={cx(
              "h-[3px] flex-1 rounded-full transition-colors duration-300",
              i < index ? "bg-[var(--color-accent)]/70" : i === index ? "bg-[var(--color-accent)]" : "bg-white/10",
            )}
          />
        ))}
      </div>

      {/* Slides */}
      <div className="relative flex-1 min-h-0">
        <AnimatePresence mode="wait">
          <SlideWrapper key={index} index={index}>
            {slides[index]}
          </SlideWrapper>
        </AnimatePresence>
      </div>

      {/* Footer nav */}
      <div className="pointer-events-none absolute bottom-0 inset-x-0 z-20 flex items-center justify-center gap-5 pb-6">
        <button
          onClick={(e) => {
            e.stopPropagation();
            prev();
          }}
          disabled={index === 0}
          className="pointer-events-auto grid place-items-center h-10 w-10 rounded-xl glass text-muted hover:text-ink transition disabled:opacity-30 disabled:pointer-events-none"
          aria-label="Previous"
        >
          <ChevronLeft className="h-5 w-5" />
        </button>

        <span className="text-[11.5px] text-dim tabular-nums">
          {index + 1} / {total}
        </span>

        <button
          onClick={(e) => {
            e.stopPropagation();
            next();
          }}
          disabled={index === total - 1}
          className="pointer-events-auto grid place-items-center h-10 w-10 rounded-xl glass text-muted hover:text-ink transition disabled:opacity-30 disabled:pointer-events-none"
          aria-label={t("recap.next")}
        >
          <ChevronRight className="h-5 w-5" />
        </button>
      </div>
    </div>
  );
}
