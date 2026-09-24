import type { CSSProperties } from "react";
import { cx } from "@/utils/format";

interface SkeletonProps {
  className?: string;
  style?: CSSProperties;
}

export function Skeleton({ className, style }: SkeletonProps) {
  return <div className={cx("skeleton", className)} style={style} aria-hidden />;
}

export function GameCardSkeleton() {
  return (
    <div className="space-y-2.5">
      <Skeleton className="aspect-[3/4] w-full rounded-2xl" />
      <Skeleton className="h-3.5 w-3/4" />
      <Skeleton className="h-3 w-1/2" />
    </div>
  );
}

export function GameGridSkeleton({ count = 8 }: { count?: number }) {
  return (
    <div className="grid gap-4 grid-cols-[repeat(auto-fill,minmax(150px,1fr))]">
      {Array.from({ length: count }).map((_, i) => (
        <GameCardSkeleton key={i} />
      ))}
    </div>
  );
}

export function LineSkeleton({ w = "100%", h = 14 }: { w?: number | string; h?: number }) {
  return <Skeleton className="rounded-md" style={{ width: w, height: h }} />;
}

export function StatsSkeleton() {
  return (
    <div className="space-y-3">
      <Skeleton className="h-24 w-full rounded-2xl" />
      <Skeleton className="h-40 w-full rounded-2xl" />
      <Skeleton className="h-32 w-full rounded-2xl" />
    </div>
  );
}
