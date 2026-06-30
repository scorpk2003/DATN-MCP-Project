import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../../auth/useAuth.js";
import { EmptyState, ErrorState, LoadingState, SectionTitle } from "../../components/ui";
import { useReviewData } from "../../hooks/useReviewData.js";
import { startLearningFlow } from "../../services/learningFlow.js";
import { ReviewQueue } from "./components/ReviewQueue.jsx";
import { ReviewSummary } from "./components/ReviewSummary.jsx";

function ReviewPage() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const { data, error, loading, reload } = useReviewData();
  const { reviewQueue } = data;
  const [startingId, setStartingId] = useState("");
  const [actionError, setActionError] = useState("");

  if (loading) {
    return <LoadingState layout="grid" title="Đang tải hàng đợi ôn tập..." />;
  }

  if (error) {
    return <ErrorState onRetry={reload} />;
  }

  if (actionError) {
    return <ErrorState title="Không thể bắt đầu ôn tập" description={actionError} onRetry={() => setActionError("")} />;
  }

  const handleStartReview = async (item) => {
    setStartingId(item.taskId || item.id);
    setActionError("");
    try {
      const { session } = await startLearningFlow({
        user,
        title: `Review: ${item.concept}`,
        intent: {
          type: "review.task.selected",
          payload: {
            taskId: item.taskId || item.id,
            concept: item.concept,
            course: item.course,
            confidence: item.confidence,
            due: item.due,
          },
        },
        metadata: { sourcePage: "review", taskId: item.taskId || item.id },
      });
      navigate("/", { state: { sessionId: session.id } });
    } catch (flowError) {
      setActionError(flowError instanceof Error ? flowError.message : "Không thể bắt đầu ôn tập.");
    } finally {
      setStartingId("");
    }
  };

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
          <ReviewQueue items={reviewQueue} onStartReview={handleStartReview} startingId={startingId} />
        </>
      ) : (
        <EmptyState title="Không có nội dung cần ôn tập hôm nay" />
      )}
    </div>
  );
}

export default ReviewPage;
