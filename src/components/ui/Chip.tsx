import type { ReactNode } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cx } from "@/utils/format";

const chip = cva(
  "inline-flex items-center gap-1 rounded-full px-2.5 py-1 text-[11px] font-medium border transition",
  {
    variants: {
      tone: {
        default: "bg-white/6 border-white/10 text-muted",
        accent: "bg-[var(--color-accent)]/15 border-[var(--color-accent)]/35 text-[var(--color-accent)]",
        success: "bg-emerald-500/12 border-emerald-400/30 text-emerald-300",
        warning: "bg-amber-500/12 border-amber-400/30 text-amber-300",
        danger: "bg-red-500/12 border-red-400/30 text-red-300",
        epic: "bg-purple-500/12 border-purple-400/35 text-purple-300",
        legendary: "bg-orange-500/12 border-orange-400/35 text-orange-300",
      },
    },
    defaultVariants: { tone: "default" },
  },
);

interface ChipProps extends VariantProps<typeof chip> {
  children: ReactNode;
  className?: string;
}

export function Chip({ children, tone, className }: ChipProps) {
  return <span className={cx(chip({ tone }), className)}>{children}</span>;
}
