import { useMemo } from "react";
import type { PlaytimeByDay } from "@/types";
import { useLocale } from "@/hooks/useT";

interface ActivityHeatmapProps {
  data: PlaytimeByDay[];
  maxValue?: number;
}

interface HeatCell {
  date: string;
  seconds: number;
  intensity: number;
}

export function ActivityHeatmap({ data, maxValue }: ActivityHeatmapProps) {
  const locale = useLocale();
  const ru = locale === "ru";

  const { weeks, monthLabels } = useMemo(() => {
    const maxVal = maxValue ?? Math.max(...data.map((d) => d.seconds), 1);

    // Group by weeks (7 days each).
    const weeksArr: HeatCell[][] = [];
    let currentWeek: HeatCell[] = [];

    // Pad start to align the first column with the weekday grid.
    const firstDate = data[0] ? new Date(data[0].date) : new Date();
    const startPadding = Number.isNaN(firstDate.getTime()) ? 0 : firstDate.getDay();
    for (let i = 0; i < startPadding; i++) {
      currentWeek.push({ date: "", seconds: 0, intensity: 0 });
    }

    for (const day of data) {
      currentWeek.push({
        date: day.date,
        seconds: day.seconds,
        intensity: maxVal > 0 ? Math.min(1, day.seconds / maxVal) : 0,
      });
      if (currentWeek.length === 7) {
        weeksArr.push(currentWeek);
        currentWeek = [];
      }
    }
    if (currentWeek.length > 0) {
      while (currentWeek.length < 7) {
        currentWeek.push({ date: "", seconds: 0, intensity: 0 });
      }
      weeksArr.push(currentWeek);
    }

    // Month labels over the columns.
    const monthLocale = locale === "ru" ? "ru-RU" : "en-US";
    const labels: Array<{ month: string; weekIndex: number }> = [];
    let lastMonth = "";
    weeksArr.forEach((week, i) => {
      const firstValidDay = week.find((d) => d.date);
      if (firstValidDay) {
        const d = new Date(firstValidDay.date);
        if (Number.isNaN(d.getTime())) return;
        const month = d.toLocaleString(monthLocale, { month: "short" });
        if (month !== lastMonth) {
          labels.push({ month, weekIndex: i });
          lastMonth = month;
        }
      }
    });

    return { weeks: weeksArr, monthLabels: labels };
  }, [data, maxValue, locale]);

  const dayLabels = ru ? ["", "Пн", "", "Ср", "", "Пт", ""] : ["", "Mo", "", "We", "", "Fr", ""];

  // Color scale based on intensity.
  const getColor = (intensity: number): string => {
    if (intensity === 0) return "rgba(255,255,255,0.05)";
    if (intensity < 0.25) return "rgba(124, 92, 255, 0.3)";
    if (intensity < 0.5) return "rgba(124, 92, 255, 0.5)";
    if (intensity < 0.75) return "rgba(124, 92, 255, 0.7)";
    return "rgba(124, 92, 255, 0.95)";
  };

  const cellTitle = ru
    ? (day: HeatCell) =>
        day.date
          ? `${day.date}: ${day.seconds > 0 ? `${Math.round(day.seconds / 60)} мин` : "нет активности"}`
          : ""
    : (day: HeatCell) =>
        day.date
          ? `${day.date}: ${day.seconds > 0 ? `${Math.round(day.seconds / 60)} min` : "no activity"}`
          : "";

  return (
    <div className="w-full overflow-x-auto no-scrollbar">
      <div className="min-w-[600px]">
        {/* Month labels */}
        <div className="flex ml-8 mb-1">
          {monthLabels.map((l, i) => (
            <span
              key={i}
              className="text-[10px] text-dim"
              style={{
                marginLeft:
                  i === 0
                    ? 0
                    : `${(l.weekIndex - (monthLabels[i - 1]?.weekIndex ?? 0) - 1) * 14}px`,
              }}
            >
              {l.month}
            </span>
          ))}
        </div>

        <div className="flex gap-1">
          {/* Day labels */}
          <div className="flex flex-col gap-[3px] pr-1">
            {dayLabels.map((label, i) => (
              <div key={i} className="h-3 text-[9px] text-dim leading-3">
                {label}
              </div>
            ))}
          </div>

          {/* Grid */}
          <div className="flex gap-[3px]">
            {weeks.map((week, wi) => (
              <div key={wi} className="flex flex-col gap-[3px]">
                {week.map((day, di) => (
                  <div
                    key={di}
                    className="w-3 h-3 rounded-[3px] transition-colors duration-200"
                    style={{ background: getColor(day.intensity) }}
                    title={cellTitle(day)}
                  />
                ))}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
