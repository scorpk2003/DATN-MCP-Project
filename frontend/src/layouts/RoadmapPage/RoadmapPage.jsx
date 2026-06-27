import { ErrorState, LoadingState } from "../../components/ui";
import { useRoadmapData } from "../../hooks/useRoadmapData.js";
import { ResourcePanel } from "./components/ResourcePanel.jsx";
import { RoadmapHero } from "./components/RoadmapHero.jsx";
import { RoadmapTimeline } from "./components/RoadmapTimeline.jsx";

function RoadmapPage() {
  const { data, error, loading, reload } = useRoadmapData();
  const { roadmapPhases, roadmapSummary, studyResources } = data;

  if (loading) {
    return <LoadingState layout="grid" title="Đang tải lộ trình..." />;
  }

  if (error || !roadmapSummary) {
    return <ErrorState onRetry={reload} />;
  }

  return (
    <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-6">
      <RoadmapHero summary={roadmapSummary} />
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
        <RoadmapTimeline phases={roadmapPhases} />
        <ResourcePanel resources={studyResources} />
      </div>
    </div>
  );
}

export default RoadmapPage;
