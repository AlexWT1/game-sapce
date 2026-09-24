import { cx } from "@/utils/format";

interface ToggleProps {
  checked: boolean;
  onChange: (v: boolean) => void;
  label?: string;
  disabled?: boolean;
}

export function Toggle({ checked, onChange, label, disabled }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      title={checked ? "ON" : "OFF"}
      className={cx(
        "relative inline-flex h-7 w-12 shrink-0 items-center rounded-full border-2 transition-colors duration-200",
        "disabled:opacity-40 disabled:pointer-events-none",
        checked
          ? "bg-[var(--color-accent)] border-[var(--color-accent)] shadow-[0_0_12px_-2px_var(--color-accent)]"
          : "bg-[#1a1e35] border-white/30",
      )}
    >
      <span
        className={cx(
          "absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white shadow-md transition-transform duration-200 ease-out",
          checked && "translate-x-5",
        )}
      />
    </button>
  );
}
