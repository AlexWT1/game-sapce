import { useCallback, useEffect, useRef, useState } from "react";
import { Gamepad2, Link2, RefreshCw, Rocket, Sparkles } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Chip } from "@/components/ui/Chip";
import { Skeleton } from "@/components/ui/Skeleton";
import { SteamLinkModal } from "@/features/shop/SteamLinkModal";
import { EpicLinkModal } from "@/features/shop/EpicLinkModal";
import { useLocale, useT } from "@/hooks/useT";
import { useGamesStore } from "@/stores/gamesStore";
import { useUiStore } from "@/stores/uiStore";
import { api } from "@/services/api";
import { formatDateTime } from "@/utils/format";
import type { StoreAccount } from "@/types";

export function ShopPage() {
  const t = useT();
  const pushToast = useUiStore((s) => s.pushToast);
  const askConfirm = useUiStore((s) => s.askConfirm);

  const [steamAccount, setSteamAccount] = useState<StoreAccount | null>(null);
  const [epicAccount, setEpicAccount] = useState<StoreAccount | null>(null);
  const [loading, setLoading] = useState(true);
  const [steamSyncing, setSteamSyncing] = useState(false);
  const [epicSyncing, setEpicSyncing] = useState(false);
  const [steamLinkOpen, setSteamLinkOpen] = useState(false);
  const [epicLinkOpen, setEpicLinkOpen] = useState(false);
  const steamSyncRef = useRef(false);
  const epicSyncRef = useRef(false);

  const reload = useCallback(async () => {
    try {
      const [steam, epic] = await Promise.all([
        api.getStoreAccount(),
        api.getEpicAccount(),
      ]);
      setSteamAccount(steam);
      setEpicAccount(epic);
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    } finally {
      setLoading(false);
    }
  }, [pushToast, t]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const syncSteam = useCallback(async () => {
    if (steamSyncRef.current) return;
    steamSyncRef.current = true;
    setSteamSyncing(true);
    pushToast({ kind: "info", title: t("shop.steam.syncing") });
    try {
      const result = await api.syncSteamLibrary();
      pushToast({
        kind: "success",
        title: t("shop.steam.syncDone"),
        body: `${t("shop.steam.added")}: ${result.added} · ${t("shop.steam.updated")}: ${result.updated}`,
      });
      await reload();
      void useGamesStore.getState().load(true);
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    } finally {
      steamSyncRef.current = false;
      setSteamSyncing(false);
    }
  }, [pushToast, reload, t]);

  const syncEpic = useCallback(async () => {
    if (epicSyncRef.current) return;
    epicSyncRef.current = true;
    setEpicSyncing(true);
    pushToast({ kind: "info", title: t("shop.epic.syncing") });
    try {
      const result = await api.syncEpicLibrary();
      pushToast({
        kind: "success",
        title: t("shop.epic.syncDone"),
        body: `${t("shop.steam.added")}: ${result.added} · ${t("shop.steam.updated")}: ${result.updated}`,
      });
      await reload();
      void useGamesStore.getState().load(true);
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    } finally {
      epicSyncRef.current = false;
      setEpicSyncing(false);
    }
  }, [pushToast, reload, t]);

  const unlinkSteam = useCallback(() => {
    askConfirm({
      title: t("shop.steam.unlinkConfirm"),
      body: t("shop.steam.unlinkConfirmBody"),
      confirmLabel: t("shop.steam.unlink"),
      danger: true,
      onConfirm: async () => {
        try {
          await api.unlinkSteamAccount();
          pushToast({ kind: "success", title: t("shop.steam.unlinked") });
          await reload();
        } catch (e) {
          pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
        }
      },
    });
  }, [askConfirm, pushToast, reload, t]);

  const unlinkEpic = useCallback(() => {
    askConfirm({
      title: t("shop.epic.unlinkConfirm"),
      body: t("shop.epic.unlinkConfirmBody"),
      confirmLabel: t("shop.steam.unlink"),
      danger: true,
      onConfirm: async () => {
        try {
          await api.unlinkEpicAccount();
          pushToast({ kind: "success", title: t("shop.epic.unlinked") });
          await reload();
        } catch (e) {
          pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
        }
      },
    });
  }, [askConfirm, pushToast, reload, t]);

  return (
    <div className="p-5 max-w-3xl space-y-6 min-h-full">
      <h1 className="text-xl font-bold text-ink">{t("shop.title")}</h1>

      <StoreSection
        icon={<Gamepad2 className="h-5 w-5 text-[#66c0f4]" />}
        iconWrapClass="bg-[#1b2838] border-[#66c0f4]/25"
        title={t("shop.steam.title")}
        hint={t("shop.steam.hint")}
        libraryHint={t("shop.steam.library")}
        connectLabel={t("shop.steam.connect")}
        syncingLabel={t("shop.steam.syncing")}
        account={steamAccount}
        loading={loading}
        syncing={steamSyncing}
        onConnect={() => setSteamLinkOpen(true)}
        onSync={() => void syncSteam()}
        onUnlink={unlinkSteam}
      />

      <StoreSection
        icon={<Sparkles className="h-5 w-5 text-[#a78bfa]" />}
        iconWrapClass="bg-[#1f1b2e] border-[#a78bfa]/25"
        title={t("shop.epic")}
        hint={t("shop.epic.hint")}
        libraryHint={t("shop.epic.library")}
        connectLabel={t("shop.epic.connect")}
        syncingLabel={t("shop.epic.syncing")}
        account={epicAccount}
        loading={loading}
        syncing={epicSyncing}
        onConnect={() => setEpicLinkOpen(true)}
        onSync={() => void syncEpic()}
        onUnlink={unlinkEpic}
      />

      <div className="grid sm:grid-cols-2 gap-4">
        <SoonCard title={t("shop.gog")} />
      </div>

      <SteamLinkModal
        open={steamLinkOpen}
        onClose={() => setSteamLinkOpen(false)}
        onLinked={() => {
          void reload();
          void syncSteam();
        }}
      />

      <EpicLinkModal
        open={epicLinkOpen}
        onClose={() => setEpicLinkOpen(false)}
        onLinked={() => {
          void reload();
          void syncEpic();
        }}
      />
    </div>
  );
}

interface StoreSectionProps {
  icon: React.ReactNode;
  iconWrapClass: string;
  title: string;
  hint: string;
  libraryHint: string;
  connectLabel: string;
  syncingLabel: string;
  account: StoreAccount | null;
  loading: boolean;
  syncing: boolean;
  onConnect: () => void;
  onSync: () => void;
  onUnlink: () => void;
}

function StoreSection({
  icon,
  iconWrapClass,
  title,
  hint,
  libraryHint,
  connectLabel,
  syncingLabel,
  account,
  loading,
  syncing,
  onConnect,
  onSync,
  onUnlink,
}: StoreSectionProps) {
  const t = useT();
  const locale = useLocale();
  const [avatarBroken, setAvatarBroken] = useState<string | null>(null);

  return (
    <section className="glass rounded-2xl p-5">
      <div className="flex items-start gap-3">
        <span
          className={`h-10 w-10 shrink-0 rounded-xl border grid place-items-center ${iconWrapClass}`}
        >
          {icon}
        </span>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h2 className="text-[15px] font-semibold text-ink">{title}</h2>
            <Chip tone={account ? "success" : "default"}>
              {account ? t("shop.steam.connected") : t("shop.steam.notConnected")}
            </Chip>
          </div>
          <p className="text-[13px] text-muted mt-1 leading-relaxed">{hint}</p>
        </div>
      </div>

      {loading ? (
        <Skeleton className="mt-4 h-16 rounded-xl" />
      ) : account ? (
        <div className="mt-4 flex items-center justify-between gap-4 rounded-xl border border-white/8 bg-white/[0.04] p-4">
          <div className="flex items-center gap-3 min-w-0">
            {account.avatarUrl && avatarBroken !== account.avatarUrl ? (
              <img
                src={account.avatarUrl}
                alt=""
                className="h-10 w-10 rounded-full object-cover shrink-0"
                loading="lazy"
                onError={() => setAvatarBroken(account.avatarUrl)}
              />
            ) : (
              <span className="h-10 w-10 shrink-0 rounded-full bg-white/10 grid place-items-center text-sm font-semibold text-muted">
                {account.personaName.slice(0, 1).toUpperCase()}
              </span>
            )}
            <div className="min-w-0">
              <div className="text-sm font-medium text-ink truncate">{account.personaName}</div>
              <div className="text-xs text-dim truncate">
                {t("shop.steam.lastSync")}:{" "}
                {account.lastSyncedAt
                  ? formatDateTime(account.lastSyncedAt, locale)
                  : t("shop.steam.never")}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <Button
              size="sm"
              variant="primary"
              loading={syncing}
              icon={<RefreshCw className="h-3.5 w-3.5" />}
              onClick={onSync}
            >
              {syncing ? syncingLabel : t("shop.steam.sync")}
            </Button>
            <Button size="sm" variant="ghost" onClick={onUnlink}>
              {t("shop.steam.unlink")}
            </Button>
          </div>
        </div>
      ) : (
        <div className="mt-4 flex items-center justify-between gap-4">
          <p className="text-[13px] text-dim">{libraryHint}</p>
          <Button
            size="sm"
            variant="primary"
            icon={<Link2 className="h-3.5 w-3.5" />}
            onClick={onConnect}
          >
            {connectLabel}
          </Button>
        </div>
      )}
    </section>
  );
}

function SoonCard({ title }: { title: string }) {
  const t = useT();
  return (
    <div className="glass rounded-2xl p-5 flex items-center gap-3 opacity-70">
      <span className="h-10 w-10 shrink-0 rounded-xl bg-white/[0.06] border border-white/10 grid place-items-center">
        <Rocket className="h-5 w-5 text-dim" />
      </span>
      <div>
        <div className="text-[14px] font-medium text-ink">{title}</div>
        <div className="text-xs text-dim">{t("shop.soon")}</div>
      </div>
    </div>
  );
}
