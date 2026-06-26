import { faClock, faLightbulb } from "@fortawesome/free-regular-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Card, ProgressBar } from "../../../components/ui";

export function ReviewSummary({ items }) {
  const dueToday = items.filter((item) => item.due === "Hôm nay").length;
  const averageConfidence =
    items.reduce((total, item) => total + item.confidence, 0) / Math.max(items.length, 1);

  return (
    <section className="grid gap-4 md:grid-cols-2">
      <Card className="flex items-center gap-4">
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-[var(--status-warning-bg)] text-[var(--color-warning)]">
          <FontAwesomeIcon icon={faClock} />
        </div>
        <div>
          <p className="text-sm font-semibold text-[var(--text-muted)]">Đến hạn hôm nay</p>
          <p className="mt-1 text-3xl font-bold text-[var(--text-primary)]">{dueToday}</p>
        </div>
      </Card>
      <Card className="space-y-3">
        <div className="flex items-center gap-3">
          <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-[var(--radius-md)] bg-[var(--status-info-bg)] text-[var(--color-info)]">
            <FontAwesomeIcon icon={faLightbulb} />
          </div>
          <div>
            <p className="text-sm font-semibold text-[var(--text-muted)]">Độ tự tin trung bình</p>
            <p className="mt-1 text-2xl font-bold text-[var(--text-primary)]">
              {Math.round(averageConfidence * 100)}%
            </p>
          </div>
        </div>
        <ProgressBar value={averageConfidence} max={1} tone="info" label="Average confidence" />
      </Card>
    </section>
  );
}
