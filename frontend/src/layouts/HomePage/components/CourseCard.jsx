import { faArrowRight } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Button, Card, ProgressBar } from "../../../components/ui";

export function CourseCard({ course }) {
  return (
    <Card interactive className="flex h-full flex-col gap-4">
      <div className="flex items-start justify-between gap-3">
        <Badge tone={course.tone}>{course.statusLabel}</Badge>
        <span className="text-xs font-semibold text-[var(--text-muted)]">{course.duration}</span>
      </div>
      <div className="min-w-0 flex-1">
        <h3 className="text-lg font-bold leading-snug text-[var(--text-primary)]">{course.title}</h3>
        <p className="mt-2 line-clamp-3 text-sm leading-6 text-[var(--text-secondary)]">
          {course.description}
        </p>
      </div>
      <ProgressBar
        value={course.progress}
        max={1}
        tone={course.tone}
        label={`${course.title} progress`}
        showLabel
      />
      <Button variant="secondary" full>
        Tiếp tục
        <FontAwesomeIcon icon={faArrowRight} />
      </Button>
    </Card>
  );
}
