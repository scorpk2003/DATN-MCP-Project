import { clampPercent, cx, resolveTone } from "./componentUtils.js";

const sizeClasses = {
  sm: "h-1.5",
  md: "h-2",
  lg: "h-3",
};

export function ProgressBar({
  value = 0,
  max = 100,
  tone = "info",
  status,
  label,
  showLabel = false,
  size = "md",
  className = "",
  style,
  ...props
}) {
  const percent = clampPercent(value, max);
  const toneVars = resolveTone(tone, status);
  const displayLabel = label || `${Math.round(percent)}%`;

  return (
    <div className={cx("w-full", className)} style={style} {...props}>
      {showLabel ? (
        <div className="mb-2 flex items-center justify-between gap-3 text-xs font-medium text-[var(--text-muted)]">
          <span className="truncate">{displayLabel}</span>
          <span>{Math.round(percent)}%</span>
        </div>
      ) : null}
      <div
        className={cx(
          "w-full overflow-hidden rounded-full bg-[var(--progress-track)]",
          sizeClasses[size] || sizeClasses.md,
        )}
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(percent)}
        aria-label={displayLabel}
      >
        <div
          className="h-full rounded-full transition-[width] duration-300"
          style={{
            width: `${percent}%`,
            backgroundColor: toneVars.fg,
          }}
        />
      </div>
    </div>
  );
}
