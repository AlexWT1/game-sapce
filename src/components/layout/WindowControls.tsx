import { Maximize2, Minus, X } from "lucide-react";
import { api } from "@/services/api";
import { useT } from "@/hooks/useT";

export function WindowControls() {
  const t = useT();
  return (
    <div className="flex items-center gap-0.5 ml-1" data-tauri-drag-region="false">
      <button
        onClick={() => void api.windowMinimize()}
        title={t("topbar.minimize")}
        aria-label={t("topbar.minimize")}
        className="h-8 w-8 grid place-items-center rounded-lg text-muted hover:text-ink hover:bg-white/10 transition"
      >
        <Minus className="h-3.5 w-3.5" />
      </button>
      <button
        onClick={() => void api.windowToggleMaximize()}
        title={t("topbar.maximize")}
        aria-label={t("topbar.maximize")}
        className="h-8 w-8 grid place-items-center rounded-lg text-muted hover:text-ink hover:bg-white/10 transition"
      >
        <Maximize2 className="h-3 w-3" />
      </button>
      <button
        onClick={() => void api.windowClose()}
        title={t("topbar.close")}
        aria-label={t("topbar.close")}
        className="h-8 w-8 grid place-items-center rounded-lg text-muted hover:text-white hover:bg-red-500/80 transition"
      >
        <X className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
