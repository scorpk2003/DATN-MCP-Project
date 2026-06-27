import { faThumbtack } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Button, Card, SectionTitle } from "../../../components/ui";

export function NoteDetail({ note }) {
  if (!note) {
    return (
      <Card className="p-5">
        <SectionTitle eyebrow="Detail" title="Chọn một ghi chú" />
      </Card>
    );
  }

  return (
    <aside className="space-y-4">
      <SectionTitle eyebrow="Detail" title="Nội dung ghi chú" />
      <Card className="space-y-5 p-5">
        <div className="space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            {note.pinned ? (
              <Badge tone="warning">
                <FontAwesomeIcon icon={faThumbtack} />
                Pinned
              </Badge>
            ) : null}
            <Badge tone={note.tone}>{note.course}</Badge>
          </div>
          <h2 className="text-2xl font-bold leading-tight text-[var(--text-primary)]">{note.title}</h2>
          <p className="text-xs font-semibold text-[var(--text-muted)]">Cập nhật {note.updatedAt}</p>
        </div>

        <p className="text-sm leading-7 text-[var(--text-secondary)]">{note.content}</p>

        <div className="flex flex-wrap gap-2">
          {note.tags.map((tag) => (
            <Badge key={tag} tone="neutral">
              {tag}
            </Badge>
          ))}
        </div>

        <div className="grid gap-2 sm:grid-cols-2">
          <Button variant="secondary">Chỉnh sửa</Button>
          <Button variant="ghost">Tạo review</Button>
        </div>
      </Card>
    </aside>
  );
}
