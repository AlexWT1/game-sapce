import { useEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { AnimatePresence, motion } from "framer-motion";
import { cx } from "@/utils/format";

export interface ContextMenuItem {
  key: string;
  label: string;
  icon?: ReactNode;
  danger?: boolean;
  disabled?: boolean;
  onSelect: () => void;
}

interface ContextMenuProps {
  items: ContextMenuItem[];
  children: ReactNode;
  className?: string;
}

interface Position {
  x: number;
  y: number;
}

export function ContextMenu({ items, children, className }: ContextMenuProps) {
  const [pos, setPos] = useState<Position | null>(null);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!pos) return;
    const close = () => setPos(null);
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && setPos(null);
    window.addEventListener("click", close);
    window.addEventListener("scroll", close, true);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("click", close);
      window.removeEventListener("scroll", close, true);
      window.removeEventListener("keydown", onKey);
    };
  }, [pos]);

  const open = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setPos({ x: e.clientX, y: e.clientY });
  };

  const menu = pos
    ? createPortal(
        <AnimatePresence>
          <motion.div
            className="fixed z-[70] min-w-[200px] py-1.5 rounded-xl overflow-hidden
              bg-[#12152b]/95 border border-white/10 shadow-2xl backdrop-blur-xl"
            style={{
              left: Math.min(pos.x, window.innerWidth - 220),
              top: Math.min(pos.y, window.innerHeight - 40 - items.length * 36),
            }}
            initial={{ opacity: 0, scale: 0.96, y: -4 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.12 }}
            onClick={(e) => e.stopPropagation()}
          >
            {items.map((item) => (
              <button
                key={item.key}
                disabled={item.disabled}
                onClick={() => {
                  item.onSelect();
                  setPos(null);
                }}
                className={cx(
                  "w-full flex items-center gap-2.5 px-3 py-2 text-[13px] text-left transition",
                  "disabled:opacity-40 disabled:pointer-events-none",
                  item.danger
                    ? "text-red-400 hover:bg-red-500/12"
                    : "text-ink/90 hover:bg-white/8",
                )}
              >
                {item.icon}
                {item.label}
              </button>
            ))}
          </motion.div>
        </AnimatePresence>,
        document.body,
      )
    : null;

  return (
    <div ref={ref} className={className} onContextMenu={open}>
      {children}
      {menu}
    </div>
  );
}
