import { faBookmark } from "@fortawesome/free-regular-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Button, Card } from "../../../components/ui";
import { ResourceFilters } from "./ResourceFilters.jsx";

export function ResourceToolbar({
  courses,
  types,
  activeCourse,
  activeType,
  query,
  onCourseChange,
  onQueryChange,
  onTypeChange,
}) {
  return (
    <Card className="space-y-3 p-4">
      <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
        <input
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          placeholder="Tìm tài liệu"
          className="h-10 min-w-0 flex-1 cursor-text rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface)] px-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:border-[var(--border-accent)] focus:outline-none focus:ring-2 focus:ring-[var(--focus-ring)]"
        />
        <Button variant="secondary">
          <FontAwesomeIcon icon={faBookmark} />
          Đã lưu
        </Button>
      </div>
      <ResourceFilters
        courses={courses}
        types={types}
        activeCourse={activeCourse}
        activeType={activeType}
        onCourseChange={onCourseChange}
        onTypeChange={onTypeChange}
      />
    </Card>
  );
}
