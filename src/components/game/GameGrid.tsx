import { GameCard } from "@/components/game/GameCard";
import { GameGridSkeleton } from "@/components/ui/Skeleton";
import { EmptyState } from "@/components/ui/EmptyState";
import { Button } from "@/components/ui/Button";
import { useT } from "@/hooks/useT";
import { Gamepad2, Plus } from "lucide-react";
import type { Game } from "@/types";

interface GameGridProps {
  games: Game[];
  loading?: boolean;
  variant?: "grid" | "list" | "recent";
  emptyAction?: React.ReactNode;
  onAdd?: () => void;
}

export function GameGrid({ games, loading, variant = "grid", onAdd }: GameGridProps) {
  const t = useT();

  if (loading && games.length === 0) {
    return <GameGridSkeleton count={variant === "recent" ? 6 : 10} />;
  }

  if (games.length === 0) {
    return (
      <EmptyState
        icon={<Gamepad2 className="h-6 w-6" />}
        title={t("library.empty")}
        body={t("library.emptyHint")}
        action={
          onAdd ? (
            <Button onClick={onAdd} icon={<Plus className="h-4 w-4" />}>
              {t("library.addGame")}
            </Button>
          ) : undefined
        }
      />
    );
  }

  if (variant === "recent") {
    return (
      <div className="flex gap-4 overflow-x-auto no-scrollbar pb-1 -mx-1 px-1">
        {games.map((g) => (
          <GameCard key={g.id} game={g} variant="recent" />
        ))}
      </div>
    );
  }

  if (variant === "list") {
    return (
      <div className="space-y-2.5">
        {games.map((g) => (
          <GameCard key={g.id} game={g} variant="list" />
        ))}
      </div>
    );
  }

  return (
    <div className="grid gap-4 grid-cols-[repeat(auto-fill,minmax(160px,1fr))]">
      {games.map((g) => (
        <GameCard key={g.id} game={g} variant="grid" />
      ))}
    </div>
  );
}
