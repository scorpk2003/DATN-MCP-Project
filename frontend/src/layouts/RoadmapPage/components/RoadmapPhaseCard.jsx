import { faCheckCircle, faCircle } from "@fortawesome/free-regular-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Button, Card, ProgressBar } from "../../../components/ui";

export function RoadmapPhaseCard({ phase, onStartTask, startingId }) {
  const isCompleted = phase.status === "completed";

  return (
    <Card selected={phase.status === "in-progress"} className="space-y-4">
      <div className="flex items-start gap-3">
        <div className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-[var(--bg-surface-muted)] text-[var(--color-info)]">
          <FontAwesomeIcon icon={isCompleted ? faCheckCircle : faCircle} />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <h2 className="text-lg font-bold text-[var(--text-primary)]">{phase.title}</h2>
            <Badge tone={phase.tone}>{phase.statusLabel}</Badge>
          </div>
          <p className="mt-1 text-sm font-semibold text-[var(--text-muted)]">{phase.duration}</p>
        </div>
      </div>
      <ProgressBar value={phase.progress} max={1} tone={phase.tone} label={phase.title} showLabel />
      <ul className="space-y-2">
        {phase.tasks.map((task) => (
          <li
            key={task.id || task.title || task}
            className="flex items-center justify-between gap-3 text-sm text-[var(--text-secondary)]"
          >
            <span className="flex min-w-0 items-center gap-2">
              <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-[var(--color-info)]" />
              <span className="min-w-0 truncate">{task.title || task}</span>
            </span>
            {typeof task === "object" ? (
              <Button
                variant="ghost"
                size="sm"
                loading={startingId === (task.taskId || task.id)}
                disabled={Boolean(startingId)}
                onClick={() => onStartTask?.(phase, task)}
              >
                Học
              </Button>
            ) : null}
          </li>
        ))}
      </ul>
    </Card>
  );
}
