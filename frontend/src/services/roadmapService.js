import { roadmapPhases, roadmapSummary, studyResources } from "../data/roadmapData.js";
import { request } from "../lib/httpClient.js";

export async function getRoadmapData() {
  return request("/roadmap", {
    fallback: () => ({
      roadmapPhases,
      roadmapSummary,
      studyResources,
    }),
  });
}
