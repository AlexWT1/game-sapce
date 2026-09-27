import { useCallback, useEffect, useRef, useState } from "react";
import { Gamepad2, Link2, RefreshCw, Rocket } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Chip } from "@/components/ui/Chip";
import { Skeleton } from "@/components/ui/Skeleton";
import { SteamLinkModal } from "@/features/shop/SteamLinkModal";
import { useLocale, useT } from "@/hooks/useT";
import { useGamesStore } from "@/stores/gamesStore";
import { useUiStore } from "@/stores/uiStore";
import { api } from "@/services/api";
import { formatDateTime } from "@/utils/format";
import type { StoreAccount } from "@/types";

export function ShopPage() {
  const t = useT();
  const locale = useLocale();
  const pushToast = useUiStore((s) => s.pushToast);
  const askConfirm = useUiStore((s) => s.askConfirm);

  const [account, setAccount] = useState<StoreAccount | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [linkOpen, setLinkOpen] = useState(false);
  const [avatarBroken, setAvatarBroken] = useState<string | null>(null);
  const syncRef = useRef(false);

  const reload = useCallback(async () => {
    try {
      setAccount(await api.getStoreAccount());
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    } finally {
      setLoading(false);
    }
  }, [pushToast, t]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const sync = useCallback(async () => {
    if (syncRef.current) return;
    syncRef.current = true;
    setSyncing(true);
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
      syncRef.current = false;
      setSyncing(false);
    }
  }, [pushToast, reload, t]);

  const unlink = useCallback(() => {
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

  return (
    <div className="p-5 max-w-3xl space-y-6 min-h-full">
      <h1 className="text-xl font-bold text-ink">{t("shop.title")}</h1>

      <section className="glass rounded-2xl p-5">
        <div className="flex items-start gap-3">
          <span className="h-10 w-10 shrink-0 rounded-xl bg-[#1b2838] border border-[#66c0f4]/25 grid place-items-center">
            <Gamepad2 className="h-5 w-5 text-[#66c0f4]" />
          </span>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <h2 className="text-[15px] font-semibold text-ink">{t("shop.steam.title")}</h2>
              <Chip tone={account ? "success" : "default"}>
                {account ? t("shop.steam.connected") : t("shop.steam.notConnected")}
              </Chip>
            </div>
            <p className="text-[13px] text-muted mt-1 leading-relaxed">{t("shop.steam.hint")}</p>
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
                onClick={() => void sync()}
              >
                {syncing ? t("shop.steam.syncing") : t("shop.steam.sync")}
              </Button>
              <Button size="sm" variant="ghost" onClick={unlink}>
                {t("shop.steam.unlink")}
              </Button>
            </div>
          </div>
        ) : (
          <div className="mt-4 flex items-center justify-between gap-4">
            <p className="text-[13px] text-dim">{t("shop.steam.library")}</p>
            <Button
              size="sm"
              variant="primary"
              icon={<Link2 className="h-3.5 w-3.5" />}
              onClick={() => setLinkOpen(true)}
            >
              {t("shop.steam.connect")}
            </Button>
          </div>
        )}
      </section>

      <div className="grid sm:grid-cols-2 gap-4">
        <SoonCard title={t("shop.epic")} />
        <SoonCard title={t("shop.gog")} />
      </div>

      <SteamLinkModal
        open={linkOpen}
        onClose={() => setLinkOpen(false)}
        onLinked={() => {
          void reload();
          void sync();
        }}
      />
    </div>
  );
}

function SoonCard({ title }: { title: string }) {
  const t = useT();
  return (
    <div className="glass rounded-2xl p-5 flex items-center gap-3 opacity-70">
      <span className="h-10 w-10 shrink-0 rounded-xl bg-white/[0.06] border border-white/10 grid place-items-center">
        <Rocket className="h-5 w-5 text-dim" />
      </span>
      <div className="min-w-0">
        <div className="text-[14px] font-medium text-ink truncate">{title}</div>
        <Chip tone="default">{t("shop.soon")}</Chip>
      </div>
    </div>
  );
}
