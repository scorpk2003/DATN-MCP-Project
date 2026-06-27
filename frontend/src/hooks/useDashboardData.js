import { getDashboardData, getRecommendedCourses } from "../services/dashboardService.js";
import { useAsyncResource } from "./useAsyncResource.js";

const initialDashboardData = {
  agentActivities: [],
  learner: null,
  learningMetrics: [],
  quickPrompts: [],
  recommendedCourses: [],
};

export function useDashboardData() {
  return useAsyncResource(getDashboardData, initialDashboardData);
}

export function useRecommendedCourses() {
  return useAsyncResource(getRecommendedCourses, []);
}
