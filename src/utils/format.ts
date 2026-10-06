/** Formats seconds → "1 мин 58 сек" / "3 ч" / "426 ч 17 мин". */
export function formatPlaytime(seconds: number): string {
  if (seconds < 0) seconds = 0;
  if (seconds === 0) return "0 мин";
  if (seconds < 60) return `${seconds} сек`;
  if (seconds < 3600) {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return s === 0 ? `${m} мин` : `${m} мин ${s} сек`;
  }
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (m === 0) return `${h} ч`;
  return `${h} ч ${m} мин`;
}

/** Compact H:MM:SS for live timers. */
export function formatHMS(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(h)}:${pad(m)}:${pad(sec)}`;
}

export function formatNumber(n: number): string {
  return new Intl.NumberFormat().format(n);
}

function bcp47(locale: string): string {
  if (locale.startsWith("ru")) return "ru-RU";
  if (locale.startsWith("en")) return "en-US";
  if (locale.includes("-") || locale.includes("_")) return locale.replace("_", "-");
  return locale;
}

function isRu(locale: string): boolean {
  return locale.startsWith("ru");
}

export function formatDate(iso: string | null | undefined, locale = "ru"): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  return d.toLocaleDateString(bcp47(locale), {
    day: "numeric",
    month: "short",
    year: "numeric",
  });
}

export function formatDateTime(iso: string | null | undefined, locale = "ru"): string {
  if (!iso) return "—";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  return d.toLocaleString(bcp47(locale), {
    day: "numeric",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function relativeTime(
  iso: string | null | undefined,
  locale = "ru",
): string {
  if (!iso) return "—";
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "—";
  const diff = Math.max(0, Date.now() - then);
  const min = Math.floor(diff / 60000);
  const ru = isRu(locale);
  if (min < 1) return ru ? "только что" : "just now";
  if (min < 60) return ru ? `${min} мин назад` : `${min}m ago`;
  const h = Math.floor(min / 60);
  if (h < 24) return ru ? `${h} ч назад` : `${h}h ago`;
  const d = Math.floor(h / 24);
  return ru ? `${d} дн назад` : `${d}d ago`;
}

export function percent(current: number, total: number): number {
  if (total <= 0) return 0;
  return Math.min(100, Math.round((current / total) * 100));
}

export function clamp(n: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, n));
}

export function cx(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

export function fileNameFromPath(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

export function baseName(path: string): string {
  const name = fileNameFromPath(path);
  return name.replace(/\.[^.]+$/, "");
}

/** Formats a byte count as a human-readable size, e.g. "12.4 МБ" / "12.4 MB". */
export function formatBytes(bytes: number | null | undefined, locale = "ru"): string {
  const ru = isRu(locale);
  if (!Number.isFinite(bytes as number) || (bytes as number) <= 0) {
    return ru ? "0 Б" : "0 B";
  }
  const b = bytes as number;
  const units = ru ? ["Б", "КБ", "МБ", "ГБ"] : ["B", "KB", "MB", "GB"];
  const i = Math.min(units.length - 1, Math.floor(Math.log(b) / Math.log(1024)));
  const value = b / 1024 ** i;
  return `${value.toFixed(value >= 10 || i === 0 ? 0 : 1)} ${units[i]}`;
}

