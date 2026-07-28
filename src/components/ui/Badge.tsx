import { cn } from "@/utils";
import type { ReactNode } from "react";

type BadgeVariant = "default" | "success" | "warning" | "error" | "muted";

const variantStyles: Record<BadgeVariant, string> = {
  default: "bg-primary/10 text-primary border-primary/20",
  success: "bg-green-500/10 text-green-400 border-green-500/20",
  warning: "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
  error: "bg-red-500/10 text-red-400 border-red-500/20",
  muted: "bg-muted text-muted-foreground border-border",
};

interface BadgeProps {
  variant?: BadgeVariant;
  children: ReactNode;
  className?: string;
}

export function Badge({ variant = "default", children, className }: BadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium",
        variantStyles[variant],
        className,
      )}
    >
      {children}
    </span>
  );
}