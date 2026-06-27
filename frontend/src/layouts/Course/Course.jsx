import Roadmap from "../../components/Roadmap";
import Item from "../../components/Item";
import { EmptyState, ErrorState, LoadingState } from "../../components/ui";
import { useRecommendedCourses } from "../../hooks/useDashboardData.js";

function Course({ study = false }) {
  const { data: recommendedCourses, error, loading, reload } = useRecommendedCourses();

  if (study) {
    return <Roadmap />;
  }

  if (loading) {
    return <LoadingState layout="grid" title="Đang tải khoá học..." />;
  }

  if (error) {
    return <ErrorState onRetry={reload} />;
  }

  if (recommendedCourses.length === 0) {
    return <EmptyState title="Chưa có khoá học được gợi ý" />;
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      {recommendedCourses.map((course) => (
        <Item
          key={course.id}
          name={course.title}
          description={course.description}
          complete={course.progress}
          duration={course.duration}
          statusLabel={course.statusLabel}
          tone={course.tone}
        />
      ))}
    </div>
  );
}

export default Course;
