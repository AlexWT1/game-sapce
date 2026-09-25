import { useState } from "react";
import { motion } from "framer-motion";
import { FolderPlus, Gamepad2, Rocket, Sparkles, Check } from "lucide-react";
import { useT } from "@/hooks/useT";
import { useSettingsStore } from "@/stores/settingsStore";
import { Button } from "@/components/ui/Button";
import { api } from "@/services/api";
import { scanForGames } from "@/services/discovery";
import { useUiStore } from "@/stores/uiStore";
import { cx } from "@/utils/format";

const STEPS = 4;

export function Onboarding() {
  const t = useT();
  const [step, setStep] = useState(1);
  const defaultDir = useSettingsStore((s) => s.defaultGamesDir);
  const [dirs, setDirs] = useState<string[]>(() => (defaultDir ? [defaultDir] : []));
  const [scanning, setScanning] = useState(false);
  const [found, setFound] = useState<number>(0);
  const setSettings = useSettingsStore((s) => s.set);
  const pushToast = useUiStore((s) => s.pushToast);

  const finish = () => {
    setSettings({ onboardingDone: true });
    if (dirs[0] && dirs[0] !== defaultDir) {
      setSettings({ defaultGamesDir: dirs[0] });
      void api.updateSettings({ default_games_dir: dirs[0] }).catch(() => {});
    }
  };

  const addDir = async () => {
    try {
      const dir = await api.selectGameDirectory();
      if (dir && !dirs.includes(dir)) setDirs([...dirs, dir]);
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    }
  };

  const scan = async () => {
    setScanning(true);
    try {
      const { found, added } = await scanForGames(dirs);
      setFound(found);
      if (added > 0) {
        pushToast({ kind: "success", title: `${t("common.add")}: ${added}` });
      }
    } catch (e) {
      pushToast({ kind: "error", title: t("toast.error"), body: String(e) });
    } finally {
      setScanning(false);
    }
  };

  return (
    <div className="relative z-10 h-full grid place-items-center overflow-y-auto px-6 py-10">
      <div className="absolute inset-0 pointer-events-none bg-[radial-gradient(800px_400px_at_50%_0%,rgba(124,92,255,0.15),transparent)]" />
      <motion.div
        key={step}
        initial={{ opacity: 1, y: 0 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.25, ease: [0.2, 0.7, 0.3, 1] }}
        className="relative w-full max-w-lg rounded-3xl p-8 shadow-2xl border border-white/10"
        style={{ background: "#101326", color: "#eceef8" }}
      >
        <div className="flex items-center gap-1.5 mb-7">
          {Array.from({ length: STEPS }).map((_, i) => (
            <span
              key={i}
              className={cx(
                "h-1 rounded-full transition-all duration-300",
                i + 1 <= step ? "w-8 bg-[var(--color-accent)]" : "w-5 bg-white/15",
              )}
            />
          ))}
          <span className="ml-auto text-[11px] text-dim">
            {t("onboarding.step")} {step}/{STEPS}
          </span>
        </div>

        {step === 1 && (
          <div className="text-center space-y-4">
            <div className="mx-auto h-16 w-16 rounded-2xl bg-gradient-to-br from-[#7c5cff] to-[#3b82f6] grid place-items-center shadow-[0_0_40px_-8px_#7c5cff]">
              <Gamepad2 className="h-8 w-8 text-white" />
            </div>
            <h1 className="text-2xl font-bold text-white" style={{ color: "#ffffff" }}>
              {t("onboarding.welcome")}
            </h1>
            <p className="text-sm leading-relaxed" style={{ color: "#9aa0b8" }}>
              {t("onboarding.welcomeHint")}
            </p>
          </div>
        )}

        {step === 2 && (
          <div className="space-y-4">
            <h2 className="text-xl font-bold text-ink">{t("onboarding.dirs")}</h2>
            <p className="text-sm text-muted">{t("onboarding.dirsHint")}</p>
            <div className="space-y-2">
              {dirs.map((d) => (
                <div
                  key={d}
                  className="flex items-center gap-2 px-3 py-2.5 rounded-xl bg-white/5 border border-white/10 text-[13px] text-ink"
                >
                  <FolderPlus className="h-4 w-4 text-[var(--color-accent)] shrink-0" />
                  <span className="truncate">{d}</span>
                </div>
              ))}
            </div>
            <Button variant="outline" onClick={() => void addDir()} className="w-full">
              <FolderPlus className="h-4 w-4" />
              {t("onboarding.selectFolder")}
            </Button>
          </div>
        )}

        {step === 3 && (
          <div className="space-y-4">
            <h2 className="text-xl font-bold text-ink">{t("onboarding.auto")}</h2>
            <p className="text-sm text-muted">{t("onboarding.autoHint")}</p>
            {found > 0 && (
              <p className="text-sm text-emerald-400">
                {t("settings.found")}: {found}
              </p>
            )}
            <div className="flex gap-2 pt-2">
              <Button
                variant="primary"
                loading={scanning}
                onClick={async () => {
                  await scan();
                  setStep(4);
                }}
                className="flex-1"
              >
                <Sparkles className="h-4 w-4" />
                {t("onboarding.find")}
              </Button>
              <Button variant="ghost" onClick={() => setStep(4)}>
                {t("onboarding.skip")}
              </Button>
            </div>
          </div>
        )}

        {step === 4 && (
          <div className="text-center space-y-4">
            <div className="mx-auto h-14 w-14 rounded-full bg-emerald-500/15 border border-emerald-400/30 grid place-items-center">
              <Check className="h-7 w-7 text-emerald-400" />
            </div>
            <h2 className="text-xl font-bold text-ink">{t("onboarding.done")}</h2>
            <p className="text-sm text-muted">{t("onboarding.doneHint")}</p>
          </div>
        )}

        <div className="flex items-center justify-between mt-8 pt-5 border-t border-white/8">
          <Button
            variant="ghost"
            disabled={step === 1}
            onClick={() => setStep((s) => Math.max(1, s - 1))}
          >
            {t("onboarding.back")}
          </Button>

          {step < 3 && (
            <Button variant="primary" onClick={() => setStep((s) => s + 1)}>
              {t("onboarding.next")}
            </Button>
          )}

          {step === 4 && (
            <Button variant="primary" onClick={finish}>
              <Rocket className="h-4 w-4" />
              {t("onboarding.start")}
            </Button>
          )}
        </div>
      </motion.div>
    </div>
  );
}


