import { useState } from "react";
import { Modal } from "@/components/ui/Modal";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { useT } from "@/hooks/useT";
import { useUiStore } from "@/stores/uiStore";
import { api } from "@/services/api";

interface SteamLinkModalProps {
  open: boolean;
  onClose: () => void;
  onLinked: () => void | Promise<void>;
}

export function SteamLinkModal({ open, onClose, onLinked }: SteamLinkModalProps) {
  const t = useT();
  const pushToast = useUiStore((s) => s.pushToast);
  const [apiKey, setApiKey] = useState("");
  const [profileUrl, setProfileUrl] = useState("");
  const [busy, setBusy] = useState(false);

  const submit = async () => {
    setBusy(true);
    try {
      await api.linkSteamAccount(apiKey.trim(), profileUrl.trim());
      pushToast({ kind: "success", title: t("shop.steam.linked") });
      setApiKey("");
      setProfileUrl("");
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
      title={t("shop.steam.connect")}
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
        <p className="text-sm text-muted leading-relaxed">{t("shop.steam.hint")}</p>
        <Input
          label={t("shop.steam.apiKey")}
          type="password"
          autoComplete="off"
          placeholder="0123456789abcdef0123456789abcdef"
          hint={t("shop.steam.apiKeyHint")}
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
        />
        <Input
          label={t("shop.steam.profileUrl")}
          placeholder={t("shop.steam.profilePlaceholder")}
          value={profileUrl}
          onChange={(e) => setProfileUrl(e.target.value)}
        />
      </div>
    </Modal>
  );
}
