import { createElement } from "react";
import { cx } from "./componentUtils.js";

const toneClasses = {
  surface: "border-[var(--border-primary)] bg-[var(--bg-surface)]",
  muted: "border-[var(--border-secondary)] bg-[var(--bg-surface-muted)]",
  inverse: "border-[var(--sidebar-border)] bg-[var(--bg-inverse)] text-[var(--text-inverse)]",
};

const paddingClasses = {
  none: "",
  sm: "p-3",
  md: "p-4",
  lg: "p-5",
};

export function Card({
  as = "div",
  children,
  tone = "surface",
  padding = "md",
  interactive = false,
  selected = false,
  className = "",
  ...props
}) {
  return createElement(
    as,
    {
      className: cx(
        "rounded-[var(--radius-card)] border shadow-[var(--shadow-card)]",
        toneClasses[tone] || toneClasses.surface,
        paddingClasses[padding] || paddingClasses.md,
        interactive &&
          "transition-colors duration-150 hover:border-[var(--border-accent)] hover:bg-[var(--bg-surface-hover)]",
        selected && "border-[var(--border-accent)] ring-2 ring-[var(--focus-ring)]",
        className,
      ),
      ...props,
    },
    children,
  );
}
