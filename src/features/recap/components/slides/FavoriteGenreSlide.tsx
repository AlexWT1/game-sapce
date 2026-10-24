import { motion } from "framer-motion";
import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";
import { getGenreTagline } from "../../utils/funFacts";
import { useLocale, useT } from "@/hooks/useT";
import type { RecapData } from "../../hooks/useRecapData";

interface FavoriteGenreSlideProps {
  data: RecapData;
}

const PALETTE = ["#7c5cff", "#5b8cff", "#22d3ee", "#34d399", "#f59e0b", "#fb7185"];

export function FavoriteGenreSlide({ data }: FavoriteGenreSlideProps) {
  const t = useT();
  const locale = useLocale();
  const { topGenre, genres } = data;

  const total = genres.reduce((sum, g) => sum + g.count, 0);
  const share = topGenre && total > 0 ? Math.round((topGenre.count / total) * 100) : 0;
  const tagline = getGenreTagline(topGenre?.genre ?? null, locale);
  const chart = genres.slice(0, 6).map((d) => ({ name: d.genre, value: d.count }));

  return (
    <div
      data-no-advance
      className="relative flex flex-col items-center justify-center text-center max-w-lg w-full px-4"
      onClick={(e) => e.stopPropagation()}
    >
      <div className="flex items-center gap-2 mb-6">
        <span className="grid place-items-center h-10 w-10 rounded-xl bg-[var(--color-accent)]/15 text-lg">🎭</span>
        <h2 className="text-2xl font-bold text-gradient">{t("recap.favoriteGenre")}</h2>
      </div>

      {topGenre ? (
        <>
          <motion.div
            initial={{ opacity: 0, scale: 0.85 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ type: "spring", duration: 0.7 }}
            className="mb-5"
          >
            <div className="text-4xl md:text-5xl font-extrabold text-gradient tracking-tight">
              {topGenre.genre}
            </div>
            <div className="text-base text-muted mt-1">
              {share}% {t("recap.genreShare")}
            </div>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.35 }}
            className="glass rounded-2xl p-3 overflow-hidden w-full"
          >
            {/* h-[120px]: the pie is sized in % of the box, so it always fits the card. */}
            <div className="h-[120px] w-full">
              <ResponsiveContainer width="100%" height="100%">
                <PieChart>
                  <Pie
                    data={chart}
                    dataKey="value"
                    nameKey="name"
                    innerRadius="54%"
                    outerRadius="88%"
                    paddingAngle={3}
                    stroke="transparent"
                  >
                    {chart.map((_, i) => (
                      <Cell key={i} fill={PALETTE[i % PALETTE.length]} />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{
                      background: "#12152b",
                      border: "1px solid rgba(255,255,255,0.1)",
                      borderRadius: 12,
                      fontSize: 12,
                    }}
                  />
                </PieChart>
              </ResponsiveContainer>
            </div>
            {/* Legend */}
            <div className="flex flex-wrap items-center justify-center gap-x-4 gap-y-1 mt-1 pb-1">
              {chart.map((g, i) => (
                <span key={g.name} className="flex items-center gap-1.5 text-[11.5px] text-muted">
                  <span
                    className="h-2 w-2 rounded-full shrink-0"
                    style={{ background: PALETTE[i % PALETTE.length] }}
                  />
                  {g.name} · {g.value}
                </span>
              ))}
            </div>
          </motion.div>

          <motion.p
            className="text-muted text-[15px] mt-5"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.55 }}
          >
            {t("recap.realGenreFan")} {tagline}
          </motion.p>
        </>
      ) : (
        <p className="text-muted">{t("recap.emptyHint")}</p>
      )}
    </div>
  );
}
