import { faBookmark } from "@fortawesome/free-regular-svg-icons";
import { faArrowRight, faBookmark as faBookmarkSolid } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Button, Card, SectionTitle } from "../../../components/ui";

export function ResourceDetail({ resource }) {
  if (!resource) {
    return (
      <Card className="p-5">
        <SectionTitle eyebrow="Detail" title="Chọn một tài liệu" />
      </Card>
    );
  }

  return (
    <aside className="space-y-4">
      <SectionTitle eyebrow="Detail" title="Thông tin tài liệu" />
      <Card className="space-y-5 p-5">
        <div className="space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <Badge tone={resource.tone}>{resource.type}</Badge>
            <Badge tone={resource.saved ? "success" : "neutral"}>
              <FontAwesomeIcon icon={resource.saved ? faBookmarkSolid : faBookmark} />
              {resource.saved ? "Saved" : "Unsaved"}
            </Badge>
          </div>
          <h2 className="text-2xl font-bold leading-tight text-[var(--text-primary)]">
            {resource.title}
          </h2>
          <p className="text-sm font-semibold text-[var(--text-muted)]">{resource.course}</p>
        </div>

        <p className="text-sm leading-7 text-[var(--text-secondary)]">{resource.description}</p>

        <div className="rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface-muted)] p-3">
          <p className="text-xs font-semibold uppercase tracking-normal text-[var(--text-muted)]">
            Source
          </p>
          <p className="mt-1 text-sm font-bold text-[var(--text-primary)]">{resource.source}</p>
        </div>

        <Button
          full
          disabled={!resource.url}
          onClick={() => {
            if (resource.url) {
              window.open(resource.url, "_blank", "noopener,noreferrer");
            }
          }}
        >
          Mở tài liệu
          <FontAwesomeIcon icon={faArrowRight} />
        </Button>
      </Card>
    </aside>
  );
}
