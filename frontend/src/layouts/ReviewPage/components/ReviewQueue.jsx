import { SectionTitle } from "../../../components/ui";
import { ReviewCard } from "./ReviewCard.jsx";

export function ReviewQueue({ items, onStartReview, startingId }) {
  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Spaced repetition" title="Hàng đợi ôn tập" />
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {items.map((item) => (
          <ReviewCard
            key={item.id}
            item={item}
            onStartReview={onStartReview}
            starting={startingId === (item.taskId || item.id)}
            disabled={Boolean(startingId)}
          />
        ))}
      </div>
    </section>
  );
}
