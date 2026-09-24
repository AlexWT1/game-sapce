import { useState } from "react";
import { Play, Square, FileCog } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { useSessionStore } from "@/stores/sessionStore";
import { useUiStore } from "@/stores/uiStore";
import { useT } from "@/hooks/useT";
import { useLocaleCode } from "@/hooks/useT";
import { formatHMS } from "@/utils/format";
import type { Game } from "@/types";

interface LaunchButtonProps {
  game: Game;
  size?: "md" | "lg";
  block?: boolean;
}

export function LaunchButton({ game, size = "md", block }: LaunchButtonProps) {
  const t = useT();
  const locale = useLocaleCode();
  const running = useSessionStore((s) => s.running.some((r) => r.gameId === game.id));
  const live = useSessionStore((s) => s.liveSeconds[game.id] ?? 0);
  const launch = useSessionStore((s) => s.launch);
  const stop = useSessionStore((s) => s.stop);
  const openMissingExe = useUiStore((s) => s.openMissingExe);
  const pushToast = useUiStore((s) => s.pushToast);
  const loadGames = () => import("@/stores/gamesStore").then((m) => m.useGamesStore.getState().load(true));
  const [busy, setBusy] = useState(false);
  void locale;

  const missingExe = !game.executablePath;

  const onPlay = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (missingExe) {
      openMissingExe(game);
      return;
    }
    setBusy(true);
    try {
      await launch(game.id);
      await loadGames();
    } catch (err) {
      const msg = String(err);
      if (/не найден|not found|Файл/i.test(msg)) {
        openMissingExe(game);
        pushToast({ kind: "error", title: t("dialog.missingExe"), body: msg });
      } else {
        pushToast({ kind: "error", title: t("toast.error"), body: msg });
      }
    } finally {
      setBusy(false);
    }
  };

  const onStop = async (e: React.MouseEvent) => {
    e.stopPropagation();
    setBusy(true);
    try {
      await stop(game.id);
      await loadGames();
    } catch (err) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(err) });
    } finally {
      setBusy(false);
    }
  };

  if (running) {
    return (
      <Button
        variant="secondary"
        size={size}
        block={block}
        loading={busy}
        onClick={onStop}
        icon={<Square className="h-4 w-4" />}
        className="border-emerald-400/40 text-emerald-300"
      >
        <span className="tabular-nums">{formatHMS(live)}</span>
        <span className="mx-1 text-dim">·</span>
        {t("game.stop")}
      </Button>
    );
  }

  if (missingExe) {
    return (
      <Button
        variant="outline"
        size={size}
        block={block}
        onClick={(e) => {
          e.stopPropagation();
          openMissingExe(game);
        }}
        icon={<FileCog className="h-4 w-4" />}
      >
        {t("game.specifyFile")}
      </Button>
    );
  }

  return (
    <Button
      variant="primary"
      size={size}
      block={block}
      loading={busy}
      onClick={onPlay}
      icon={<Play className="h-4 w-4 fill-current" />}
      className={block ? "w-full" : undefined}
    >
      {t("game.play")}
    </Button>
  );
}
