import { X } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { useT } from "@/hooks/useT";
import { RecapCard } from "../RecapCard";
import type { RecapData } from "../../hooks/useRecapData";

interface SummarySlideProps {
  data: RecapData;
  onClose: () => void;
}

export function SummarySlide({ data, onClose }: SummarySlideProps) {
  const t = useT();

  return (
    <div className="relative flex flex-col items-center justify-center gap-6">
      <h2 className="text-2xl font-bold text-gradient text-center">{t("recap.summaryTitle")}</h2>

      <div data-no-advance onClick={(e) => e.stopPropagation()} className="rounded-[20px]">
        <RecapCard data={data} />
      </div>

      <div data-no-advance onClick={(e) => e.stopPropagation()}>
        <Button size="lg" variant="secondary" onClick={onClose} icon={<X className="h-4 w-4" />}>
          {t("recap.close")}
        </Button>
      </div>
    </div>
  );
}
