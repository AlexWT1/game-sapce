import type { PlaytimeByDay } from "@/types";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { formatPlaytime } from "@/utils/format";

interface PlaytimeChartProps {
  data: PlaytimeByDay[];
  height?: number;
}

export function PlaytimeChart({ data, height = 200 }: PlaytimeChartProps) {
  const chartData = data.map((d) => ({
    name: shortLabel(d.date),
    seconds: d.seconds,
    label: formatPlaytime(d.seconds),
  }));

  return (
    <ResponsiveContainer width="100%" height={height}>
      <BarChart data={chartData} margin={{ top: 8, right: 8, left: -12, bottom: 0 }}>
        <CartesianGrid stroke="rgba(255,255,255,0.06)" vertical={false} />
        <XAxis
          dataKey="name"
          tick={{ fill: "#6b7189", fontSize: 11 }}
          axisLine={false}
          tickLine={false}
        />
        <YAxis
          tick={{ fill: "#6b7189", fontSize: 11 }}
          axisLine={false}
          tickLine={false}
          tickFormatter={(v: number) => {
            const h = Math.floor(v / 3600);
            return h > 0 ? `${h}ч` : `${Math.floor(v / 60)}м`;
          }}
        />
        <Tooltip
          cursor={{ fill: "rgba(255,255,255,0.04)" }}
          contentStyle={{
            background: "#12152b",
            border: "1px solid rgba(255,255,255,0.1)",
            borderRadius: 12,
            fontSize: 12,
          }}
          labelStyle={{ color: "#eceef8" }}
          formatter={(value: number) => [formatPlaytime(value), "Время"]}
        />
        <Bar dataKey="seconds" fill="var(--color-accent)" radius={[6, 6, 0, 0]} maxBarSize={28} />
      </BarChart>
    </ResponsiveContainer>
  );
}

const PALETTE = ["#7c5cff", "#5b8cff", "#22d3ee", "#34d399", "#f59e0b", "#fb7185", "#a78bfa"];

interface GenreChartProps {
  data: Array<{ genre: string; count: number }>;
  height?: number;
}

export function GenreChart({ data, height = 200 }: GenreChartProps) {
  const chart = data.slice(0, 6).map((d) => ({ name: d.genre, value: d.count }));
  return (
    <ResponsiveContainer width="100%" height={height}>
      <PieChart>
        <Pie
          data={chart}
          dataKey="value"
          nameKey="name"
          innerRadius={52}
          outerRadius={80}
          paddingAngle={3}
          stroke="transparent"
        >
          {chart.map((_, i) => (
            <Cell key={i} fill={PALETTE[i % PALETTE.length]} />
          ))}
        </Pie>
        <Tooltip
          contentStyle={{
            background: "#12152b",
            border: "1px solid rgba(255,255,255,0.1)",
            borderRadius: 12,
            fontSize: 12,
          }}
        />
      </PieChart>
    </ResponsiveContainer>
  );
}

function shortLabel(date: string): string {
  // "2026-09-24" → "24.09" ; "2026-09-01" → "сен"
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(date);
  if (!m) return date.slice(-5);
  if (date.endsWith("-01") && date.length === 7) {
    const months = ["янв", "фев", "мар", "апр", "май", "июн", "июл", "авг", "сен", "окт", "ноя", "дек"];
    return months[Number(m[2]) - 1] ?? m[2] ?? date;
  }
  return `${m[3]}.${m[2]}`;
}
