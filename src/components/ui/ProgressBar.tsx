import { cx } from "@/utils/format";

interface ProgressBarProps {
  value: number;
  max?: number;
  className?: string;
  color?: string;
  height?: number;
}

export function ProgressBar({
  value,
  max = 100,
  className,
  color = "var(--color-accent)",
  height = 6,
}: ProgressBarProps) {
  const pct = max <= 0 ? 0 : Math.min(100, Math.max(0, (value / max) * 100));
  return (
    <div
      className={cx("w-full rounded-full bg-white/10 overflow-hidden", className)}
      style={{ height }}
      role="progressbar"
      aria-valuenow={Math.round(pct)}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      <div
        className="h-full rounded-full transition-[width] duration-500"
        style={{
          width: `${pct}%`,
          background: `linear-gradient(90deg, ${color}, color-mix(in srgb, ${color} 60%, #5b8cff))`,
        }}
      />
    </div>
  );
}
