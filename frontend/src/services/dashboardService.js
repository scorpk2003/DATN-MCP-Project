import {
  agentActivities,
  learner,
  learningMetrics,
  quickPrompts,
  recommendedCourses,
} from "../data/dashboardData.js";
import { request } from "../lib/httpClient.js";

export async function getDashboardData() {
  return request("/api/dashboard", {
    fallback: () => ({
      learner,
      quickPrompts,
      learningMetrics,
      recommendedCourses,
      agentActivities,
    }),
  });
}

export async function getRecommendedCourses() {
  return request("/api/courses/recommended", {
    fallback: () => recommendedCourses,
  });
}
