import { useEffect, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { AnimatePresence, motion } from "framer-motion";
import { X } from "lucide-react";
import { cx } from "@/utils/format";

interface ModalProps {
  open: boolean;
  onClose: () => void;
  title?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  width?: number;
  className?: string;
}

export function Modal({ open, onClose, title, children, footer, width = 520, className }: ModalProps) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  return createPortal(
    <AnimatePresence>
      {open && (
        <motion.div
          className="fixed inset-0 z-50 grid place-items-center p-6"
          initial={{ opacity: 1 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.12 }}
        >
          <div
            className="absolute inset-0 bg-black/65 backdrop-blur-sm"
            onClick={onClose}
            aria-hidden
          />
          <motion.div
            role="dialog"
            aria-modal="true"
            className={cx(
              "relative w-full glass rounded-2xl shadow-2xl overflow-hidden",
              "bg-[#0d1020]/95",
              className,
            )}
            style={{ maxWidth: width }}
            initial={{ opacity: 1, y: 0, scale: 1 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 10, scale: 0.98 }}
            transition={{ duration: 0.15, ease: [0.2, 0.7, 0.3, 1] }}
          >
            {title && (
              <div className="flex items-center justify-between px-5 py-4 border-b border-white/8">
                <h2 className="text-[15px] font-semibold text-ink">{title}</h2>
                <button
                  onClick={onClose}
                  className="p-1.5 rounded-lg text-muted hover:text-ink hover:bg-white/10 transition"
                  aria-label="Close"
                >
                  <X className="h-4 w-4" />
                </button>
              </div>
            )}
            <div className="p-5 max-h-[70vh] overflow-y-auto">{children}</div>
            {footer && (
              <div className="px-5 py-4 border-t border-white/8 flex justify-end gap-2 bg-white/[0.02]">
                {footer}
              </div>
            )}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>,
    document.body,
  );
}
