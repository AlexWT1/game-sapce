import { type SelectHTMLAttributes } from "react";
import { ChevronDown } from "lucide-react";
import { cx } from "@/utils/format";

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  options: Array<{ value: string; label: string }>;
}

export function Select({ label, options, className, id, ...rest }: SelectProps) {
  const selectId = id ?? (label ? `sel-${label}` : undefined);
  return (
    <label className="block space-y-1.5" htmlFor={selectId}>
      {label && (
        <span className="text-[12px] font-medium text-muted uppercase tracking-wide">
          {label}
        </span>
      )}
      <div className="relative">
        <select
          id={selectId}
          className={cx(
            "w-full h-10 pl-3 pr-9 rounded-xl bg-white/[0.05] border border-white/10",
            "text-sm text-ink outline-none appearance-none transition",
            "focus:border-[var(--color-accent)]/60",
            className,
          )}
          {...rest}
        >
          {options.map((o) => (
            <option key={o.value} value={o.value} className="bg-[#0d1020] text-ink">
              {o.label}
            </option>
          ))}
        </select>
        <ChevronDown className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted" />
      </div>
    </label>
  );
}
