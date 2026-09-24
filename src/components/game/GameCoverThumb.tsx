import { Gamepad2 } from "lucide-react";
import { cx } from "@/utils/format";

interface GameCoverThumbProps {
  src: string | null;
  title: string;
  size?: number;
  className?: string;
}

export function GameCoverThumb({ src, title, size = 22, className }: GameCoverThumbProps) {
  if (src) {
    return (
      <img
        src={src}
        alt=""
        width={size}
        height={size}
        className={cx("rounded object-cover shrink-0", className)}
        style={{ width: size, height: size }}
        loading="lazy"
      />
    );
  }
  return (
    <span
      className={cx(
        "rounded bg-white/10 grid place-items-center font-bold text-muted shrink-0",
        className,
      )}
      style={{ width: size, height: size, fontSize: Math.max(9, size * 0.4) }}
      aria-hidden
    >
      {title.slice(0, 1).toUpperCase()}
    </span>
  );
}

export function GameCoverFallback() {
  return (
    <div className="h-full w-full grid place-items-center bg-gradient-to-br from-[#1a1f3a] to-[#0d1020]">
      <Gamepad2 className="h-10 w-10 text-dim" />
    </div>
  );
}
