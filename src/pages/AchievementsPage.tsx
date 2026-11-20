import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { Lock, Trophy } from "lucide-react";
import { Chip } from "@/components/ui/Chip";
import { ProgressBar } from "@/components/ui/ProgressBar";
import { EmptyState } from "@/components/ui/EmptyState";
import { Skeleton } from "@/components/ui/Skeleton";
import { useT, useLocaleCode } from "@/hooks/useT";
import { useAchievementsStore } from "@/stores/achievementsStore";
import { cx, formatDateTime, percent } from "@/utils/format";
import type { Achievement, Rarity } from "@/types";

type ScopeFilter = "all" | "app" | "game";

export function AchievementsPage() {
  const t = useT();
  const locale = useLocaleCode();
  const items = useAchievementsStore((s) => s.items);
  const loading = useAchievementsStore((s) => s.loading);
  const load = useAchievementsStore((s) => s.load);
  const [scope, setScope] = useState<ScopeFilter>("all");

  useEffect(() => {
    void load(null, true);
  }, [load]);

  const unlocked = items.filter((a) => a.unlocked).length;
  const total = items.length;
  const pct = percent(unlocked, total);

  const filtered = useMemo(() => {
    if (scope === "all") return items;
    return items.filter((a) => a.scope === scope);
  }, [items, scope]);

  const recentUnlocked = filtered
    .filter((a) => a.unlocked && a.unlockedAt)
    .sort((a, b) => (b.unlockedAt ?? "").localeCompare(a.unlockedAt ?? ""))
    .slice(0, 4);

  return (
    <div className="p-5 space-y-5 min-h-full">
      <div className="flex flex-wrap items-center gap-3">
        <h1 className="text-xl font-bold text-ink">{t("achievements.title")}</h1>
        <div className="flex gap-1.5 ml-2">
          {(
            [
              ["all", t("achievements.all")],
              ["app", t("achievements.app")],
              ["game", t("achievements.games")],
            ] as Array<[ScopeFilter, string]>
          ).map(([key, label]) => (
            <button key={key} onClick={() => setScope(key)}>
              <Chip tone={scope === key ? "accent" : "default"}>{label}</Chip>
            </button>
          ))}
        </div>
        <div className="ml-auto flex items-center gap-3 text-[13px]">
          <span className="text-muted">
            {t("achievements.unlocked")}{" "}
            <span className="text-ink font-semibold tabular-nums">
              {unlocked} {t("achievements.of")} {total}
            </span>
          </span>
          <div className="w-36">
            <ProgressBar value={pct} height={6} />
          </div>
          <span className="text-ink font-semibold tabular-nums w-10 text-right">{pct}%</span>
        </div>
      </div>

      {recentUnlocked.length > 0 && (
        <section>
          <h2 className="text-[14px] font-semibold text-ink mb-3">{t("achievements.recent")}</h2>
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
            {recentUnlocked.map((a) => (
              <div
                key={a.id}
                className="glass rounded-2xl p-4 border-amber-400/20 flex items-start gap-3"
              >
                <div className="h-11 w-11 rounded-xl bg-amber-500/15 border border-amber-400/25 grid place-items-center shrink-0">
                  <Trophy className="h-5 w-5 text-amber-300" />
                </div>
                <div className="min-w-0">
                  <p className="text-[13.5px] font-semibold text-ink truncate">{a.title}</p>
                  <p className="text-[11.5px] text-muted truncate">
                    {a.gameTitle ?? t("achievements.app")}
                  </p>
                  <p className="text-[11px] text-dim mt-1">
                    {formatDateTime(a.unlockedAt, locale)}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      <section>
        <h2 className="text-[14px] font-semibold text-ink mb-3">{t("achievements.all")}</h2>

        {loading && items.length === 0 ? (
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-24 rounded-2xl" />
            ))}
          </div>
        ) : filtered.length === 0 ? (
          <EmptyState
            icon={<Trophy className="h-6 w-6" />}
            title={t("achievements.empty")}
          />
        ) : (
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
            {filtered.map((a) => (
              <div
                key={a.id}
                className={cx(
                  "glass rounded-2xl p-4 flex items-start gap-3.5 transition",
                  !a.unlocked && "opacity-60",
                )}
                title={a.unlocked ? undefined : a.description}
              >
                <div
                  className={cx(
                    "h-12 w-12 rounded-xl grid place-items-center shrink-0 border",
                    a.unlocked
                      ? rarityStyles(a.rarity).box
                      : "bg-white/5 border-white/10 text-dim",
                  )}
                >
                  {a.unlocked ? (
                    <Trophy className={cx("h-5 w-5", rarityStyles(a.rarity).icon)} />
                  ) : (
                    <Lock className="h-5 w-5" />
                  )}
                </div>

                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <p className="text-[13.5px] font-semibold text-ink truncate">{a.title}</p>
                    <Chip tone={rarityStyles(a.rarity).tone as never} className="shrink-0">
                      {t(`achievements.rarity.${a.rarity as Rarity}` as never)}
                    </Chip>
                  </div>
                  <p className="text-[12px] text-muted mt-0.5 line-clamp-2">{a.description}</p>

                  {a.unlocked ? (
                    <p className="text-[11.5px] text-emerald-400/90 mt-1.5 flex items-center gap-1.5">
                      <span className="inline-block h-1.5 w-1.5 rounded-full bg-emerald-400" />
                      {t("achievements.unlocked")}
                      {a.gameTitle && (
                        <span className="text-dim">
                          ·{" "}
                          <Link to={`/game/${a.gameId}`} className="hover:text-ink underline-offset-2 hover:underline">
                            {a.gameTitle}
                          </Link>
                        </span>
                      )}
                      {a.unlockedAt && (
                        <span className="text-dim">· {formatDateTime(a.unlockedAt, locale)}</span>
                      )}
                    </p>
                  ) : (
                    <div className="mt-2">
                      <ProgressBar
                        value={a.progressCurrent}
                        max={Math.max(a.progressTarget, 1)}
                        height={4}
                        color="#6b7189"
                      />
                      <span className="text-[11px] text-dim tabular-nums">
                        {progressLabel(a)}
                      </span>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

function progressLabel(a: Achievement) {
  if (a.key?.startsWith("playtime_")) {
    return `${a.progressCurrent} ч / ${a.progressTarget} ч`;
  }
  return `${a.progressCurrent} / ${a.progressTarget}`;
}

function rarityStyles(rarity: string) {
  switch (rarity) {
    case "Rare":
      return { box: "bg-sky-500/12 border-sky-400/30", icon: "text-sky-300", tone: "default" };
    case "Epic":
      return { box: "bg-purple-500/12 border-purple-400/35", icon: "text-purple-300", tone: "epic" };
    case "Legendary":
      return {
        box: "bg-orange-500/12 border-orange-400/35",
        icon: "text-orange-300",
        tone: "legendary",
      };
    default:
      return { box: "bg-amber-500/12 border-amber-400/30", icon: "text-amber-300", tone: "warning" };
  }
}
