import { useState } from "react";
import { Modal } from "@/components/ui/Modal";
import { Button } from "@/components/ui/Button";
import { useUiStore } from "@/stores/uiStore";
import { useGamesStore } from "@/stores/gamesStore";
import { useT } from "@/hooks/useT";
import { api } from "@/services/api";
import { AlertTriangle } from "lucide-react";

/**
 * Shown when launch fails because the executable path is missing on disk.
 * User can pick a new path without re-opening the full edit form.
 */
export function MissingExecutableDialog() {
  const t = useT();
  const game = useUiStore((s) => s.missingExeGame);
  const close = useUiStore((s) => s.closeMissingExe);
  const pushToast = useUiStore((s) => s.pushToast);
  const setExecutable = useGamesStore((s) => s.setExecutable);
  const [busy, setBusy] = useState(false);

  const show = !!game;

  const pick = async () => {
    if (!game) return;
    setBusy(true);
    try {
      const path = await api.selectGameExecutable();
      if (!path) return;
      await setExecutable(game.id, path);
      pushToast({ kind: "success", title: t("game.specifyFile"), body: path });
      close();
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <Modal
      open={show}
      onClose={close}
      title={
        <span className="flex items-center gap-2">
          <AlertTriangle className="h-4 w-4 text-amber-400" />
          {t("dialog.missingExe")}
        </span>
      }
      width={480}
      footer={
        <>
          <Button variant="ghost" onClick={close}>
            {t("dialog.cancel")}
          </Button>
          <Button variant="primary" loading={busy} onClick={() => void pick()}>
            {t("dialog.specifyNewPath")}
          </Button>
        </>
      }
    >
      <p className="text-sm text-muted">{t("dialog.missingExeBody")}</p>
      <code className="mt-2 block px-3 py-2 rounded-lg bg-black/40 border border-white/10 text-[12.5px] text-amber-200 break-all">
        {game?.executablePath}
      </code>
    </Modal>
  );
}
