import { faBookmark } from "@fortawesome/free-regular-svg-icons";
import { faBookmark as faBookmarkSolid } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Card } from "../../../components/ui";

export function ResourceCard({ active = false, resource, onSelect }) {
  return (
    <Card
      as="button"
      type="button"
      interactive
      selected={active}
      className="flex h-full w-full flex-col gap-4 text-left"
      onClick={() => onSelect(resource.id)}
    >
      <div className="flex items-start justify-between gap-3">
        <Badge tone={resource.tone}>{resource.type}</Badge>
        <FontAwesomeIcon
          icon={resource.saved ? faBookmarkSolid : faBookmark}
          className="mt-1 text-[var(--color-agent)]"
        />
      </div>
      <div className="min-w-0 flex-1">
        <h3 className="text-lg font-bold leading-snug text-[var(--text-primary)]">
          {resource.title}
        </h3>
        <p className="mt-2 line-clamp-3 text-sm leading-6 text-[var(--text-secondary)]">
          {resource.description}
        </p>
      </div>
      <div className="flex flex-wrap items-center gap-2 text-xs font-semibold text-[var(--text-muted)]">
        <span>{resource.course}</span>
        <span aria-hidden="true">/</span>
        <span>{resource.updatedAt}</span>
      </div>
    </Card>
  );
}
