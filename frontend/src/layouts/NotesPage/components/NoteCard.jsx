import { faThumbtack } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Card, cx } from "../../../components/ui";

export function NoteCard({ active = false, note, onSelect }) {
  return (
    <Card
      as="button"
      type="button"
      interactive
      selected={active}
      className="w-full text-left"
      onClick={() => onSelect(note.id)}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            {note.pinned ? (
              <FontAwesomeIcon icon={faThumbtack} className="text-xs text-[var(--color-warning)]" />
            ) : null}
            <h3 className="truncate text-base font-bold text-[var(--text-primary)]">{note.title}</h3>
          </div>
          <p className="mt-1 text-xs font-semibold text-[var(--text-muted)]">{note.course}</p>
        </div>
        <Badge tone={note.tone}>{note.updatedAt}</Badge>
      </div>
      <p className="mt-3 line-clamp-2 text-sm leading-6 text-[var(--text-secondary)]">
        {note.excerpt}
      </p>
      <div className="mt-3 flex flex-wrap gap-2">
        {note.tags.map((tag) => (
          <span
            key={tag}
            className={cx(
              "rounded-full bg-[var(--bg-surface-muted)] px-2 py-1 text-xs font-semibold",
              "text-[var(--text-muted)]",
            )}
          >
            {tag}
          </span>
        ))}
      </div>
    </Card>
  );
}
