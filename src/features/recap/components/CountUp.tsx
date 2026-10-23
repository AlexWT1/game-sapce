import { useEffect, useRef, useState } from "react";
import { motion, useMotionValue, useTransform, animate } from "framer-motion";

interface CountUpProps {
  to: number;
  duration?: number;
  formatter?: (n: number) => string;
  className?: string;
  delay?: number;
}

export function CountUp({
  to,
  duration = 1.5,
  formatter = (n) => Math.round(n).toLocaleString(),
  className,
  delay = 0,
}: CountUpProps) {
  const [started, setStarted] = useState(false);
  const motionValue = useMotionValue(0);
  const displayValue = useTransform(motionValue, (v) => formatter(v));
  const prevTo = useRef(to);

  useEffect(() => {
    const timeout = setTimeout(() => setStarted(true), delay * 1000);
    return () => clearTimeout(timeout);
  }, [delay]);

  useEffect(() => {
    if (!started) return;
    const controls = animate(motionValue, to, {
      duration,
      ease: "easeOut",
    });
    prevTo.current = to;
    return controls.stop;
  }, [to, duration, started, motionValue]);

  return (
    <motion.span className={className}>
      {started ? displayValue : formatter(0)}
    </motion.span>
  );
}

/** Animated counter for seconds → hours display */
export function CountUpHours({ seconds, duration = 1.5, className, delay = 0 }: {
  seconds: number;
  duration?: number;
  className?: string;
  delay?: number;
}) {
  const hours = seconds / 3600;
  return (
    <CountUp
      to={hours}
      duration={duration}
      formatter={(n) => Math.round(n).toLocaleString()}
      className={className}
      delay={delay}
    />
  );
}
