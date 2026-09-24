import type { ReactNode } from "react";
import { cx } from "@/utils/format";

interface EmptyStateProps {
  icon?: ReactNode;
  title: string;
  body?: string;
  action?: ReactNode;
  className?: string;
}

export function EmptyState({ icon, title, body, action, className }: EmptyStateProps) {
  return (
    <div
      className={cx(
        "flex flex-col items-center justify-center text-center py-16 px-6 rounded-2xl",
        "border border-dashed border-white/10 bg-white/[0.02]",
        className,
      )}
    >
      {icon && (
        <div className="mb-4 grid place-items-center h-14 w-14 rounded-2xl bg-[var(--color-accent)]/12 text-[var(--color-accent)]">
          {icon}
        </div>
      )}
      <h3 className="text-[15px] font-semibold text-ink">{title}</h3>
      {body && <p className="mt-1.5 text-[13px] text-muted max-w-sm leading-relaxed">{body}</p>}
      {action && <div className="mt-5">{action}</div>}
    </div>
  );
}
