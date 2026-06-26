import { faArrowRight } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Button, Card, ProgressBar } from "../../../components/ui";

export function ReviewCard({ item }) {
  return (
    <Card interactive className="flex flex-col gap-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <h2 className="text-lg font-bold text-[var(--text-primary)]">{item.concept}</h2>
          <p className="mt-1 text-sm text-[var(--text-muted)]">{item.course}</p>
        </div>
        <Badge tone={item.tone}>{item.due}</Badge>
      </div>
      <ProgressBar
        value={item.confidence}
        max={1}
        tone={item.tone}
        label="Mức tự tin"
        showLabel
      />
      <Button variant="secondary" full>
        Bắt đầu ôn
        <FontAwesomeIcon icon={faArrowRight} />
      </Button>
    </Card>
  );
}
