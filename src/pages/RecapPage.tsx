import { useState } from "react";
import { Sparkles } from "lucide-react";
import { Link } from "react-router-dom";
import { RecapSlideshow } from "@/features/recap/RecapSlideshow";
import { useRecapData, type RecapPeriod } from "@/features/recap/hooks/useRecapData";
import { EmptyState } from "@/components/ui/EmptyState";
import { Skeleton } from "@/components/ui/Skeleton";
import { useT } from "@/hooks/useT";

export function RecapPage() {
  const t = useT();
  const [period, setPeriod] = useState<RecapPeriod>("month");
  const { data, loading } = useRecapData(period);

  if (loading && !data) {
    return (
      <div className="p-5 space-y-4">
        <Skeleton className="h-10 w-56" />
        <div className="grid sm:grid-cols-2 xl:grid-cols-4 gap-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <Skeleton key={i} className="h-28 rounded-2xl" />
          ))}
        </div>
        <Skeleton className="h-64 rounded-2xl" />
      </div>
    );
  }

  if (!data || data.gamesCount === 0) {
    return (
      <div className="p-5">
        <EmptyState
          icon={<Sparkles className="h-6 w-6" />}
          title={t("recap.empty")}
          body={t("recap.emptyHint")}
          action={
            <Link to="/library" className="text-[var(--color-accent)] text-sm hover:underline">
              {t("nav.library")} →
            </Link>
          }
        />
      </div>
    );
  }

  return <RecapSlideshow data={data} period={period} onPeriodChange={setPeriod} />;
}
