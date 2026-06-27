import { getRoadmapData } from "../services/roadmapService.js";
import { useAsyncResource } from "./useAsyncResource.js";

const initialRoadmapData = {
  roadmapPhases: [],
  roadmapSummary: null,
  studyResources: [],
};

export function useRoadmapData() {
  return useAsyncResource(getRoadmapData, initialRoadmapData);
}
