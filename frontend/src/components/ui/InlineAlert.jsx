import { Badge } from "./Badge.jsx";
import { Card } from "./Card.jsx";

export function InlineAlert({ title, description, tone = "info" }) {
  return (
    <Card className="flex items-start gap-3 p-4">
      <Badge tone={tone}>{tone}</Badge>
      <div className="min-w-0">
        <p className="text-sm font-bold text-[var(--text-primary)]">{title}</p>
        {description ? (
          <p className="mt-1 text-sm leading-6 text-[var(--text-secondary)]">{description}</p>
        ) : null}
      </div>
    </Card>
  );
}
