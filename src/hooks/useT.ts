import { useCallback } from "react";
import { dictionaries, type TranslationKey } from "@/i18n";
import { useSettingsStore } from "@/stores/settingsStore";

export type { TranslationKey };

export function useLocale() {
  return useSettingsStore((s) => s.locale);
}

export function useT() {
  const locale = useSettingsStore((s) => s.locale);
  return useCallback(
    (key: TranslationKey): string => {
      const dict = dictionaries[locale] ?? dictionaries.ru;
      return dict[key] ?? dictionaries.ru[key] ?? String(key);
    },
    [locale],
  );
}

export function useLocaleCode() {
  const locale = useLocale();
  return locale === "ru" ? "ru-RU" : "en-US";
}
