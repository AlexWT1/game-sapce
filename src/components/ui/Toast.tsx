import { AnimatePresence, motion } from "framer-motion";
import { AlertCircle, CheckCircle2, Info, Trophy, X } from "lucide-react";
import { useUiStore, type ToastKind } from "@/stores/uiStore";
import { cx } from "@/utils/format";

const icons: Record<ToastKind, React.ReactNode> = {
  info: <Info className="h-4 w-4 text-sky-400" />,
  success: <CheckCircle2 className="h-4 w-4 text-emerald-400" />,
  error: <AlertCircle className="h-4 w-4 text-red-400" />,
  achievement: <Trophy className="h-4 w-4 text-amber-300" />,
};

export function ToastHost() {
  const toasts = useUiStore((s) => s.toasts);
  const dismiss = useUiStore((s) => s.dismissToast);

  return (
    <div className="fixed bottom-5 right-5 z-[90] flex flex-col gap-2 w-[340px]">
      <AnimatePresence>
        {toasts.map((t) => (
          <motion.div
            key={t.id}
            layout
            initial={{ opacity: 0, x: 40, scale: 0.96 }}
            animate={{ opacity: 1, x: 0, scale: 1 }}
            exit={{ opacity: 0, x: 40, scale: 0.96 }}
            transition={{ duration: 0.22 }}
            className={cx(
              "glass rounded-xl px-3.5 py-3 flex items-start gap-3 shadow-xl",
              t.kind === "achievement" && "border-amber-400/30 bg-amber-500/10",
              t.kind === "error" && "border-red-400/30",
            )}
          >
            <div className="mt-0.5 shrink-0">{icons[t.kind]}</div>
            <div className="min-w-0 flex-1">
              <p className="text-[13px] font-semibold text-ink truncate">{t.title}</p>
              {t.body && <p className="text-[12px] text-muted mt-0.5">{t.body}</p>}
            </div>
            <button
              onClick={() => dismiss(t.id)}
              className="p-1 text-dim hover:text-ink transition"
              aria-label="Dismiss"
            >
              <X className="h-3.5 w-3.5" />
            </button>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
