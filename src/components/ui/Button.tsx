import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from "react";
import { Loader2 } from "lucide-react";
import { cx } from "@/utils/format";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "outline" | "danger";
export type ButtonSize = "sm" | "md" | "lg" | "icon";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  icon?: ReactNode;
  block?: boolean;
}

const variants: Record<ButtonVariant, string> = {
  primary:
    "bg-[var(--color-accent)] text-white hover:brightness-110 shadow-[0_8px_24px_-10px_var(--color-accent)]",
  secondary: "bg-white/10 text-ink hover:bg-white/15 border border-white/10",
  ghost: "bg-transparent text-muted hover:bg-white/8 hover:text-ink",
  outline: "border border-white/15 text-ink hover:bg-white/8 hover:border-white/25",
  danger: "bg-red-500/90 text-white hover:bg-red-500",
};

const sizes: Record<ButtonSize, string> = {
  sm: "h-8 px-3 text-[13px] rounded-lg gap-1.5",
  md: "h-10 px-4 text-sm rounded-xl gap-2",
  lg: "h-12 px-6 text-[15px] rounded-xl gap-2.5",
  icon: "h-9 w-9 rounded-lg",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = "primary", size = "md", loading, icon, className, children, disabled, block, ...rest },
  ref,
) {
  return (
    <button
      ref={ref}
      disabled={disabled || loading}
      className={cx(
        "inline-flex items-center justify-center font-medium transition-all duration-150 select-none",
        "disabled:opacity-50 disabled:pointer-events-none",
        variants[variant],
        sizes[size],
        block && "w-full",
        className,
      )}
      {...rest}
    >
      {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : icon}
      {children}
    </button>
  );
});
