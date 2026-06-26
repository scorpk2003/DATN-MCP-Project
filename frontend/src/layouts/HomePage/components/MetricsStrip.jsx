import { Badge, Card } from "../../../components/ui";

export function MetricsStrip({ metrics }) {
  return (
    <section className="grid gap-3 sm:grid-cols-3">
      {metrics.map((metric) => (
        <Card key={metric.id} className="flex items-center justify-between gap-4">
          <div className="min-w-0">
            <p className="truncate text-sm font-medium text-[var(--text-muted)]">{metric.label}</p>
            <p className="mt-1 text-2xl font-bold text-[var(--text-primary)]">{metric.value}</p>
          </div>
          <Badge tone={metric.tone}>Live</Badge>
        </Card>
      ))}
    </section>
  );
}
