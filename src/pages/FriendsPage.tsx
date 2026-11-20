import { useEffect, useState } from "react";
import { Users } from "lucide-react";
import { EmptyState } from "@/components/ui/EmptyState";
import { Skeleton } from "@/components/ui/Skeleton";
import { Chip } from "@/components/ui/Chip";
import { useT, useLocaleCode } from "@/hooks/useT";
import { fetchFriends } from "@/services/friends";
import { useSettingsStore } from "@/stores/settingsStore";
import { relativeTime } from "@/utils/format";
import type { Friend } from "@/types";

export function FriendsPage() {
  const t = useT();
  const locale = useLocaleCode();
  const demoMode = useSettingsStore((s) => s.demoMode);
  const [friends, setFriends] = useState<Friend[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let alive = true;
    setLoading(true);
    void fetchFriends().then((f) => {
      if (alive) {
        setFriends(f);
        setLoading(false);
      }
    });
    return () => {
      alive = false;
    };
  }, [demoMode]);

  const online = friends.filter((f) => f.online);
  const offline = friends.filter((f) => !f.online);

  return (
    <div className="p-5 space-y-5 min-h-full">
      <div className="flex items-center gap-3">
        <h1 className="text-xl font-bold text-ink">{t("friends.title")}</h1>
        <Chip tone="success">
          {t("friends.online")}: {online.length}
        </Chip>
        <span className="text-[12.5px] text-dim ml-auto">
          {t("friends.hint")}
        </span>
      </div>

      {loading ? (
        <div className="space-y-3">
          {Array.from({ length: 4 }).map((_, i) => (
            <Skeleton key={i} className="h-[72px] rounded-2xl" />
          ))}
        </div>
      ) : friends.length === 0 ? (
        <EmptyState icon={<Users className="h-6 w-6" />} title={t("friends.empty")} body={t("friends.hint")} />
      ) : (
        <div className="grid gap-4 lg:grid-cols-2">
          <section>
            <h2 className="text-[13px] font-semibold text-muted uppercase tracking-wide mb-3">
              {t("friends.online")}
            </h2>
            <ul className="space-y-2.5">
              {online.map((f) => (
                <FriendRow key={f.id} friend={f} locale={locale} t={t} />
              ))}
              {online.length === 0 && (
                <li className="text-[13px] text-dim py-4">{t("common.nothing")}</li>
              )}
            </ul>
          </section>

          <section>
            <h2 className="text-[13px] font-semibold text-muted uppercase tracking-wide mb-3">
              {t("friends.offline")}
            </h2>
            <ul className="space-y-2.5">
              {offline.map((f) => (
                <FriendRow key={f.id} friend={f} locale={locale} t={t} />
              ))}
            </ul>
          </section>
        </div>
      )}
    </div>
  );
}

function FriendRow({
  friend,
  locale,
  t,
}: {
  friend: Friend;
  locale: string;
  t: ReturnType<typeof useT>;
}) {
  const initial = friend.name.slice(0, 1).toUpperCase();
  const hue = [...friend.name].reduce((a, c) => a + c.charCodeAt(0), 0) % 360;

  return (
    <li className="glass rounded-2xl px-4 py-3.5 flex items-center gap-3.5">
      <div className="relative shrink-0">
        <span
          className="h-11 w-11 rounded-full grid place-items-center text-[15px] font-bold text-white/90"
          style={{ background: `linear-gradient(135deg, hsl(${hue} 60% 40%), hsl(${(hue + 40) % 360} 70% 50%))` }}
        >
          {initial}
        </span>
        <span
          className={`absolute -bottom-0.5 -right-0.5 h-3.5 w-3.5 rounded-full border-2 border-[#0d1020] ${
            friend.online ? "bg-emerald-400" : "bg-white/30"
          }`}
        />
      </div>

      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <p className="text-[14px] font-semibold text-ink truncate">{friend.name}</p>
          {friend.online && friend.currentGame && (
            <Chip tone="success">{t("home.statusPlaying")}</Chip>
          )}
        </div>
        <p className="text-[12.5px] text-muted truncate mt-0.5">
          {friend.online
            ? friend.currentGame
              ? `${t("friends.playing")} ${friend.currentGame}`
              : t("time.online")
            : `${t("friends.lastSeen")} ${relativeTime(friend.lastSeen, locale)}`}
        </p>
      </div>
    </li>
  );
}
