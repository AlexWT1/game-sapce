import { useEffect, useState } from "react";
import { Download } from "lucide-react";
import { Modal } from "@/components/ui/Modal";
import { Button } from "@/components/ui/Button";
import { Input, Textarea } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { useUiStore } from "@/stores/uiStore";
import { useGamesStore } from "@/stores/gamesStore";
import { useT } from "@/hooks/useT";
import { api } from "@/services/api";
import type { GameInput, IgdbGameMeta } from "@/types";

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

/** IGDB genre names (EN) → GENRES list values. Unmatched genres are skipped. */
const IGDB_GENRE_MAP: Record<string, string> = {
  "Role-playing (RPG)": "RPG",
  Shooter: "Шутер",
  Adventure: "Приключения",
  Strategy: "Стратегия",
  "Turn-based strategy (TBS)": "Стратегия",
  "Real Time Strategy (RTS)": "Стратегия",
  Tactical: "Стратегия",
  Simulator: "Симулятор",
  Simulation: "Симулятор",
  Racing: "Гонки",
  Sport: "Спорт",
  Indie: "Инди",
  Platform: "Экшен",
  "Hack and slash": "Экшен",
  "Beat 'em up": "Экшен",
  Fighting: "Экшен",
  Arcade: "Экшен",
};

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
    useLauncher: false,
    launcherPath: "",
    launcherArgs: "",
  });
  const [busy, setBusy] = useState(false);
  const [igdbBusy, setIgdbBusy] = useState(false);
  const [igdbResults, setIgdbResults] = useState<IgdbGameMeta[] | null>(null);

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
        useLauncher: editing.useLauncher,
        launcherPath: editing.launcherPath ?? "",
        launcherArgs: editing.launcherArgs ?? "",
      });
      setIgdbResults(null);
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
        useLauncher: false,
        launcherPath: "",
        launcherArgs: "",
      });
    }
  }, [open, editing, prefill]);

  const close = () => {
    if (editing) openEdit(null);
    closeAdd();
  };

  /** Fills form fields from IGDB metadata. */
  const applyMeta = (m: IgdbGameMeta) => {
    setForm((f) => ({
      ...f,
      title: m.name || f.title,
      description: m.summary || f.description,
      developer: m.developer || f.developer,
      publisher: m.publisher || f.publisher,
      genre: m.genres.map((g) => IGDB_GENRE_MAP[g]).find(Boolean) ?? f.genre,
      releaseYear: m.releaseYear ? String(m.releaseYear) : f.releaseYear,
      coverPath: m.coverUrl || f.coverPath,
    }));
    setIgdbResults(null);
  };

  /** Searches IGDB by the current title and fills or disambiguates. */
  const fetchMeta = async () => {
    const query = form.title.trim();
    if (!query) {
      pushToast({ kind: "error", title: t("toast.error"), body: t("game.title") });
      return;
    }
    setIgdbBusy(true);
    try {
      const results = await api.igdbSearchGames(query);
      if (results.length === 0) {
        setIgdbResults(null);
        pushToast({ kind: "info", title: t("game.fetchMetaNone") });
      } else {
        const [first] = results;
        if (first && results.length === 1) {
          applyMeta(first);
          pushToast({ kind: "success", title: t("game.fetchMetaDone"), body: first.name });
        } else {
          setIgdbResults(results);
        }
      }
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    } finally {
      setIgdbBusy(false);
    }
  };

  const pickExe = async () => {
    try {
      const path = await api.selectGameExecutable();
      if (path) setForm((f) => ({ ...f, executablePath: path }));
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    }
  };

  const pickLauncher = async () => {
    try {
      const path = await api.selectGameExecutable();
      if (path) setForm((f) => ({ ...f, launcherPath: path }));
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
    const launcherPath = form.useLauncher ? (form.launcherPath || null) : null;
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
      useLauncher: form.useLauncher,
      launcherPath,
      launcherArgs: form.useLauncher ? (form.launcherArgs || null) : null,
    };
    try {
      if (editing) {
        await update(editing.id, payload);
        pushToast({ kind: "success", title: form.title });
      } else {
        const isInstalled = form.useLauncher
          ? Boolean(form.launcherPath)
          : Boolean(form.executablePath);
        await create({ ...payload, isInstalled });
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
        <div>
          <span className="block text-[12px] font-medium text-muted uppercase tracking-wide mb-1.5">
            {t("game.title")}
          </span>
          <div className="flex gap-2">
            <input
              value={form.title}
              onChange={(e) => setForm({ ...form, title: e.target.value })}
              placeholder="Cyberpunk 2077"
              className="flex-1 h-10 px-3 rounded-xl bg-white/[0.05] border border-white/10 text-sm text-ink placeholder:text-dim outline-none focus:border-[var(--color-accent)]/60"
            />
            <Button
              variant="secondary"
              loading={igdbBusy}
              onClick={() => void fetchMeta()}
              title={t("game.igdbHint")}
            >
              <Download className="h-4 w-4" />
              {igdbBusy ? t("game.fetchMetaLoading") : t("game.fetchMeta")}
            </Button>
          </div>
          {igdbResults && (
            <div className="mt-2 rounded-xl border border-white/10 bg-white/[0.03] p-2 space-y-1">
              <span className="block text-[11px] text-dim uppercase tracking-wide px-1">
                {t("game.igdbPick")}
              </span>
              {igdbResults.map((r) => (
                <button
                  key={r.igdbId}
                  type="button"
                  onClick={() => applyMeta(r)}
                  className="w-full text-left px-2.5 py-2 rounded-lg hover:bg-white/[0.06] transition"
                >
                  <span className="block text-[13px] text-ink">{r.name}</span>
                  <span className="block text-[11.5px] text-dim">
                    {r.releaseYear ?? "—"}
                    {r.developer ? ` · ${r.developer}` : ""}
                    {r.genres[0] ? ` · ${r.genres[0]}` : ""}
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Launch mode — Game / Launcher */}
        <div>
          <span className="block text-[12px] font-medium text-muted uppercase tracking-wide mb-2">
            {t("game.launchMode")}
          </span>
          <div className="flex gap-1 p-1 rounded-xl bg-white/[0.05] border border-white/10 mb-3">
            <button
              type="button"
              onClick={() => setForm((f) => ({ ...f, useLauncher: false }))}
              className={
                "!flex-1 !h-8 !rounded-lg !text-sm !font-medium !transition-all " +
                (!form.useLauncher
                  ? "!bg-[var(--color-accent)] !text-white"
                  : "!bg-transparent !text-dim hover:!bg-white/[0.06] hover:!text-ink")
              }
            >
              {t("game.launchModeGame")}
            </button>
            <button
              type="button"
              onClick={() => setForm((f) => ({ ...f, useLauncher: true }))}
              className={
                "!flex-1 !h-8 !rounded-lg !text-sm !font-medium !transition-all " +
                (form.useLauncher
                  ? "!bg-[var(--color-accent)] !text-white"
                  : "!bg-transparent !text-dim hover:!bg-white/[0.06] hover:!text-ink")
              }
            >
              {t("game.launchModeLauncher")}
            </button>
          </div>

          {!form.useLauncher ? (
            /* Direct game exe path */
            <div className="flex gap-2">
              <input
                value={form.executablePath}
                onChange={(e) => setForm((f) => ({ ...f, executablePath: e.target.value }))}
                placeholder="/path/to/game.exe"
                className="flex-1 h-10 px-3 rounded-xl bg-white/[0.05] border border-white/10 text-sm text-ink placeholder:text-dim outline-none focus:border-[var(--color-accent)]/60"
              />
              <Button variant="secondary" onClick={() => void pickExe()}>
                …
              </Button>
            </div>
          ) : (
            /* Launcher path + optional args */
            <div className="space-y-2">
              <div className="flex gap-2">
                <input
                  value={form.launcherPath}
                  onChange={(e) => setForm((f) => ({ ...f, launcherPath: e.target.value }))}
                  placeholder={t("game.launcherPathPlaceholder")}
                  className="flex-1 h-10 px-3 rounded-xl bg-white/[0.05] border border-white/10 text-sm text-ink placeholder:text-dim outline-none focus:border-[var(--color-accent)]/60"
                />
                <Button variant="secondary" onClick={() => void pickLauncher()}>
                  …
                </Button>
              </div>
              <input
                value={form.launcherArgs}
                onChange={(e) => setForm((f) => ({ ...f, launcherArgs: e.target.value }))}
                placeholder={t("game.launcherArgsPlaceholder")}
                className="w-full h-9 px-3 rounded-xl bg-white/[0.05] border border-white/10 text-sm text-ink placeholder:text-dim outline-none focus:border-[var(--color-accent)]/60"
              />
              <p className="text-[11px] text-dim leading-relaxed">{t("game.launcherArgsHint")}</p>
            </div>
          )}
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
