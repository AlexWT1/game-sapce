import type { RecapData } from "../hooks/useRecapData";
import { useLocale } from "@/hooks/useT";

/**
 * Shareable summary card shown on the final recap slide. Built with inline
 * styles and plain hex/rgba colors (no Tailwind color functions) so it renders
 * identically regardless of chart/theme plumbing.
 */
export function RecapCard({ data }: { data: RecapData }) {
  const locale = useLocale();
  const ru = locale === "ru";

  const totalMinutes = Math.round(data.totalSeconds / 60);
  const showHours = data.totalSeconds >= 3600;
  const periodLabel = data.period === "year" ? (ru ? "за год" : "yearly") : (ru ? "за месяц" : "monthly");
  const monthName = ru
    ? new Date().toLocaleString("ru-RU", { month: "long" })
    : new Date().toLocaleString("en-US", { month: "long" });
  const title = ru ? "Твой игровой месяц" : "Your gaming month";
  const top3 = data.topGames.slice(0, 3);

  return (
    <div
      style={{
        width: 420,
        borderRadius: 20,
        padding: 28,
        background: "linear-gradient(160deg, #0a0d1a 0%, #151933 100%)",
        border: "1px solid rgba(124, 92, 255, 0.35)",
        boxShadow: "0 24px 60px rgba(0, 0, 0, 0.5)",
        fontFamily: "Inter, Segoe UI, system-ui, sans-serif",
        color: "#eceef8",
        boxSizing: "border-box",
        userSelect: "none",
      }}
    >
      {/* Brand row */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 18 }}>
        <span
          style={{
            fontSize: 11,
            letterSpacing: 2.5,
            fontWeight: 700,
            color: "rgba(124, 92, 255, 0.9)",
            textTransform: "uppercase",
          }}
        >
          Game Space
        </span>
        <span
          style={{
            fontSize: 10,
            letterSpacing: 1.5,
            padding: "4px 10px",
            borderRadius: 999,
            background: "rgba(124, 92, 255, 0.16)",
            color: "#a78bfa",
            textTransform: "uppercase",
          }}
        >
          Recap · {periodLabel}
        </span>
      </div>

      <div style={{ fontSize: 15, color: "#9aa0b8", marginBottom: 2 }}>{monthName}</div>
      <div
        style={{
          fontSize: 26,
          fontWeight: 800,
          lineHeight: 1.15,
          background: "linear-gradient(90deg, #eceef8, #a78bfa)",
          WebkitBackgroundClip: "text",
          backgroundClip: "text",
          WebkitTextFillColor: "transparent",
          marginBottom: 20,
        }}
      >
        {title}
      </div>

      {/* Hero number */}
      <div style={{ borderRadius: 14, background: "rgba(124, 92, 255, 0.1)", padding: "16px 18px", marginBottom: 16 }}>
        <div style={{ fontSize: 11, letterSpacing: 1, color: "#9aa0b8", textTransform: "uppercase", marginBottom: 4 }}>
          {ru ? "Суммарно наиграно" : "Total playtime"}
        </div>
        <div style={{ display: "flex", alignItems: "baseline", gap: 8 }}>
          <span style={{ fontSize: 40, fontWeight: 800, color: "#eceef8", lineHeight: 1 }}>
            {showHours ? Math.round(data.totalSeconds / 3600) : totalMinutes}
          </span>
          <span style={{ fontSize: 15, color: "#a78bfa", fontWeight: 600 }}>
            {showHours ? (ru ? "часов" : "hours") : ru ? "минут" : "minutes"}
          </span>
        </div>
      </div>

      {/* Number grid */}
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10, marginBottom: 18 }}>
        <Cell label={ru ? "Достижения" : "Achievements"} value={`${data.achievementsUnlocked}/${data.achievementsTotal}`} />
        <Cell label={ru ? "Сессий" : "Sessions"} value={String(data.sessionsCount)} />
        <Cell label={ru ? "Любимый жанр" : "Top genre"} value={data.topGenre?.genre ?? "—"} />
        <Cell label={ru ? "Игр" : "Games"} value={String(data.gamesCount)} />
      </div>

      {/* Top games */}
      {top3.length > 0 && (
        <div style={{ display: "flex", gap: 10, marginBottom: 18 }}>
          {top3.map((g, i) => (
            <div
              key={g.gameId}
              style={{
                flex: 1,
                borderRadius: 12,
                background: "rgba(255, 255, 255, 0.05)",
                border: "1px solid rgba(255, 255, 255, 0.08)",
                overflow: "hidden",
              }}
            >
              <div style={{ position: "relative", height: 64, background: "rgba(255, 255, 255, 0.06)" }}>
                {g.coverPath ? (
                  <img
                    src={g.coverPath}
                    alt=""
                    style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }}
                  />
                ) : (
                  <div
                    style={{
                      height: "100%",
                      display: "grid",
                      placeItems: "center",
                      fontSize: 22,
                      fontWeight: 800,
                      color: "#6b7189",
                    }}
                  >
                    {g.title.slice(0, 1)}
                  </div>
                )}
                <span
                  style={{
                    position: "absolute",
                    top: 4,
                    left: 4,
                    width: 18,
                    height: 18,
                    borderRadius: 6,
                    display: "grid",
                    placeItems: "center",
                    fontSize: 10,
                    fontWeight: 800,
                    background: i === 0 ? "rgba(251, 191, 36, 0.85)" : "rgba(0, 0, 0, 0.55)",
                    color: i === 0 ? "#0a0d1a" : "#eceef8",
                  }}
                >
                  {i + 1}
                </span>
              </div>
              <div style={{ padding: "6px 8px" }}>
                <div style={{ fontSize: 10, fontWeight: 600, color: "#eceef8", whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                  {g.title}
                </div>
                <div style={{ fontSize: 9.5, color: "#6b7189" }}>
                  {formatShort(g.seconds)}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Footer */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          borderTop: "1px solid rgba(255, 255, 255, 0.08)",
          paddingTop: 12,
        }}
      >
        <span style={{ fontSize: 10, color: "#6b7189" }}>
          {ru ? "Ваши игры. Ваше время." : "Your games. Your time."}
        </span>
        <span style={{ fontSize: 10, color: "#6b7189" }}>Game Space · {new Date().getFullYear()}</span>
      </div>
    </div>
  );
}

function Cell({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        borderRadius: 12,
        background: "rgba(255, 255, 255, 0.04)",
        border: "1px solid rgba(255, 255, 255, 0.08)",
        padding: "10px 12px",
      }}
    >
      <div style={{ fontSize: 9.5, letterSpacing: 0.8, color: "#9aa0b8", textTransform: "uppercase", marginBottom: 2 }}>
        {label}
      </div>
      <div style={{ fontSize: 17, fontWeight: 700, color: "#eceef8" }}>{value}</div>
    </div>
  );
}

/** Compact "Xh" / "Xm" label for the card footer tiles. */
function formatShort(seconds: number): string {
  if (seconds >= 3600) {
    const h = seconds / 3600;
    return `${h >= 10 ? Math.round(h) : h.toFixed(1)} ч`;
  }
  if (seconds >= 60) return `${Math.round(seconds / 60)} мин`;
  return `${seconds} с`;
}
