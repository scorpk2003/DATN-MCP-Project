import { faBookmark } from "@fortawesome/free-regular-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Card, SectionTitle } from "../../../components/ui";

export function ResourcePanel({ resources }) {
  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Materials" title="Tài liệu gắn với lộ trình" />
      <Card className="space-y-3">
        {resources.map((resource) => (
          <article
            key={resource.id}
            className="flex gap-3 rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface-muted)] p-3"
          >
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[var(--radius-sm)] bg-[var(--bg-surface)] text-[var(--color-agent)]">
              <FontAwesomeIcon icon={faBookmark} />
            </div>
            <div className="min-w-0 flex-1">
              <div className="flex flex-wrap items-center gap-2">
                <h3 className="text-sm font-bold text-[var(--text-primary)]">{resource.title}</h3>
                <Badge tone={resource.tone}>{resource.type}</Badge>
              </div>
              <p className="mt-1 text-xs font-medium text-[var(--text-muted)]">{resource.source}</p>
            </div>
          </article>
        ))}
      </Card>
    </section>
  );
}
