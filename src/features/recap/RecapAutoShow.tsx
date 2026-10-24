import { useEffect, useMemo, useRef, useState } from "react";
import { useLocation } from "react-router-dom";
import { RecapSlideshow } from "./RecapSlideshow";
import { useRecapData, type RecapData, type RecapPeriod } from "./hooks/useRecapData";
import { useSettingsStore } from "@/stores/settingsStore";
import { useStatisticsStore } from "@/stores/statisticsStore";

/**
 * === TEST MODE ===
 * true → Recap показывается один раз при запуске приложения даже если сейчас
 * не конец месяца (для проверки). Показ запоминается в настройках, поэтому
 * повторных автопоказов до следующего месяца не будет, а месячная логика
 * не меняется. Когда закончишь проверку — поставь false.
 */
const RECAP_TEST_NOW = false;

/** Последний ли сегодня день месяца? */
function isLastDayOfMonth(d: Date = new Date()): boolean {
  return new Date(d.getFullYear(), d.getMonth() + 1, 0).getDate() === d.getDate();
}

/** Период-метка текущего месяца, «YYYY-MM». */
function monthKey(d: Date = new Date()): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

/**
 * Ключ автопоказа для текущего запуска: что показывать и почему.
 * Возвращаем `undefined`, когда автопоказ не положен (не конец месяца и не тест).
 * Тестовый показ выключается на последний день месяца, чтобы не перехватывать
 * настоящий месячный Recap.
 */
function recapTrigger(): { key: string; reason: "test" | "monthEnd" } | undefined {
  const now = new Date();
  const lastDay = isLastDayOfMonth(now);
  if (RECAP_TEST_NOW && !lastDay) return { key: `test:${monthKey(now)}`, reason: "test" };
  if (lastDay) return { key: `auto:${monthKey(now)}`, reason: "monthEnd" };
  return undefined;
}

/**
 * Автопоказ Recap «при первом входе»: в последний день месяца слайдшоу
 * открывается само (один раз за месяц, состояние живёт в settingsStore).
 */
export function RecapAutoShow() {
  const location = useLocation();
  const onRecapPage = location.pathname === "/recap";
  const onboardingDone = useSettingsStore((s) => s.onboardingDone);
  const hydrated = useSettingsStore((s) => s.hydrated);
  const recapShownKey = useSettingsStore((s) => s.recapShownKey);
  const setSetting = useSettingsStore((s) => s.set);
  // Only load recap stats when a show is actually due (and after the shared
  // lifetime cache is loaded by AppLayout, so HomePage panels stay untouched).
  const statsLoaded = useStatisticsStore((s) => s.loaded);

  const trigger = useMemo(() => recapTrigger(), []);
  const showDue =
    !!trigger &&
    hydrated &&
    onboardingDone &&
    !onRecapPage &&
    statsLoaded &&
    recapShownKey !== trigger.key;

  const [open, setOpen] = useState(false);
  const [period, setPeriod] = useState<RecapPeriod>("month");
  const { data, loading } = useRecapData(period, showDue);

  // Пока оверлей открыт и данные перезагружаются (смена месяц/год),
  // рендерим последнюю удачную порцию данных, чтобы слайдшоу не мигало.
  const lastData = useRef<RecapData | null>(null);
  if (data) lastData.current = data;

  useEffect(() => {
    if (!showDue || open) return;
    if (loading || !data) return; // ждём, пока стата подтянется
    // Решение принимаем один раз за период: помечаем, даже если показывать
    // нечего (пустая библиотека), чтобы recap не «выстрелил» посреди сессии.
    setSetting({ recapShownKey: trigger!.key });
    if (data.gamesCount === 0) return;
    setOpen(true);
  }, [showDue, open, loading, data, setSetting, trigger]);

  if (!open || !lastData.current) return null;

  return (
    <RecapSlideshow
      data={lastData.current}
      period={period}
      onPeriodChange={setPeriod}
      onClose={() => setOpen(false)}
    />
  );
}
