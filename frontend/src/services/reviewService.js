import { reviewQueue } from "../data/reviewData.js";
import { request } from "../lib/httpClient.js";

export async function getReviewData() {
  return request("/api/review", {
    fallback: () => ({
      reviewQueue,
    }),
  });
}
