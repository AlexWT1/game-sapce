import { useState } from "react";
import { UserRound } from "lucide-react";
import { Modal } from "@/components/ui/Modal";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { useT } from "@/hooks/useT";
import { usePlayerStore, PLAYER_NAME_MAX_LEN } from "@/stores/playerStore";
import { SHOW_NAME_PROMPT_FOR_TEST } from "@/config/testFlags";

const errorKeys = {
  empty: "profile.nameErrorEmpty",
  tooLong: "profile.nameErrorTooLong",
  saveFailed: "profile.nameErrorSave",
} as const;

/**
 * Запрос ника «Как к вам обращаться?».
 *
 * TEST: показывается при каждом входе, пока включён
 * `SHOW_NAME_PROMPT_FOR_TEST` (см. `src/config/testFlags.ts`).
 * Закрытие окна (крестик / фон / Esc — как и кнопка «Сохранить»)
 * фиксирует введённый ник: он пишется в БД (`settings.player_name`)
 * и сразу отображается на клиенте. При пустом поле окно просто закрывается.
 */
export function PlayerNameModal() {
  if (!SHOW_NAME_PROMPT_FOR_TEST) return null;
  return <PlayerNameModalBody />;
}

function PlayerNameModalBody() {
  const t = useT();
  const name = usePlayerStore((s) => s.nameDraft);
  const setNameDraft = usePlayerStore((s) => s.setNameDraft);
  const storeError = usePlayerStore((s) => s.setPlayerError);
  const [open, setOpen] = useState(true);
  const [saving, setSaving] = useState(false);

  // Save whatever is typed, close the window in any case.
  const commitAndClose = async () => {
    const value = name.trim();
    // TEST-окно: пустое поле = просто закрыть.
    if (!value) {
      setOpen(false);
      return;
    }
    setSaving(true);
    usePlayerStore.getState().clearError();
    const ok = await usePlayerStore.getState().setPlayerName(value);
    setSaving(false);
    if (ok) setOpen(false);
  };

  return (
    <Modal
      open={open}
      onClose={() => void commitAndClose()}
      width={420}
      title={
        <span className="flex items-center gap-2">
          <UserRound className="h-4 w-4 text-[var(--color-accent)]" />
          {t("profile.approachTitle")}
        </span>
      }
    >
      <div className="space-y-3">
        <p className="text-[13px] text-muted leading-relaxed">{t("profile.approachHint")}</p>
        <Input
          autoFocus
          label={t("profile.nameLabel")}
          placeholder={t("profile.namePlaceholder")}
          value={name}
          maxLength={PLAYER_NAME_MAX_LEN}
          onChange={(e) => setNameDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !saving) void commitAndClose();
          }}
        />
        {storeError && (
          <p className="text-[12px] text-red-400">{t(errorKeys[storeError])}</p>
        )}
      </div>
      <div className="mt-5 flex justify-end gap-2">
        <Button variant="ghost" onClick={() => setOpen(false)}>
          {t("common.cancel")}
        </Button>
        <Button variant="primary" loading={saving} onClick={() => void commitAndClose()}>
          {t("common.save")}
        </Button>
      </div>
    </Modal>
  );
}
