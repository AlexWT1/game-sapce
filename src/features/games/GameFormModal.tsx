import { useEffect, useState } from "react";
import { Modal } from "@/components/ui/Modal";
import { Button } from "@/components/ui/Button";
import { Input, Textarea } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { useUiStore } from "@/stores/uiStore";
import { useGamesStore } from "@/stores/gamesStore";
import { useT } from "@/hooks/useT";
import { api } from "@/services/api";
import type { GameInput } from "@/types";

const GENRES = [
  "RPG",
  "Экшен",
  "Приключения",
  "Стратегия",
  "Симулятор",
  "Гонки",
  "Спорт",
  "Хоррор",
  "Инди",
  "Шутер",
];

export function GameFormModal() {
  const t = useT();
  const open = useUiStore((s) => s.addGameOpen);
  const prefill = useUiStore((s) => s.addGamePrefill);
  const editing = useUiStore((s) => s.editingGame);
  const closeAdd = useUiStore((s) => s.closeAddGame);
  const openEdit = useUiStore((s) => s.openEditGame);
  const pushToast = useUiStore((s) => s.pushToast);
  const create = useGamesStore((s) => s.create);
  const update = useGamesStore((s) => s.update);
  const load = useGamesStore((s) => s.load);

  const isOpen = open || !!editing;

  const [form, setForm] = useState({
    title: "",
    description: "",
    executablePath: "",
    developer: "",
    publisher: "",
    genre: "",
    releaseYear: "",
    platform: "PC",
    coverPath: "",
  });
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (editing) {
      setForm({
        title: editing.title,
        description: editing.description,
        executablePath: editing.executablePath ?? "",
        developer: editing.developer,
        publisher: editing.publisher,
        genre: editing.genre,
        releaseYear: editing.releaseYear ? String(editing.releaseYear) : "",
        platform: editing.platform,
        coverPath: editing.coverPath ?? "",
      });
    } else if (open) {
      setForm({
        title: prefill.title ?? "",
        description: "",
        executablePath: prefill.executablePath ?? "",
        developer: "",
        publisher: "",
        genre: "",
        releaseYear: "",
        platform: "PC",
        coverPath: "",
      });
    }
  }, [open, editing, prefill]);

  const close = () => {
    if (editing) openEdit(null);
    closeAdd();
  };

  const pickExe = async () => {
    try {
      const path = await api.selectGameExecutable();
      if (path) setForm((f) => ({ ...f, executablePath: path }));
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    }
  };

  const submit = async () => {
    if (!form.title.trim()) {
      pushToast({ kind: "error", title: t("toast.error"), body: "Title required" });
      return;
    }
    setBusy(true);
    const payload: GameInput = {
      title: form.title.trim(),
      description: form.description,
      executablePath: form.executablePath || null,
      developer: form.developer,
      publisher: form.publisher,
      genre: form.genre,
      releaseYear: form.releaseYear ? Number(form.releaseYear) : null,
      platform: form.platform,
      coverPath: form.coverPath || null,
    };
    try {
      if (editing) {
        await update(editing.id, payload);
        pushToast({ kind: "success", title: form.title });
      } else {
        await create({ ...payload, isInstalled: !!form.executablePath });
        pushToast({ kind: "success", title: t("dialog.add"), body: form.title });
      }
      await load(true);
      close();
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <Modal
      open={isOpen}
      onClose={close}
      title={editing ? t("game.edit") : t("library.addGame")}
      width={560}
      footer={
        <>
          <Button variant="ghost" onClick={close}>
            {t("dialog.cancel")}
          </Button>
          <Button variant="primary" loading={busy} onClick={() => void submit()}>
            {editing ? t("dialog.save") : t("dialog.add")}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <Input
          label={t("game.title")}
          placeholder="Cyberpunk 2077"
          value={form.title}
          onChange={(e) => setForm({ ...form, title: e.target.value })}
        />

        <div>
          <span className="block text-[12px] font-medium text-muted uppercase tracking-wide mb-1.5">
            {t("game.specifyFile")}
          </span>
          <div className="flex gap-2">
            <input
              value={form.executablePath}
              onChange={(e) => setForm({ ...form, executablePath: e.target.value })}
              placeholder="/path/to/game"
              className="flex-1 h-10 px-3 rounded-xl bg-white/[0.05] border border-white/10 text-sm text-ink placeholder:text-dim outline-none focus:border-[var(--color-accent)]/60"
            />
            <Button variant="secondary" onClick={() => void pickExe()}>
              …
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <Input
            label={t("game.developer")}
            value={form.developer}
            onChange={(e) => setForm({ ...form, developer: e.target.value })}
          />
          <Input
            label={t("game.publisher")}
            value={form.publisher}
            onChange={(e) => setForm({ ...form, publisher: e.target.value })}
          />
        </div>

        <div className="grid grid-cols-3 gap-3">
          <Select
            label={t("game.genre")}
            options={[
              { value: "", label: "—" },
              ...GENRES.map((g) => ({ value: g, label: g })),
            ]}
            value={form.genre}
            onChange={(e) => setForm({ ...form, genre: e.target.value })}
          />
          <Input
            label={t("game.year")}
            inputMode="numeric"
            placeholder="2024"
            value={form.releaseYear}
            onChange={(e) => setForm({ ...form, releaseYear: e.target.value })}
          />
          <Select
            label={t("game.platform")}
            options={[
              { value: "PC", label: "PC" },
              { value: "Windows", label: "Windows" },
              { value: "Linux", label: "Linux" },
              { value: "macOS", label: "macOS" },
            ]}
            value={form.platform}
            onChange={(e) => setForm({ ...form, platform: e.target.value })}
          />
        </div>

        <Textarea
          label={t("game.about")}
          value={form.description}
          onChange={(e) => setForm({ ...form, description: e.target.value })}
        />
      </div>
    </Modal>
  );
}
