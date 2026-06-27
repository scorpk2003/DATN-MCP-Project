import { EmptyState, ErrorState, LoadingState, SectionTitle } from "../../components/ui";
import { useReviewData } from "../../hooks/useReviewData.js";
import { ReviewQueue } from "./components/ReviewQueue.jsx";
import { ReviewSummary } from "./components/ReviewSummary.jsx";

function ReviewPage() {
  const { data, error, loading, reload } = useReviewData();
  const { reviewQueue } = data;

  if (loading) {
    return <LoadingState layout="grid" title="Đang tải hàng đợi ôn tập..." />;
  }

  if (error) {
    return <ErrorState onRetry={reload} />;
  }

  return (
    <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-6">
      <SectionTitle
        eyebrow="Review"
        title="Ôn tập hôm nay"
        className="rounded-[var(--radius-card)] border border-[var(--border-primary)] bg-[var(--bg-surface)] p-5 shadow-[var(--shadow-card)]"
      />
      {reviewQueue.length > 0 ? (
        <>
          <ReviewSummary items={reviewQueue} />
          <ReviewQueue items={reviewQueue} />
        </>
      ) : (
        <EmptyState title="Không có nội dung cần ôn tập hôm nay" />
      )}
    </div>
  );
}

export default ReviewPage;
