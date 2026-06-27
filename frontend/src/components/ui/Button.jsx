import { cx } from "./componentUtils.js";

const variantClasses = {
  primary:
    "border-transparent bg-[var(--action-primary)] text-[var(--action-primary-text)] hover:bg-[var(--action-primary-hover)]",
  accent:
    "border-transparent bg-[var(--action-accent)] text-[var(--action-accent-text)] hover:bg-[var(--action-accent-hover)]",
  secondary:
    "border-[var(--border-primary)] bg-[var(--bg-surface)] text-[var(--text-primary)] hover:bg-[var(--bg-surface-hover)]",
  ghost:
    "border-transparent bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-surface-muted)] hover:text-[var(--text-primary)]",
  danger:
    "border-transparent bg-[var(--color-risk)] text-[var(--text-inverse)] hover:brightness-95",
};

const sizeClasses = {
  sm: "h-8 gap-1.5 rounded-[var(--radius-sm)] px-3 text-xs",
  md: "h-10 gap-2 rounded-[var(--radius-md)] px-4 text-sm",
  lg: "h-12 gap-2.5 rounded-[var(--radius-md)] px-5 text-base",
  icon: "h-10 w-10 rounded-[var(--radius-md)] p-0",
};

export function Button({
  children,
  variant = "primary",
  size = "md",
  full = false,
  loading = false,
  disabled = false,
  className = "",
  type = "button",
  ...props
}) {
  const isDisabled = disabled || loading;

  return (
    <button
      type={type}
      className={cx(
        "inline-flex shrink-0 cursor-pointer items-center justify-center border font-semibold",
        "transition-colors duration-150 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--focus-ring)]",
        "disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-60",
        full && "w-full",
        variantClasses[variant] || variantClasses.primary,
        sizeClasses[size] || sizeClasses.md,
        className,
      )}
      disabled={isDisabled}
      aria-busy={loading || undefined}
      {...props}
    >
      {loading ? (
        <span
          className="h-4 w-4 animate-spin rounded-full border-2 border-current border-r-transparent"
          aria-hidden="true"
        />
      ) : null}
      {children}
    </button>
  );
}
