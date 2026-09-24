import { forwardRef, type InputHTMLAttributes, type TextareaHTMLAttributes } from "react";
import { cx } from "@/utils/format";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  hint?: string;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(function Input(
  { label, hint, className, id, ...rest },
  ref,
) {
  const inputId = id ?? (label ? `in-${label}` : undefined);
  return (
    <label className="block space-y-1.5" htmlFor={inputId}>
      {label && (
        <span className="text-[12px] font-medium text-muted uppercase tracking-wide">
          {label}
        </span>
      )}
      <input
        ref={ref}
        id={inputId}
        className={cx(
          "w-full h-10 px-3 rounded-xl bg-white/[0.05] border border-white/10",
          "text-sm text-ink placeholder:text-dim outline-none transition",
          "focus:border-[var(--color-accent)]/60 focus:bg-white/[0.07]",
          className,
        )}
        {...rest}
      />
      {hint && <span className="block text-[11px] text-dim">{hint}</span>}
    </label>
  );
});

interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(function Textarea(
  { label, className, ...rest },
  ref,
) {
  return (
    <label className="block space-y-1.5">
      {label && (
        <span className="text-[12px] font-medium text-muted uppercase tracking-wide">
          {label}
        </span>
      )}
      <textarea
        ref={ref}
        className={cx(
          "w-full min-h-[96px] px-3 py-2.5 rounded-xl bg-white/[0.05] border border-white/10",
          "text-sm text-ink placeholder:text-dim outline-none transition resize-y",
          "focus:border-[var(--color-accent)]/60",
          className,
        )}
        {...rest}
      />
    </label>
  );
});
