import { faPlus } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Button, Card } from "../../../components/ui";

export function NotesToolbar({
  courses,
  activeCourse,
  query,
  onCourseChange,
  onQueryChange,
}) {
  return (
    <Card className="flex flex-col gap-3 p-4 lg:flex-row lg:items-center lg:justify-between">
      <div className="flex min-w-0 flex-1 flex-col gap-3 sm:flex-row">
        <input
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          placeholder="Tìm ghi chú"
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
      <Button variant="primary">
        <FontAwesomeIcon icon={faPlus} />
        Ghi chú mới
      </Button>
    </Card>
  );
}
