import { faArrowRight, faBolt } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Badge, Button, Card, Composer, ProgressBar } from "../../../components/ui";

export function LearningHero({
  learner,
  quickPrompts,
  prompt,
  onPromptChange,
  onPromptSelect,
  onSubmit,
}) {
  const weeklyProgress = learner.completedHours / learner.weeklyGoalHours;

  return (
    <section className="grid gap-4 lg:grid-cols-[minmax(0,1.45fr)_minmax(280px,0.55fr)]">
      <Card className="space-y-5 p-5 sm:p-6">
        <div className="flex flex-wrap items-center gap-2">
          <Badge tone="agent">AI mentor</Badge>
          <Badge tone="teal">{learner.focus}</Badge>
        </div>
        <div className="max-w-3xl space-y-3">
          <h2 className="text-2xl font-bold leading-tight text-[var(--text-primary)] sm:text-4xl">
            Hỏi, lập kế hoạch, luyện tập và theo dõi tiến độ trong một luồng học.
          </h2>
          <p className="text-base leading-7 text-[var(--text-secondary)]">
            Nhập mục tiêu học, hệ thống sẽ gợi ý roadmap nhỏ, tài liệu, bài tập và lịch ôn tập phù hợp.
          </p>
        </div>
        <Composer
          value={prompt}
          onChange={onPromptChange}
          onSubmit={onSubmit}
          submitLabel="Bắt đầu"
          actions={
            quickPrompts.map((item) => (
              <Button
                key={item}
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => onPromptSelect(item)}
              >
                {item}
              </Button>
            ))
          }
        />
      </Card>

      <Card tone="inverse" className="flex flex-col justify-between gap-6 p-5">
        <div>
          <div className="mb-4 flex h-10 w-10 items-center justify-center rounded-[var(--radius-md)] bg-[var(--sl-inverse-surface)]">
            <FontAwesomeIcon icon={faBolt} />
          </div>
          <p className="text-sm text-[var(--sl-inverse-panel-muted)]">Mục tiêu tuần</p>
          <h3 className="mt-1 text-3xl font-bold">
            {learner.completedHours}/{learner.weeklyGoalHours}h
          </h3>
        </div>
        <ProgressBar
          value={weeklyProgress}
          max={1}
          tone="success"
          label="Tiến độ mục tiêu tuần"
          size="lg"
        />
        <Button variant="accent" full>
          Xem lịch học
          <FontAwesomeIcon icon={faArrowRight} />
        </Button>
      </Card>
    </section>
  );
}
