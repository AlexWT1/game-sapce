import { motion } from "framer-motion";

interface SlideWrapperProps {
  children: React.ReactNode;
  className?: string;
  index: number;
}

const variants = {
  enter: { opacity: 0, y: 40, scale: 0.96 },
  center: { opacity: 1, y: 0, scale: 1 },
  exit: { opacity: 0, y: -40, scale: 0.96 },
};

export function SlideWrapper({ children, className = "", index }: SlideWrapperProps) {
  return (
    <motion.div
      key={index}
      variants={variants}
      initial="enter"
      animate="center"
      exit="exit"
      transition={{
        duration: 0.5,
        ease: [0.25, 0.46, 0.45, 0.94],
      }}
      className={`absolute inset-0 flex flex-col items-center justify-center p-8 ${className}`}
    >
      {children}
    </motion.div>
  );
}

/** Stagger container for list animations */
export function StaggerContainer({
  children,
  className,
  delayChildren = 0,
  staggerChildren = 0.1,
}: {
  children: React.ReactNode;
  className?: string;
  delayChildren?: number;
  staggerChildren?: number;
}) {
  return (
    <motion.div
      initial="hidden"
      animate="show"
      variants={{
        hidden: {},
        show: {
          transition: {
            delayChildren,
            staggerChildren,
          },
        },
      }}
      className={className}
    >
      {children}
    </motion.div>
  );
}

export const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  show: { opacity: 1, y: 0 },
};
