import { useEffect, useState } from "react";
import {
  Bell,
  Database,
  Download,
  FolderOpen,
  Info,
  Monitor,
  Palette,
  RotateCcw,
  Search,
  Upload,
} from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Select } from "@/components/ui/Select";
import { Toggle } from "@/components/ui/Toggle";
import { Chip } from "@/components/ui/Chip";
import { useT } from "@/hooks/useT";
import { useSettingsStore, applyVisualPreferences, type AccentColor, type ThemeMode } from "@/stores/settingsStore";
import { useUiStore } from "@/stores/uiStore";
import { api } from "@/services/api";
import { scanForGames } from "@/services/discovery";
import { LOCALES, type Locale } from "@/i18n";
import type { AppInfo, SettingsMap } from "@/types";

export function SettingsPage() {
  const t = useT();
  const local = useSettingsStore();
  const pushToast = useUiStore((s) => s.pushToast);
  const askConfirm = useUiStore((s) => s.askConfirm);
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [savedFlash, setSavedFlash] = useState(false);
  const [scanning, setScanning] = useState(false);

  useEffect(() => {
    void api.getAppInfo().then(setInfo).catch(() => setInfo(null));
  }, []);

  const persistBackend = async (patch: SettingsMap) => {
    try {
      await api.updateSettings(patch);
      setSavedFlash(true);
      window.setTimeout(() => setSavedFlash(false), 1500);
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    }
  };

  const onExport = async () => {
    try {
      const path = await api.exportData();
      pushToast({ kind: "success", title: t("toast.exportDone"), body: path });
    } catch (e) {
      if (!String(e).includes("отменена")) {
        pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
      }
    }
  };

  const onImport = async () => {
    try {
      const [games, sessions] = await api.importData();
      pushToast({
        kind: "success",
        title: t("toast.importDone"),
        body: `${t("toast.imported")}: ${games} / ${sessions}`,
      });
      const { useGamesStore } = await import("@/stores/gamesStore");
      const { useStatisticsStore } = await import("@/stores/statisticsStore");
      await useGamesStore.getState().load(true);
      await useStatisticsStore.getState().load(true);
    } catch (e) {
      if (!String(e).includes("отменена")) {
        pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
      }
    }
  };

  const onReset = () =>
    askConfirm({
      title: t("settings.reset"),
      body: t("dialog.deleteBody"),
      danger: true,
      confirmLabel: t("dialog.confirm"),
      onConfirm: async () => {
        await api.resetData();
        pushToast({ kind: "success", title: t("toast.resetDone") });
        const { useGamesStore } = await import("@/stores/gamesStore");
        const { useStatisticsStore } = await import("@/stores/statisticsStore");
        const { useAchievementsStore } = await import("@/stores/achievementsStore");
        await useGamesStore.getState().load(true);
        await useStatisticsStore.getState().load(true);
        await useAchievementsStore.getState().load(null, true);
      },
    });

  const runScan = async (dirs?: string[]) => {
    setScanning(true);
    try {
      const target = dirs ?? (local.defaultGamesDir ? [local.defaultGamesDir] : []);
      const { found, added } = await scanForGames(target);
      pushToast({
        kind: added > 0 ? "success" : "info",
        title: t("settings.scanDone"),
        body: `${t("settings.found")}: ${found} · ${t("settings.added")}: ${added}`,
      });
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    } finally {
      setScanning(false);
    }
  };

  const pickDir = async () => {
    try {
      const dir = await api.selectGameDirectory();
      if (dir) {
        local.set({ defaultGamesDir: dir });
        await persistBackend({ default_games_dir: dir });
        if (local.autoDetect) {
          void runScan([dir]);
        }
      }
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    }
  };

  return (
    <div className="p-5 max-w-3xl space-y-6 min-h-full">
      <div className="flex items-center gap-3">
        <h1 className="text-xl font-bold text-ink">{t("settings.title")}</h1>
        {savedFlash && <Chip tone="success">{t("settings.save")}</Chip>}
      </div>

      <Section icon={<Info className="h-4 w-4" />} title={t("settings.general")}>
        <Row
          label={t("settings.language")}
          control={
            <Select
              options={LOCALES.map((l) => ({ value: l.value, label: l.label }))}
              value={local.locale}
              onChange={(e) => {
                const locale = e.target.value as Locale;
                local.set({ locale });
                void persistBackend({ language: locale });
              }}
              className="w-40"
            />
          }
        />
        <Row
          label={t("settings.launchOnStartup")}
          control={
            <Toggle
              checked={local.launchOnStartup}
              onChange={(v) => {
                local.set({ launchOnStartup: v });
                void persistBackend({ launch_on_startup: v });
              }}
            />
          }
        />
        <Row
          label={t("settings.minimizeToTray")}
          control={
            <Toggle
              checked={local.minimizeToTray}
              onChange={(v) => {
                local.set({ minimizeToTray: v });
                void persistBackend({ minimize_to_tray: v });
              }}
            />
          }
        />
      </Section>

      <Section icon={<Palette className="h-4 w-4" />} title={t("settings.appearance")}>
        <Row
          label={t("settings.theme")}
          control={
            <Select
              options={[
                { value: "dark", label: "Dark" },
                { value: "black", label: "Black" },
              ]}
              value={local.theme}
              onChange={(e) => {
                const theme = e.target.value as ThemeMode;
                local.set({ theme });
                void persistBackend({ theme });
              }}
              className="w-40"
            />
          }
        />
        <Row
          label={t("settings.accent")}
          control={
            <div className="flex gap-2">
              {(["violet", "blue", "cyan", "amber", "rose"] as AccentColor[]).map((a) => (
                <button
                  key={a}
                  onClick={() => {
                    local.set({ accent: a });
                    void persistBackend({ accent: a });
                  }}
                  className={`h-7 w-7 rounded-full border-2 transition ${
                    local.accent === a ? "border-white scale-110" : "border-transparent"
                  }`}
                  style={{
                    background:
                      a === "violet"
                        ? "#7c5cff"
                        : a === "blue"
                          ? "#5b8cff"
                          : a === "cyan"
                            ? "#22d3ee"
                            : a === "amber"
                              ? "#f59e0b"
                              : "#fb7185",
                  }}
                  aria-label={a}
                />
              ))}
            </div>
          }
        />
        <Row
          label={t("settings.animations")}
          control={
            <Toggle
              checked={local.animations}
              onChange={(v) => {
                local.set({ animations: v });
                applyVisualPreferences(v, local.transparency);
              }}
            />
          }
        />
        <Row
          label={t("settings.transparency")}
          control={
            <Toggle
              checked={local.transparency}
              onChange={(v) => {
                local.set({ transparency: v });
                applyVisualPreferences(local.animations, v);
              }}
            />
          }
        />
      </Section>

      <Section icon={<Monitor className="h-4 w-4" />} title={t("settings.games")}>
        <Row
          label={t("settings.defaultDir")}
          control={
            <div className="flex gap-2 items-center">
              <span className="text-[12.5px] text-muted truncate max-w-[240px]">
                {local.defaultGamesDir || "—"}
              </span>
              <Button size="sm" variant="outline" onClick={() => void pickDir()}>
                <FolderOpen className="h-3.5 w-3.5" />
              </Button>
            </div>
          }
        />
        <Row
          label={t("settings.autoDetect")}
          control={
            <Toggle
              checked={local.autoDetect}
              onChange={(v) => {
                local.set({ autoDetect: v });
                void persistBackend({ auto_detect: v });
                if (v && local.defaultGamesDir) void runScan([local.defaultGamesDir]);
              }}
            />
          }
        />
        <Row
          label={t("settings.scanNow")}
          control={
            <Button
              size="sm"
              variant="outline"
              loading={scanning}
              icon={<Search className="h-3.5 w-3.5" />}
              onClick={() => void runScan()}
            >
              {t("settings.scanNow")}
            </Button>
          }
        />
        <Row
          label={t("settings.monitorInterval")}
          control={
            <Select
              options={[
                { value: "1", label: "1 c" },
                { value: "2", label: "2 c" },
                { value: "3", label: "3 c" },
                { value: "5", label: "5 c" },
              ]}
              value={String(local.monitoringInterval)}
              onChange={(e) => {
                const monitoringInterval = Number(e.target.value);
                local.set({ monitoringInterval });
                void persistBackend({ monitoring_interval: monitoringInterval });
              }}
              className="w-28"
            />
          }
        />
        <Row
          label={t("settings.demoMode")}
          control={
            <Toggle
              checked={local.demoMode}
              onChange={(v) => {
                local.set({ demoMode: v });
                void persistBackend({ demo_mode: v ? "on" : "off" }).then(async () => {
                  const { useGamesStore } = await import("@/stores/gamesStore");
                  const { useStatisticsStore } = await import("@/stores/statisticsStore");
                  const { useAchievementsStore } = await import("@/stores/achievementsStore");
                  await useGamesStore.getState().load(true);
                  await useStatisticsStore.getState().load(true);
                  await useAchievementsStore.getState().load(null, true);
                });
              }}
            />
          }
        />
      </Section>

      <Section icon={<Bell className="h-4 w-4" />} title={t("settings.notifications")}>
        <Row
          label={t("settings.notifyAchievement")}
          control={
            <Toggle
              checked={local.notifyAchievement}
              onChange={(v) => local.set({ notifyAchievement: v })}
            />
          }
        />
        <Row
          label={t("settings.notifyGameStart")}
          control={
            <Toggle
              checked={local.notifyGameStart}
              onChange={(v) => local.set({ notifyGameStart: v })}
            />
          }
        />
        <Row
          label={t("settings.notifyFriendOnline")}
          control={
            <Toggle
              checked={local.notifyFriendOnline}
              onChange={(v) => local.set({ notifyFriendOnline: v })}
            />
          }
        />
        <Row
          label={t("settings.notifySessionEnd")}
          control={
            <Toggle
              checked={local.notifySessionEnd}
              onChange={(v) => local.set({ notifySessionEnd: v })}
            />
          }
        />
      </Section>

      <Section icon={<Database className="h-4 w-4" />} title={t("settings.data")}>
        <Row
          label={t("settings.dbLocation")}
          control={
            <span className="text-[12px] text-muted break-all max-w-md text-right">
              {info?.databasePath ?? "—"}
            </span>
          }
        />
        <Row
          label={t("settings.export")}
          control={
            <Button size="sm" variant="outline" onClick={() => void onExport()}>
              <Download className="h-3.5 w-3.5" /> {t("settings.export")}
            </Button>
          }
        />
        <Row
          label={t("settings.import")}
          control={
            <Button size="sm" variant="outline" onClick={() => void onImport()}>
              <Upload className="h-3.5 w-3.5" /> {t("settings.import")}
            </Button>
          }
        />
        <Row
          label={t("settings.reset")}
          control={
            <Button size="sm" variant="danger" onClick={onReset}>
              <RotateCcw className="h-3.5 w-3.5" /> {t("settings.reset")}
            </Button>
          }
        />
      </Section>

      <Section icon={<Info className="h-4 w-4" />} title={t("settings.about")}>
        <Row label={t("settings.version")} control={<span className="text-[13px] text-ink">{info?.version ?? "0.1.0"}</span>} />
        <Row label={t("settings.platform")} control={<span className="text-[13px] text-ink">{info?.platform ?? "—"}</span>} />
      </Section>
    </div>
  );
}

function Section({
  icon,
  title,
  children,
}: {
  icon: React.ReactNode;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="glass rounded-2xl p-5">
      <h2 className="flex items-center gap-2 text-[14px] font-semibold text-ink mb-4">
        <span className="text-[var(--color-accent)]">{icon}</span>
        {title}
      </h2>
      <div className="space-y-4">{children}</div>
    </section>
  );
}

function Row({ label, control }: { label: string; control: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-[13.5px] text-muted">{label}</span>
      <div className="flex items-center shrink-0">{control}</div>
    </div>
  );
}
