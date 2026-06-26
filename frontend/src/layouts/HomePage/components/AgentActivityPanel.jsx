import { faCircleCheck } from "@fortawesome/free-regular-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Card, SectionTitle } from "../../../components/ui";

export function AgentActivityPanel({ activities }) {
  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Agent" title="Hoạt động gần đây" />
      <Card className="divide-y divide-[var(--border-secondary)] p-0">
        {activities.map((activity) => (
          <article key={activity.id} className="flex gap-3 p-4">
            <div className="mt-1 flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[var(--bg-surface-muted)] text-[var(--color-info)]">
              <FontAwesomeIcon icon={faCircleCheck} />
            </div>
            <div className="min-w-0 flex-1">
              <div className="flex flex-wrap items-center gap-2">
                <h3 className="text-sm font-bold text-[var(--text-primary)]">{activity.title}</h3>
                <Badge tone={activity.tone}>{activity.tone}</Badge>
              </div>
              <p className="mt-1 text-sm leading-6 text-[var(--text-secondary)]">
                {activity.description}
              </p>
            </div>
          </article>
        ))}
      </Card>
    </section>
  );
}
