import { useState } from "react";
import { motion } from "framer-motion";
import { Gamepad2, Rocket, Check, Gamepad } from "lucide-react";
import { useT } from "@/hooks/useT";
import { useSettingsStore } from "@/stores/settingsStore";
import { useUiStore } from "@/stores/uiStore";
import { Button } from "@/components/ui/Button";
import { cx } from "@/utils/format";

const STEPS = 3;

export function Onboarding() {
  const t = useT();
  const [step, setStep] = useState(1);
  const setSettings = useSettingsStore((s) => s.set);
  const openAddGame = useUiStore((s) => s.openAddGame);

  const finish = () => {
    setSettings({ onboardingDone: true });
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
          <div className="text-center space-y-4">
            <div className="mx-auto h-16 w-16 rounded-2xl bg-gradient-to-br from-[#7c5cff] to-[#3b82f6] grid place-items-center shadow-[0_0_40px_-8px_#7c5cff]">
              <Gamepad className="h-8 w-8 text-white" />
            </div>
            <h2 className="text-xl font-bold text-ink">{t("onboarding.addFirst")}</h2>
            <p className="text-sm leading-relaxed text-muted">{t("onboarding.addFirstHint")}</p>
            <div className="pt-2">
              <Button variant="primary" onClick={() => openAddGame()} className="w-full">
                <Gamepad className="h-4 w-4" />
                {t("onboarding.addFirst")}
              </Button>
            </div>
          </div>
        )}

        {step === 3 && (
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

          {step < STEPS && (
            <Button variant="primary" onClick={() => setStep((s) => s + 1)}>
              {t("onboarding.next")}
            </Button>
          )}

          {step === STEPS && (
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
