import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../../auth/useAuth.js";
import { EmptyState, ErrorState, LoadingState } from "../../components/ui";
import { useRoadmapData } from "../../hooks/useRoadmapData.js";
import { startLearningFlow } from "../../services/learningFlow.js";
import { ResourcePanel } from "./components/ResourcePanel.jsx";
import { RoadmapHero } from "./components/RoadmapHero.jsx";
import { RoadmapTimeline } from "./components/RoadmapTimeline.jsx";

function RoadmapPage() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const { data, error, loading, reload } = useRoadmapData();
  const { roadmapPhases, roadmapSummary, studyResources } = data;
  const [startingId, setStartingId] = useState("");
  const [actionError, setActionError] = useState("");

  const startRoadmapIntent = async (title, intent, options = {}) => {
    setStartingId(intent.payload?.taskId || intent.payload?.phaseId || intent.payload?.roadmapId || "roadmap");
    setActionError("");
    try {
      const { session } = await startLearningFlow({
        user,
        title,
        intent,
        metadata: { sourcePage: "roadmap" },
      });
      if (options.navigate !== false) {
        navigate("/", { state: { sessionId: session.id } });
      }
    } catch (flowError) {
      setActionError(flowError instanceof Error ? flowError.message : "Không thể bắt đầu học từ lộ trình.");
    } finally {
      setStartingId("");
    }
  };

  const handleScheduleUpdate = () => {
    startRoadmapIntent(`Update schedule: ${roadmapSummary.title}`, {
      type: "roadmap.schedule_update.requested",
      payload: {
        roadmapId: roadmapSummary.roadmapId || roadmapSummary.id,
        title: roadmapSummary.title,
        nextMilestone: roadmapSummary.nextMilestone,
      },
    }, { navigate: false });
  };

  const handleStartTask = (phase, task) => {
    startRoadmapIntent(`Learn: ${task.title}`, {
      type: "roadmap.task.selected",
      payload: {
        roadmapId: task.roadmapId || phase.roadmapId || roadmapSummary.roadmapId || roadmapSummary.id,
        phaseId: phase.phaseId || phase.id,
        milestoneId: task.milestoneId,
        taskId: task.taskId || task.id,
        title: task.title,
        description: task.description,
        level: task.level,
      },
    });
  };

  if (loading) {
    return <LoadingState layout="grid" title="Đang tải lộ trình..." />;
  }

  if (error) {
    return <ErrorState onRetry={reload} />;
  }

  if (actionError) {
    return <ErrorState title="Không thể bắt đầu phiên học" description={actionError} onRetry={() => setActionError("")} />;
  }

  if (!roadmapSummary) {
    return (
      <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-6">
        <EmptyState
          title="Chưa có lộ trình thật"
          description="Tạo lộ trình mới từ Agent workspace để xem nội dung ở đây."
        />
      </div>
    );
  }

  return (
    <div className="mx-auto flex w-full max-w-[var(--layout-max-width)] flex-col gap-6">
      <RoadmapHero summary={roadmapSummary} onScheduleUpdate={handleScheduleUpdate} loading={startingId === "roadmap"} />
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_360px]">
        <RoadmapTimeline phases={roadmapPhases} onStartTask={handleStartTask} startingId={startingId} />
        <ResourcePanel resources={studyResources} />
      </div>
    </div>
  );
}

export default RoadmapPage;
