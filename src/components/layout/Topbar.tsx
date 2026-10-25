import { useEffect, useRef } from "react";
import {
  Bell,
  PanelLeftClose,
  PanelLeftOpen,
  Search,
  Settings,
  User,
} from "lucide-react";
import { useNavigate } from "react-router-dom";
import { useT } from "@/hooks/useT";
import { useUiStore } from "@/stores/uiStore";
import { useSettingsStore } from "@/stores/settingsStore";
import { usePlayerDisplayName } from "@/stores/playerStore";
import { WindowControls } from "@/components/layout/WindowControls";
import { cx, relativeTime } from "@/utils/format";

export function Topbar() {
  const t = useT();
  const navigate = useNavigate();
  const collapsed = useUiStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);
  const setPalette = useUiStore((s) => s.setCommandPalette);
  const notifOpen = useUiStore((s) => s.notificationsOpen);
  const setNotifOpen = useUiStore((s) => s.setNotificationsOpen);
  const notifications = useUiStore((s) => s.recentNotifications);
  const locale = useSettingsStore((s) => s.locale);
  const playerName = usePlayerDisplayName();
  const boxRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!notifOpen) return;
    const onDoc = (e: MouseEvent) => {
      if (boxRef.current && !boxRef.current.contains(e.target as Node)) {
        setNotifOpen(false);
      }
    };
    window.addEventListener("mousedown", onDoc);
    return () => window.removeEventListener("mousedown", onDoc);
  }, [notifOpen, setNotifOpen]);

  return (
    <header
      data-tauri-drag-region="deep"
      className="relative z-30 h-[var(--topbar-height)] shrink-0 border-b border-white/8 bg-[#0a0c18]/70 backdrop-blur-xl flex items-center gap-3 px-4"
    >
      <button
        onClick={toggleSidebar}
        className="h-9 w-9 grid place-items-center rounded-lg text-muted hover:text-ink hover:bg-white/8 transition"
        aria-label="Toggle sidebar"
      >
        {collapsed ? (
          <PanelLeftOpen className="h-4.5 w-4.5 h-[18px] w-[18px]" />
        ) : (
          <PanelLeftClose className="h-[18px] w-[18px]" />
        )}
      </button>

      <div className="flex-1 flex justify-center min-w-0 px-2">
        <button
          onClick={() => setPalette(true)}
          className={cx(
            "group w-full max-w-[520px] h-10 flex items-center gap-2.5 px-3.5 rounded-xl",
            "bg-white/[0.05] border border-white/10 text-muted hover:border-white/20 transition",
          )}
        >
          <Search className="h-4 w-4 shrink-0" />
          <span className="text-[13.5px] truncate">{t("topbar.search")}</span>
          <kbd className="ml-auto hidden sm:inline-flex items-center gap-0.5 rounded-md border border-white/10 bg-white/5 px-1.5 py-0.5 text-[10px] text-dim font-medium">
            Ctrl K
          </kbd>
        </button>
      </div>

      <div className="flex items-center gap-1.5">
        <div className="relative" ref={boxRef}>
          <button
            onClick={() => setNotifOpen(!notifOpen)}
            className="relative h-9 w-9 grid place-items-center rounded-lg text-muted hover:text-ink hover:bg-white/8 transition"
            aria-label={t("topbar.notifications")}
          >
            <Bell className="h-[18px] w-[18px]" />
            {notifications.length > 0 && (
              <span className="absolute top-1.5 right-1.5 h-2 w-2 rounded-full bg-[var(--color-accent)] shadow-[0_0_6px_var(--color-accent)]" />
            )}
          </button>
          {notifOpen && (
            <div
              data-tauri-drag-region="false"
              className="absolute right-0 top-11 w-80 max-h-96 overflow-y-auto rounded-xl border border-white/10 bg-[#12152b]/95 backdrop-blur-xl shadow-2xl p-2 z-50"
            >
              {notifications.length === 0 ? (
                <p className="text-[13px] text-dim text-center py-6">{t("common.nothing")}</p>
              ) : (
                notifications.map((n) => (
                  <div key={n.id} className="px-3 py-2.5 rounded-lg hover:bg-white/5 transition">
                    <p className="text-[13px] font-medium text-ink">{n.title}</p>
                    {n.body && <p className="text-[12px] text-muted mt-0.5">{n.body}</p>}
                    <p className="text-[11px] text-dim mt-1">{relativeTime(n.at, locale)}</p>
                  </div>
                ))
              )}
            </div>
          )}
        </div>

        <button
          onClick={() => navigate("/settings")}
          className="h-9 w-9 grid place-items-center rounded-lg text-muted hover:text-ink hover:bg-white/8 transition"
          aria-label={t("topbar.settings")}
        >
          <Settings className="h-[18px] w-[18px]" />
        </button>

        <button
          onClick={() => navigate("/settings")}
          className="flex items-center gap-2 h-9 pl-1 pr-2.5 rounded-full hover:bg-white/8 transition"
          aria-label={t("topbar.profile")}
        >
          <span className="h-7 w-7 rounded-full bg-gradient-to-br from-[#7c5cff] to-[#22d3ee] grid place-items-center">
            <User className="h-3.5 w-3.5 text-white" />
          </span>
          <span className="text-[13px] font-medium text-ink hidden md:inline">
            {playerName}
          </span>
        </button>

        <div className="w-px h-6 bg-white/10 mx-1" />
        <WindowControls />
      </div>
    </header>
  );
}
