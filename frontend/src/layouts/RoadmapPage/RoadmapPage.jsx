import {
  roadmapPhases,
  roadmapSummary,
  studyResources,
} from "../../data/selfLearnDashboard.js";
import { ResourcePanel } from "./components/ResourcePanel.jsx";
import { RoadmapHero } from "./components/RoadmapHero.jsx";
import { RoadmapTimeline } from "./components/RoadmapTimeline.jsx";

function RoadmapPage() {
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
