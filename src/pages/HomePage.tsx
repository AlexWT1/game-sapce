import { Link } from "react-router-dom";
import { ChevronRight, Gamepad2, Plus, Trophy, Users } from "lucide-react";
import { HeroSection } from "@/components/home/HeroSection";
import { StatsPanel } from "@/components/home/StatsPanel";
import { GameGrid } from "@/components/game/GameGrid";
import { Button } from "@/components/ui/Button";
import { useT } from "@/hooks/useT";
import { useGamesStore } from "@/stores/gamesStore";
import { useUiStore } from "@/stores/uiStore";

export function HomePage() {
  const t = useT();
  const games = useGamesStore((s) => s.games);
  const loading = useGamesStore((s) => s.loading);
  const openAddGame = useUiStore((s) => s.openAddGame);

  const visible = games.filter((g) => !g.isHidden);
  const recent = visible
    .slice()
    .sort((a, b) => (b.lastPlayedAt ?? "").localeCompare(a.lastPlayedAt ?? ""))
    .slice(0, 8);
  const all = visible
    .slice()
    .sort((a, b) => a.title.localeCompare(b.title))
    .slice(0, 15);

  return (
    <div className="flex gap-5 p-5 min-h-full">
      <div className="flex-1 min-w-0 space-y-7">
        <HeroSection />

        <section>
          <SectionHeader title={t("home.recentGames")} to="/library?filter=recent" />
          <GameGrid games={recent} loading={loading} variant="recent" onAdd={() => openAddGame()} />
        </section>

        <section>
          <SectionHeader
            title={t("home.allGames")}
            to="/library"
            action={
              <Button
                size="sm"
                variant="outline"
                onClick={() => openAddGame()}
                icon={<Plus className="h-3.5 w-3.5" />}
              >
                {t("library.addGame")}
              </Button>
            }
          />
          <GameGrid games={all} loading={loading} onAdd={() => openAddGame()} />
        </section>

        <div className="grid sm:grid-cols-3 gap-4 pb-4">
          <QuickLink to="/achievements" icon={<Trophy className="h-5 w-5 text-amber-400" />} label={t("nav.achievements")} />
          <QuickLink to="/statistics" icon={<Gamepad2 className="h-5 w-5 text-sky-400" />} label={t("nav.statistics")} />
          <QuickLink to="/friends" icon={<Users className="h-5 w-5 text-emerald-400" />} label={t("nav.friends")} />
        </div>
      </div>

      <StatsPanel />
    </div>
  );
}

function SectionHeader({
  title,
  to,
  action,
}: {
  title: string;
  to: string;
  action?: React.ReactNode;
}) {
  const t = useT();
  return (
    <div className="flex items-center justify-between mb-3.5">
      <h2 className="text-[16px] font-semibold text-ink">{title}</h2>
      <div className="flex items-center gap-3">
        {action}
        <Link
          to={to}
          className="inline-flex items-center gap-1 text-[12.5px] text-muted hover:text-ink transition"
        >
          {t("home.viewAll")} <ChevronRight className="h-3.5 w-3.5" />
        </Link>
      </div>
    </div>
  );
}

function QuickLink({ to, icon, label }: { to: string; icon: React.ReactNode; label: string }) {
  return (
    <Link
      to={to}
      className="glass rounded-2xl px-4 py-3.5 flex items-center gap-3 card-hover"
    >
      {icon}
      <span className="text-[13.5px] font-medium text-ink">{label}</span>
      <ChevronRight className="ml-auto h-4 w-4 text-dim" />
    </Link>
  );
}
