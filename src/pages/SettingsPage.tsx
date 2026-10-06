import { useEffect, useRef, useState } from "react";
import {
  Bell,
  Database,
  Download,
  Info,
  Monitor,
  Palette,
  RefreshCw,
  RotateCcw,
  Upload,
} from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { Toggle } from "@/components/ui/Toggle";
import { Chip } from "@/components/ui/Chip";
import { useT } from "@/hooks/useT";
import { useSettingsStore, applyVisualPreferences, type AccentColor, type ThemeMode } from "@/stores/settingsStore";
import { useUiStore } from "@/stores/uiStore";
import { api } from "@/services/api";
import { useUpdater } from "@/hooks/useUpdater";
import { ProgressBar } from "@/components/ui/ProgressBar";
import { formatBytes } from "@/utils/format";
import { useLocaleCode } from "@/hooks/useT";
import { LOCALES, type Locale } from "@/i18n";
import type { AppInfo, SettingsMap } from "@/types";

export function SettingsPage() {
  const t = useT();
  const local = useSettingsStore();
  const pushToast = useUiStore((s) => s.pushToast);
  const askConfirm = useUiStore((s) => s.askConfirm);
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [savedFlash, setSavedFlash] = useState(false);
  const [igdbClientId, setIgdbClientId] = useState("");
  const [igdbClientSecret, setIgdbClientSecret] = useState("");

  useEffect(() => {
    void api.getAppInfo().then(setInfo).catch(() => setInfo(null));
    void api
      .getSettings()
      .then((settings) => {
        setIgdbClientId(String(settings.igdb_client_id ?? ""));
        setIgdbClientSecret(String(settings.igdb_client_secret ?? ""));
      })
      .catch(() => undefined);
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

  const onSaveIgdb = async () => {
    await persistBackend({
      igdb_client_id: igdbClientId.trim(),
      igdb_client_secret: igdbClientSecret.trim(),
    });
    pushToast({ kind: "success", title: t("settings.igdb.saved") });
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
        {import.meta.env.DEV && (
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
        )}
        <div className="border-t border-white/[0.06] pt-4 space-y-3">
          <span className="block text-[12px] font-semibold text-ink uppercase tracking-wide">
            IGDB
          </span>
          <span className="block text-[12px] text-dim leading-relaxed">
            {t("settings.igdb.hint")}
          </span>
          <div className="grid grid-cols-2 gap-3">
            <Input
              label={t("settings.igdb.clientId")}
              value={igdbClientId}
              onChange={(e) => setIgdbClientId(e.target.value)}
              placeholder="abc123"
            />
            <Input
              label={t("settings.igdb.clientSecret")}
              type="password"
              value={igdbClientSecret}
              onChange={(e) => setIgdbClientSecret(e.target.value)}
              placeholder="def456"
            />
          </div>
          <div className="flex justify-end">
            <Button size="sm" variant="outline" onClick={() => void onSaveIgdb()}>
              {t("common.save")}
            </Button>
          </div>
        </div>
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
        <Row label={t("settings.checkUpdates")} control={<UpdaterBlock />} />
      </Section>
    </div>
  );
}

/** Update check / download / install flow for the launcher itself. */
function UpdaterBlock() {
  const t = useT();
  const locale = useLocaleCode();
  const pushToast = useUiStore((s) => s.pushToast);
  const updater = useUpdater();
  const { stage, check, progress, error } = updater;
  const autoChecked = useRef(false);

  useEffect(() => {
    if (autoChecked.current) return;
    autoChecked.current = true;
    void updater.checkForUpdates();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (stage === "ready") {
      pushToast({ kind: "success", title: t("settings.updateReady") });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [stage]);

  const onInstall = async () => {
    try {
      await updater.install();
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    }
  };

  const sizeLabel =
    check?.downloadSize != null ? ` (${formatBytes(check.downloadSize, locale)})` : "";

  return (
    <div className="flex flex-col items-end gap-2">
      {stage === "idle" && (
        <div className="flex items-center gap-2">
          {check && !check.updateAvailable && (
            <Chip tone="success">{t("settings.upToDate")}</Chip>
          )}
          <Button
            variant="secondary"
            size="sm"
            icon={<RefreshCw className="h-3.5 w-3.5" />}
            onClick={() => void updater.checkForUpdates()}
          >
            {t("settings.checkUpdates")}
          </Button>
        </div>
      )}

      {stage === "checking" && (
        <Button variant="secondary" size="sm" loading>
          {t("settings.checking")}
        </Button>
      )}

      {stage === "available" && check?.latestVersion && (
        <div className="flex items-center gap-2">
          <Chip tone="accent">
            {t("settings.updateAvailable")} — v{check.latestVersion}
          </Chip>
          <Button
            size="sm"
            icon={<Download className="h-3.5 w-3.5" />}
            onClick={() => void updater.startDownload()}
          >
            {t("settings.downloadUpdate")}
            {sizeLabel}
          </Button>
        </div>
      )}

      {stage === "downloading" && (
        <div className="flex flex-col items-end gap-1.5 w-64">
          <div className="flex items-center justify-between w-full text-[12px] text-muted">
            <span>{t("settings.downloadProgress")}</span>
            <span>{Math.round(progress?.percent ?? 0)}%</span>
          </div>
          <ProgressBar value={progress?.percent ?? 0} />
          <Button variant="ghost" size="sm" onClick={() => void updater.cancelDownload()}>
            {t("settings.cancelDownload")}
          </Button>
        </div>
      )}

      {stage === "ready" && (
        <div className="flex items-center gap-2">
          <Chip tone="success">
            {t("settings.updateReady")}
            {check?.latestVersion ? ` — v${check.latestVersion}` : ""}
          </Chip>
          <Button size="sm" onClick={() => void onInstall()}>
            {t("settings.installRestart")}
          </Button>
        </div>
      )}

      {stage === "error" && (
        <div className="flex items-center gap-2">
          <Chip tone="danger">{t("settings.updateFailed")}</Chip>
          <Button
            variant="secondary"
            size="sm"
            icon={<RefreshCw className="h-3.5 w-3.5" />}
            onClick={() => void updater.checkForUpdates()}
          >
            {t("settings.checkUpdates")}
          </Button>
        </div>
      )}

      {error && stage === "error" && (
        <span className="text-[11px] text-muted max-w-64 text-right break-all">{error}</span>
      )}
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
