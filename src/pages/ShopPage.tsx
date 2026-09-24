import { ShoppingBag } from "lucide-react";
import { EmptyState } from "@/components/ui/EmptyState";
import { useT } from "@/hooks/useT";

export function ShopPage() {
  const t = useT();
  return (
    <div className="p-5 min-h-full">
      <h1 className="text-xl font-bold text-ink mb-6">{t("shop.title")}</h1>
      <EmptyState
        icon={<ShoppingBag className="h-6 w-6" />}
        title={t("shop.empty")}
        body={t("shop.hint")}
      />
    </div>
  );
}
