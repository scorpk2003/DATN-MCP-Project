import { SectionTitle } from "../../../components/ui";
import { ReviewCard } from "./ReviewCard.jsx";

export function ReviewQueue({ items }) {
  return (
    <section className="space-y-4">
      <SectionTitle eyebrow="Spaced repetition" title="Hàng đợi ôn tập" />
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {items.map((item) => (
          <ReviewCard key={item.id} item={item} />
        ))}
      </div>
    </section>
  );
}
