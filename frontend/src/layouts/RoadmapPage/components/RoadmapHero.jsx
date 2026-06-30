import { faCalendarCheck } from "@fortawesome/free-regular-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Button, Card, ProgressBar } from "../../../components/ui";

export function RoadmapHero({ summary, onScheduleUpdate, loading = false }) {
  return (
    <Card className="grid gap-5 p-5 sm:p-6 lg:grid-cols-[minmax(0,1fr)_280px]">
      <div className="min-w-0 space-y-4">
        <Badge tone="agent">{summary.currentSprint}</Badge>
        <div>
          <h1 className="text-3xl font-bold leading-tight text-[var(--text-primary)]">
            {summary.title}
          </h1>
          <p className="mt-3 max-w-3xl text-base leading-7 text-[var(--text-secondary)]">
            {summary.description}
          </p>
        </div>
      </div>
      <div className="flex flex-col justify-between gap-4 rounded-[var(--radius-md)] bg-[var(--bg-surface-muted)] p-4">
        <div>
          <p className="text-sm font-semibold text-[var(--text-muted)]">Milestone tiếp theo</p>
          <p className="mt-2 text-lg font-bold text-[var(--text-primary)]">{summary.nextMilestone}</p>
        </div>
        <ProgressBar value={summary.progress} max={1} tone="success" label="Roadmap progress" showLabel />
        <Button variant="secondary" onClick={onScheduleUpdate} loading={loading} aria-label="Update schedule">
          <FontAwesomeIcon icon={faCalendarCheck} />
          Cập nhật lịch
        </Button>
      </div>
    </Card>
  );
}
