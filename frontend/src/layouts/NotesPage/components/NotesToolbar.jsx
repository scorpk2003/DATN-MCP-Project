import { faPlus } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Button, Card } from "../../../components/ui";

export function NotesToolbar({
  courses,
  activeCourse,
  query,
  creating = false,
  saving = false,
  newNoteContent = "",
  onCourseChange,
  onQueryChange,
  onNewNoteContentChange,
  onStartCreate,
  onCreateNote,
  onCancelCreate,
}) {
  return (
    <Card className="space-y-3 p-4">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <div className="flex min-w-0 flex-1 flex-col gap-3 sm:flex-row">
          <input
            value={query}
            onChange={(event) => onQueryChange(event.target.value)}
            placeholder="Tim ghi chu"
            className="h-10 min-w-0 flex-1 cursor-text rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface)] px-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--border-accent)] focus:outline-none focus:ring-2 focus:ring-[var(--focus-ring)]"
          />
          <select
            value={activeCourse}
            onChange={(event) => onCourseChange(event.target.value)}
            className="h-10 cursor-pointer rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface)] px-3 text-sm font-semibold text-[var(--text-primary)] focus:border-[var(--border-accent)] focus:outline-none focus:ring-2 focus:ring-[var(--focus-ring)]"
          >
            {courses.map((course) => (
              <option key={course} value={course}>
                {course}
              </option>
            ))}
          </select>
        </div>
        <Button variant="primary" onClick={creating ? onCreateNote : onStartCreate} loading={saving}>
          <FontAwesomeIcon icon={faPlus} />
          Ghi chu moi
        </Button>
      </div>
      {creating ? (
        <form className="space-y-3" onSubmit={onCreateNote}>
          <textarea
            value={newNoteContent}
            onChange={(event) => onNewNoteContentChange(event.target.value)}
            placeholder="Nhap noi dung ghi chu"
            rows={4}
            className="min-h-28 w-full resize-y rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface)] px-3 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--border-accent)] focus:outline-none focus:ring-2 focus:ring-[var(--focus-ring)]"
            autoFocus
          />
          <div className="flex justify-end gap-2">
            <Button variant="ghost" onClick={onCancelCreate}>
              Huy
            </Button>
            <Button variant="primary" type="submit" loading={saving} disabled={!newNoteContent.trim()}>
              Luu ghi chu
            </Button>
          </div>
        </form>
      ) : null}
    </Card>
  );
}
