export function ResourceFilters({
  courses,
  types,
  activeCourse,
  activeType,
  onCourseChange,
  onTypeChange,
}) {
  return (
    <div className="grid gap-3 sm:grid-cols-2">
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
      <select
        value={activeType}
        onChange={(event) => onTypeChange(event.target.value)}
        className="h-10 cursor-pointer rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface)] px-3 text-sm font-semibold text-[var(--text-primary)] focus:border-[var(--border-accent)] focus:outline-none focus:ring-2 focus:ring-[var(--focus-ring)]"
      >
        {types.map((type) => (
          <option key={type} value={type}>
            {type}
          </option>
        ))}
      </select>
    </div>
  );
}
