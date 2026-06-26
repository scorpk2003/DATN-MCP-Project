import { cx, resolveTone } from "./componentUtils.js";

const sizeClasses = {
  sm: "px-2 py-1 text-[11px]",
  md: "px-2.5 py-1.5 text-xs",
};

export function Badge({
  children,
  label,
  tone = "neutral",
  status,
  size = "sm",
  className = "",
  style,
  ...props
}) {
  const toneVars = resolveTone(tone, status);

  return (
    <span
      className={cx(
        "inline-flex max-w-full items-center gap-1 rounded-full border font-semibold leading-none",
        "whitespace-nowrap align-middle",
        sizeClasses[size] || sizeClasses.sm,
        className,
      )}
      style={{
        backgroundColor: toneVars.bg,
        borderColor: toneVars.border,
        color: toneVars.fg,
        ...style,
      }}
      {...props}
    >
      {children || label}
    </span>
  );
}
