import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Check, Gamepad2, Heart, MoreHorizontal, Clock } from "lucide-react";
import { ContextMenu, type ContextMenuItem } from "@/components/ui/ContextMenu";
import { Chip } from "@/components/ui/Chip";
import { useT } from "@/hooks/useT";
import { useUiStore } from "@/stores/uiStore";
import { useGamesStore } from "@/stores/gamesStore";
import { useSessionStore } from "@/stores/sessionStore";
import { cx, formatPlaytime, formatHMS, relativeTime } from "@/utils/format";
import { canLaunch } from "@/utils/game";
import type { Game } from "@/types";

interface GameCardProps {
  game: Game;
  variant?: "grid" | "list" | "recent";
  onPlay?: (game: Game) => void;
  selecting?: boolean;
  selected?: boolean;
  onToggleSelect?: () => void;
}

function SelectMark({ checked }: { checked: boolean }) {
  return (
    <span
      className={cx(
        "h-5 w-5 rounded-md grid place-items-center border transition",
        checked
          ? "bg-[var(--color-accent)] border-[var(--color-accent)] text-white"
          : "bg-black/60 border-white/40 text-transparent backdrop-blur",
      )}
    >
      <Check className="h-3.5 w-3.5" strokeWidth={3} />
    </span>
  );
}

export function buildGameMenu(game: Game, t: ReturnType<typeof useT>): ContextMenuItem[] {
  const ui = useUiStore.getState();
  const games = useGamesStore.getState();
  const session = useSessionStore.getState();

  return [
    {
      key: "play",
      label: session.isRunning(game.id) ? t("game.stop") : t("game.play"),
      icon: <Clock className="h-3.5 w-3.5" />,
      onSelect: () => {
        if (session.isRunning(game.id)) {
          void session.stop(game.id);
        } else if (canLaunch(game)) {
          void session
            .launch(game.id)
            .then(() => games.load(true))
            .catch((e) => {
              ui.pushToast({
                kind: "error",
                title: t("dialog.missingExe"),
                body: String(e),
              });
            });
        } else {
          ui.openMissingExe(game);
        }
      },
    },
    {
      key: "open-folder",
      label: t("game.openFolder"),
      onSelect: () => {
        const path = game.installPath ?? game.executablePath;
        if (path) {
          void import("@/services/api").then(({ api }) => api.openPath(path));
        }
      },
    },
    {
      key: "edit",
      label: t("game.edit"),
      onSelect: () => ui.openEditGame(game),
    },
    {
      key: "favorite",
      label: game.isFavorite ? t("game.unfavorite") : t("game.favorite"),
      onSelect: () => void games.toggleFavorite(game.id),
    },
    {
      key: "hide",
      label: game.isHidden ? t("game.unhide") : t("game.hide"),
      onSelect: () =>
        void games.update(game.id, { isHidden: !game.isHidden }),
    },
    {
      key: "delete",
      label: t("game.delete"),
      danger: true,
      onSelect: () =>
        ui.askConfirm({
          title: t("dialog.deleteTitle"),
          body: t("dialog.deleteBody"),
          danger: true,
          confirmLabel: t("dialog.confirm"),
          onConfirm: async () => {
            try {
              await games.remove(game.id);
              ui.pushToast({ kind: "success", title: game.title });
            } catch (e) {
              ui.pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
            }
          },
        }),
    },
  ];
}

export function GameCard({
  game,
  variant = "grid",
  onPlay,
  selecting,
  selected,
  onToggleSelect,
}: GameCardProps) {
  const t = useT();
  const navigate = useNavigate();
  const running = useSessionStore((s) => s.running.some((r) => r.gameId === game.id));
  const live = useSessionStore((s) => s.liveSeconds[game.id] ?? 0);
  const toggleFavorite = useGamesStore((s) => s.toggleFavorite);
  const openEdit = useUiStore((s) => s.openEditGame);

  const menu = buildGameMenu(game, t);
  const [brokenCover, setBrokenCover] = useState<string | null>(null);
  const onCoverError = () => setBrokenCover(game.coverPath);

  const go = () => {
    if (selecting) {
      onToggleSelect?.();
      return;
    }
    navigate(`/game/${game.id}`);
  };

  if (variant === "recent") {
    return (
      <ContextMenu items={menu}>
        <button
          onClick={() => navigate(`/game/${game.id}`)}
          className="group relative w-[168px] shrink-0 text-left"
        >
          <div className="relative aspect-[3/4] rounded-2xl overflow-hidden border border-white/8 card-hover bg-white/5">
            {game.coverPath && brokenCover !== game.coverPath ? (
              <img
                src={game.coverPath}
                alt={game.title}
                className={cx(
                  "h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.04]",
                  !game.isInstalled && "opacity-60 saturate-[0.65]",
                )}
                loading="lazy"
                onError={onCoverError}
              />
            ) : (
              <div className="h-full w-full grid place-items-center bg-gradient-to-br from-[#1a1f3a] to-[#0d1020]">
                <Gamepad2 className="h-8 w-8 text-dim" />
              </div>
            )}
            <div className="absolute inset-x-0 bottom-0 h-2/5 bg-gradient-to-t from-black/90 to-transparent" />
            {game.isFavorite && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  void toggleFavorite(game.id);
                }}
                className="absolute top-2 right-2 p-1.5 rounded-lg bg-black/50 backdrop-blur text-rose-400 hover:text-rose-300 transition"
                aria-label={t("game.favorite")}
              >
                <Heart className="h-3.5 w-3.5 fill-current" />
              </button>
            )}
            <div className="absolute left-2.5 bottom-2.5 right-2.5">
              <p className="text-[12.5px] font-semibold text-white truncate drop-shadow">
                {game.title}
              </p>
            </div>
            {running && (
              <div className="absolute top-2 left-2 flex items-center gap-1.5 px-2 py-1 rounded-full bg-black/60 backdrop-blur border border-emerald-400/30">
                <span className="live-dot" />
                <span className="text-[10px] font-medium text-emerald-300">
                  {t("home.statusPlaying")}
                </span>
              </div>
            )}
          </div>
          <div className="mt-2 flex items-center justify-between gap-2 px-0.5">
            <span className="text-[11px] text-muted flex items-center gap-1">
              {running ? (
                <>
                  <span className="live-dot" /> {formatHMS(live)}
                </>
              ) : (
                <>
                  <Clock className="h-3 w-3" /> {formatPlaytime(game.totalPlaytimeSeconds)}
                </>
              )}
            </span>
          </div>
        </button>
      </ContextMenu>
    );
  }

  if (variant === "list") {
    return (
      <ContextMenu items={menu}>
        <div
          role="button"
          tabIndex={0}
          onClick={go}
          onKeyDown={(e) => e.key === "Enter" && go()}
          className={cx(
            "group flex items-center gap-4 px-3.5 py-3 rounded-2xl card card-hover cursor-pointer",
            selecting && (selected ? "ring-2 ring-[var(--color-accent)]" : "opacity-60"),
          )}
        >
          {selecting && (
            <span className="shrink-0">
              <SelectMark checked={!!selected} />
            </span>
          )}
          <div className="h-14 w-11 rounded-lg overflow-hidden bg-white/5 shrink-0">
            {game.coverPath && brokenCover !== game.coverPath ? (
              <img
                src={game.coverPath}
                alt=""
                className={cx(
                  "h-full w-full object-cover",
                  !game.isInstalled && "opacity-60 saturate-[0.65]",
                )}
                loading="lazy"
                onError={onCoverError}
              />
            ) : (
              <div className="h-full w-full grid place-items-center">
                <Gamepad2 className="h-5 w-5 text-dim" />
              </div>
            )}
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <p className="text-[14px] font-semibold text-ink truncate">{game.title}</p>
              {game.isFavorite && <Heart className="h-3.5 w-3.5 text-rose-400 fill-current shrink-0" />}
            </div>
            <p className="text-[12px] text-muted truncate mt-0.5">
              {[game.genre, game.developer, game.platform].filter(Boolean).join(" · ")}
            </p>
          </div>
          <div className="hidden md:flex items-center gap-4 text-[12px] text-muted">
            <span className="w-28 text-right">{formatPlaytime(game.totalPlaytimeSeconds)}</span>
            <span className="w-24 text-right text-dim">
              {game.lastPlayedAt ? relativeTime(game.lastPlayedAt) : "—"}
            </span>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            {running ? (
              <Chip tone="success">
                <span className="live-dot" /> {t("home.statusPlaying")}
              </Chip>
            ) : game.isInstalled ? (
              <Chip tone="default">{t("game.installed")}</Chip>
            ) : (
              <Chip tone="danger">
                <span className="h-1.5 w-1.5 rounded-full bg-red-400" />
                {t("game.notInstalled")}
              </Chip>
            )}
            <button
              onClick={(e) => {
                e.stopPropagation();
                openEdit(game);
              }}
              className="p-1.5 rounded-lg text-dim hover:text-ink hover:bg-white/10 transition"
              aria-label={t("game.edit")}
            >
              <MoreHorizontal className="h-4 w-4" />
            </button>
          </div>
        </div>
      </ContextMenu>
    );
  }

  // grid
  return (
    <ContextMenu items={menu}>
      <div
        role="button"
        tabIndex={0}
        onClick={go}
        onKeyDown={(e) => e.key === "Enter" && go()}
        className={cx(
          "group relative rounded-2xl overflow-hidden border border-white/8 bg-white/[0.03] card-hover cursor-pointer",
          selecting && (selected ? "ring-2 ring-[var(--color-accent)]" : "opacity-60"),
        )}
        onDoubleClick={() => !selecting && onPlay?.(game)}
      >
        <div className="aspect-[3/4] relative">
          {game.coverPath && brokenCover !== game.coverPath ? (
            <img
              src={game.coverPath}
              alt={game.title}
                className={cx(
                  "h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.03]",
                  !game.isInstalled && "opacity-60 saturate-[0.65]",
                )}
              loading="lazy"
              onError={onCoverError}
            />
          ) : (
            <div className="h-full w-full grid place-items-center bg-gradient-to-br from-[#1a1f3a] to-[#0d1020]">
              <Gamepad2 className="h-10 w-10 text-dim" />
            </div>
          )}
          <div className="absolute inset-x-0 bottom-0 h-1/2 bg-gradient-to-t from-[#070912] via-[#070912]/70 to-transparent" />

          {running && (
            <div className="absolute top-2.5 left-2.5 flex items-center gap-1.5 px-2 py-1 rounded-full bg-black/60 backdrop-blur border border-emerald-400/30">
              <span className="live-dot" />
              <span className="text-[10px] font-medium text-emerald-300">
                {t("home.statusPlaying")}
              </span>
            </div>
          )}

          {selecting ? (
            <span className="absolute top-2.5 right-2.5">
              <SelectMark checked={!!selected} />
            </span>
          ) : (
            <button
              onClick={(e) => {
                e.stopPropagation();
                void toggleFavorite(game.id);
              }}
              className={cx(
                "absolute top-2.5 right-2.5 p-1.5 rounded-lg bg-black/50 backdrop-blur transition",
                game.isFavorite
                  ? "text-rose-400"
                  : "text-white/60 opacity-0 group-hover:opacity-100 hover:text-rose-300",
              )}
              aria-label={t("game.favorite")}
            >
              <Heart className={cx("h-3.5 w-3.5", game.isFavorite && "fill-current")} />
            </button>
          )}

          <div className="absolute inset-x-3 bottom-2.5">
            <p className="text-[13px] font-semibold text-ink truncate leading-tight">
              {game.title}
            </p>
            <div className="mt-1.5 flex flex-col gap-0.5 text-[11px] text-muted">
              <span className="flex items-center gap-1.5">
                <span
                  className={cx(
                    "h-2 w-2 rounded-full shrink-0",
                    game.isInstalled
                      ? "bg-emerald-400"
                      : "bg-red-500 shadow-[0_0_6px_rgba(239,68,68,0.8)]",
                  )}
                />
                {game.isInstalled ? t("game.installed") : t("game.notInstalled")}
              </span>
              <span className="flex items-center gap-1 text-dim">
                <Clock className="h-3 w-3" />
                {formatPlaytime(game.totalPlaytimeSeconds)}
              </span>
            </div>
          </div>
        </div>
      </div>
    </ContextMenu>
  );
}
