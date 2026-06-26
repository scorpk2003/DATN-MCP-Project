import { cx } from "./componentUtils.js";

export function SectionTitle({
  eyebrow,
  title,
  children,
  action,
  className = "",
  titleClassName = "",
}) {
  return (
    <div className={cx("flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between", className)}>
      <div className="min-w-0">
        {eyebrow ? (
          <p className="mb-1 text-xs font-semibold uppercase tracking-normal text-[var(--text-muted)]">
            {eyebrow}
          </p>
        ) : null}
        <h2
          className={cx(
            "truncate text-xl font-bold leading-tight text-[var(--text-primary)]",
            titleClassName,
          )}
        >
          {title || children}
        </h2>
      </div>
      {action ? <div className="shrink-0">{action}</div> : null}
    </div>
  );
}
