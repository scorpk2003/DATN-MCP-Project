import { reviewQueue } from "../../data/selfLearnDashboard.js";
import { SectionTitle } from "../../components/ui";
import { ReviewQueue } from "./components/ReviewQueue.jsx";
import { ReviewSummary } from "./components/ReviewSummary.jsx";

function ReviewPage() {
  return (
    <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-6">
      <SectionTitle
        eyebrow="Review"
        title="Ôn tập hôm nay"
        className="rounded-[var(--radius-card)] border border-[var(--border-primary)] bg-[var(--bg-surface)] p-5 shadow-[var(--shadow-card)]"
      />
      <ReviewSummary items={reviewQueue} />
      <ReviewQueue items={reviewQueue} />
    </div>
  );
}

export default ReviewPage;
