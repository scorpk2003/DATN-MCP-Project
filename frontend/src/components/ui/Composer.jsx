import { Button } from "./Button.jsx";
import { Card } from "./Card.jsx";
import { cx } from "./componentUtils.js";

export function Composer({
  value,
  onChange,
  onSubmit,
  placeholder = "Bạn muốn học gì hôm nay?",
  submitLabel = "Tạo lộ trình",
  submitLoading = false,
  children,
  actions,
  minRows = 3,
  disabled = false,
  className = "",
  textareaClassName = "",
}) {
  const handleSubmit = (event) => {
    event.preventDefault();
    onSubmit?.(event);
  };

  return (
    <Card as="form" padding="sm" className={cx("space-y-3", className)} onSubmit={handleSubmit}>
      <textarea
        value={value}
        onChange={onChange}
        rows={minRows}
        placeholder={placeholder}
        disabled={disabled}
        className={cx(
          "min-h-[96px] w-full resize-none rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--text-primary)]",
          "placeholder:text-[var(--text-muted)] focus:border-[var(--border-accent)] focus:outline-none focus:ring-2 focus:ring-[var(--focus-ring)]",
          "disabled:cursor-not-allowed disabled:opacity-60",
          textareaClassName,
        )}
      />
      {children ? <div className="flex flex-wrap gap-2">{children}</div> : null}
      <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex min-h-10 flex-wrap items-center gap-2">{actions}</div>
        <Button type="submit" disabled={disabled} loading={submitLoading} className="sm:min-w-[140px]">
          {submitLabel}
        </Button>
      </div>
    </Card>
  );
}
