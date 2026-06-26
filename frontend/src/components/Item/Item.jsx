import { Badge, Card, ProgressBar } from "../ui";

function Item({
  name,
  description,
  complete,
  duration,
  statusLabel = "Đang học",
  tone = "info",
}) {
  return (
    <Card interactive className="flex min-h-48 flex-col gap-4">
      <div className="flex items-start justify-between gap-3">
        <Badge tone={tone}>{statusLabel}</Badge>
        {duration ? <span className="text-xs font-semibold text-[var(--text-muted)]">{duration}</span> : null}
      </div>
      <div className="min-w-0 flex-1">
        <h3 className="truncate text-base font-bold text-[var(--text-primary)]">{name}</h3>
        <p className="mt-2 text-sm leading-6 text-[var(--text-secondary)]">{description}</p>
      </div>
      <ProgressBar value={complete} max={1} tone={tone} label={`${name} progress`} />
    </Card>
  );
}

export default Item;
