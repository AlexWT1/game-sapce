import { Modal } from "@/components/ui/Modal";
import { Button } from "@/components/ui/Button";
import { useUiStore } from "@/stores/uiStore";
import { useT } from "@/hooks/useT";

export function ConfirmDialog() {
  const confirm = useUiStore((s) => s.confirm);
  const askConfirm = useUiStore((s) => s.askConfirm);
  const t = useT();

  const close = () => askConfirm(null);

  return (
    <Modal
      open={!!confirm}
      onClose={close}
      title={confirm?.title}
      width={440}
      footer={
        <>
          <Button variant="ghost" onClick={close}>
            {t("dialog.cancel")}
          </Button>
          <Button
            variant={confirm?.danger ? "danger" : "primary"}
            onClick={() => {
              void confirm?.onConfirm();
              close();
            }}
          >
            {confirm?.confirmLabel ?? t("dialog.confirm")}
          </Button>
        </>
      }
    >
      <p className="text-sm text-muted leading-relaxed">{confirm?.body}</p>
    </Modal>
  );
}
