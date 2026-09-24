import { useEffect, useRef, useState, type ReactNode } from "react";
import { cx } from "@/utils/format";

interface TooltipProps {
  content: string;
  children: ReactNode;
  side?: "top" | "bottom";
  className?: string;
}

export function Tooltip({ content, children, side = "top", className }: TooltipProps) {
  const [open, setOpen] = useState(false);
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  const show = () => {
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setOpen(true), 350);
  };
  const hide = () => {
    window.clearTimeout(timer.current);
    setOpen(false);
  };

  return (
    <span
      className={cx("relative inline-flex", className)}
      onMouseEnter={show}
      onMouseLeave={hide}
      onFocus={() => setOpen(true)}
      onBlur={hide}
    >
      {children}
      {open && (
        <span
          role="tooltip"
          className={cx(
            "absolute left-1/2 -translate-x-1/2 z-[80] whitespace-nowrap px-2 py-1 rounded-md",
            "text-[11px] text-ink bg-[#12152b] border border-white/10 shadow-lg pointer-events-none",
            side === "top" ? "bottom-full mb-1.5" : "top-full mt-1.5",
          )}
        >
          {content}
        </span>
      )}
    </span>
  );
}
