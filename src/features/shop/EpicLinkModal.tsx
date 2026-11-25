import { useState } from "react";
import { ExternalLink } from "lucide-react";
import { Modal } from "@/components/ui/Modal";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { useT } from "@/hooks/useT";
import { useUiStore } from "@/stores/uiStore";
import { api } from "@/services/api";

const EPIC_AUTH_URL =
  "https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2Fwww.epicgames.com%2Fid%2Fapi%2Fredirect%3FclientId%3D34a02cf8f4414e29b15921876da36f9a%26responseType%3Dcode";

interface EpicLinkModalProps {
  open: boolean;
  onClose: () => void;
  onLinked: () => void | Promise<void>;
}

export function EpicLinkModal({ open, onClose, onLinked }: EpicLinkModalProps) {
  const t = useT();
  const pushToast = useUiStore((s) => s.pushToast);
  const [authCode, setAuthCode] = useState("");
  const [busy, setBusy] = useState(false);

  const openLogin = () => {
    void api.openPath(EPIC_AUTH_URL).catch((e) => {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    });
  };

  const submit = async () => {
    setBusy(true);
    try {
      await api.linkEpicAccount(authCode.trim());
      pushToast({ kind: "success", title: t("shop.epic.linked") });
      setAuthCode("");
      await onLinked();
      onClose();
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={t("shop.epic.connect")}
      width={520}
      footer={
        <>
          <Button variant="ghost" onClick={onClose} disabled={busy}>
            {t("dialog.cancel")}
          </Button>
          <Button variant="primary" loading={busy} onClick={() => void submit()}>
            {t("shop.steam.linkAction")}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <p className="text-sm text-muted leading-relaxed">{t("shop.epic.hint")}</p>
        <div>
          <Button
            type="button"
            variant="secondary"
            size="sm"
            icon={<ExternalLink className="h-3.5 w-3.5" />}
            onClick={openLogin}
          >
            {t("shop.epic.openLogin")}
          </Button>
        </div>
        <Input
          label={t("shop.epic.authCode")}
          type="password"
          autoComplete="off"
          placeholder={t("shop.epic.authCodePlaceholder")}
          hint={t("shop.epic.authCodeHint")}
          value={authCode}
          onChange={(e) => setAuthCode(e.target.value)}
        />
      </div>
    </Modal>
  );
}
